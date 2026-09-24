//! Replay sealed V9 Criterion samples against the direct, source-bound pair.
//! Trace: FR-051, NFR-009, TC-190, MP-062..MP-073.

use std::fs;

use serde_json::Value;
use sha2::{Digest, Sha256};

use super::{raw_bytes, DependencyResult, RawBundle};

const PARSE_CASES: &[&str] = &[
    "bounded_small",
    "past_small",
    "infinite_small",
    "infinite_fairness",
    "bounded_median",
    "past_median",
    "infinite_median",
    "bounded_near_node_cap",
    "past_near_node_cap",
    "infinite_near_node_cap",
];
const REWRITE_CASES: &[&str] = &["small_1", "median_24", "near_cap_64"];
const FAMILIES: &[&str] = &[
    "closed",
    "prefix",
    "lasso",
    "fairness",
    "c2po",
    "closed_trace",
    "closed_width",
];
const SCALES: &[&str] = &["small", "median", "near_cap"];
const SAMPLES: usize = 20;
const BOOTSTRAPS: usize = 2_000;

pub(super) enum Replay {
    Accept,
    Reject(&'static str),
    Inconclusive(&'static str),
}

pub(super) struct Member<'a> {
    crate_name: &'a str,
    group: &'static str,
    pair: u8,
    candidate: bool,
}

impl<'a> Member<'a> {
    pub(super) fn parse(name: &'a str) -> Option<Self> {
        let (crate_name, run) = name.strip_prefix("V9.")?.split_once('_')?;
        let group = match crate_name {
            "parse" => "parser_roundtrip",
            "rewrite" => "rewrite_rules",
            "mltl" => "v9_workloads",
            _ => return None,
        };
        let (pair, candidate) = match run {
            "pair1_baseline" => (1, false),
            "pair1_candidate" => (1, true),
            "pair2_baseline" => (2, false),
            "pair2_candidate" => (2, true),
            _ => return None,
        };
        Some(Self {
            crate_name,
            group,
            pair,
            candidate,
        })
    }

    fn repository(&self) -> String {
        if self.candidate {
            format!("tl-{}", self.crate_name)
        } else {
            format!("tl-{}-baseline", self.crate_name)
        }
    }

    fn dependency(&self) -> Option<String> {
        match (self.pair, self.candidate) {
            (1, false) => None,
            (1, true) => Some(format!("V9.{}_pair1_baseline", self.crate_name)),
            (2, false) => Some(format!("V9.{}_pair1_candidate", self.crate_name)),
            (2, true) => Some(format!("V9.{}_pair2_baseline", self.crate_name)),
            _ => unreachable!(),
        }
    }

    fn cases(&self) -> Vec<String> {
        match self.crate_name {
            "parse" => PARSE_CASES.iter().map(|s| (*s).into()).collect(),
            "rewrite" => REWRITE_CASES.iter().map(|s| (*s).into()).collect(),
            _ => SCALES
                .iter()
                .flat_map(|scale| {
                    FAMILIES
                        .iter()
                        .map(move |family| format!("{family}_{scale}"))
                })
                .collect(),
        }
    }
}

fn source_bound(definition: &Value, request: &Value, member: &Member<'_>) -> bool {
    let repository = member.repository();
    definition["sourceGraph"].as_array().is_some_and(|sources| {
        sources
            .iter()
            .filter(|source| source["repository"] == repository)
            .count()
            == 1
            && sources.iter().any(|source| {
                source["repository"] == repository
                    && source["revision"] == request["producer"]["sourceRevision"]
            })
    })
}

fn invocation_bound(request: &Value, member: &Member<'_>) -> bool {
    let mut arguments = vec!["bench", "--locked", "--offline"];
    if member.crate_name == "mltl" {
        arguments.extend(["--features", "infinite-trace"]);
    }
    arguments.extend(["--bench", member.group, "--", "--noplot"]);
    request["producer"]["name"] == "cargo"
        && request["producer"]["version"] == "1.98.1"
        && request["procedure"] == "direct"
        && request["arguments"].as_array().is_some_and(|items| {
            items.len() == arguments.len()
                && items
                    .iter()
                    .zip(arguments)
                    .all(|(item, argument)| item["kind"] == "literal" && item["value"] == argument)
        })
        && request["environment"]["CARGO_NET_OFFLINE"] == "true"
        && request["environment"]["CARGO_TARGET_DIR"] == ".quoin-target"
        && request["inputs"].as_array().is_some_and(Vec::is_empty)
        && request["outputs"].as_array().is_some_and(Vec::is_empty)
        && request["outputTrees"].as_array().is_some_and(|trees| {
            trees.len() == 1
                && trees[0]["role"] == "criterion"
                && trees[0]["path"] == ".quoin-target/criterion"
                && trees[0]["required"] == true
        })
}

fn host(result: &Value) -> Option<&Value> {
    let host = result.get("observedHost")?;
    let object = host.as_object()?;
    for key in [
        "machineDigest",
        "os",
        "kernelRelease",
        "architecture",
        "cpuModel",
        "runtimeClass",
    ] {
        if object
            .get(key)
            .and_then(Value::as_str)
            .is_none_or(str::is_empty)
        {
            return None;
        }
    }
    if object
        .get("logicalCpus")
        .and_then(Value::as_u64)
        .unwrap_or(0)
        == 0
        || object
            .get("memoryBytes")
            .and_then(Value::as_u64)
            .unwrap_or(0)
            == 0
    {
        return None;
    }
    Some(host)
}

fn samples(bundle: &RawBundle, group: &str, case: &str) -> Option<Vec<f64>> {
    let prefix = format!("criterion/{group}/{case}/new");
    let sample = raw_bytes(bundle, &format!("{prefix}/sample.json"))?;
    let estimate = raw_bytes(bundle, &format!("{prefix}/estimates.json"))?;
    let estimate: Value = serde_json::from_slice(estimate).ok()?;
    if !estimate["median"]["point_estimate"]
        .as_f64()
        .is_some_and(|v| v.is_finite() && v > 0.0)
    {
        return None;
    }
    let sample: Value = serde_json::from_slice(sample).ok()?;
    let iters = sample["iters"].as_array()?;
    let times = sample["times"].as_array()?;
    if iters.len() != SAMPLES || times.len() != SAMPLES {
        return None;
    }
    let values: Vec<f64> = iters
        .iter()
        .zip(times)
        .map(|(i, t)| {
            let i = i.as_f64()?;
            let t = t.as_f64()?;
            let value = t / i;
            (i.is_finite()
                && i > 0.0
                && t.is_finite()
                && t > 0.0
                && value.is_finite()
                && value > 0.0)
                .then_some(value)
        })
        .collect::<Option<_>>()?;
    let mut ordered = values.clone();
    let computed = median(&mut ordered);
    let reported = estimate["median"]["point_estimate"].as_f64()?;
    ((computed / reported - 1.0).abs() <= 0.01).then_some(values)
}

fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    (values[9] + values[10]) / 2.0
}

fn bootstrap_bounds(base: &[f64], candidate: &[f64], seed: &[u8]) -> (f64, f64) {
    let hash = Sha256::digest(seed);
    let mut state = u64::from_le_bytes(hash[..8].try_into().expect("eight hash bytes"));
    let mut changes = Vec::with_capacity(BOOTSTRAPS);
    for _ in 0..BOOTSTRAPS {
        let mut a = [0.0; SAMPLES];
        let mut b = [0.0; SAMPLES];
        for index in 0..SAMPLES {
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            a[index] = base[((z ^ (z >> 31)) % SAMPLES as u64) as usize];
            state = state.wrapping_add(0x9e3779b97f4a7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
            b[index] = candidate[((z ^ (z >> 31)) % SAMPLES as u64) as usize];
        }
        changes.push(median(&mut b) / median(&mut a) - 1.0);
    }
    changes.sort_by(f64::total_cmp);
    (changes[50], changes[1949])
}

pub(super) fn replay(
    name: &str,
    definition: &Value,
    request: &Value,
    result: &Value,
    bundle: &RawBundle,
    dependencies: &[DependencyResult],
) -> Replay {
    let Some(member) = Member::parse(name) else {
        return Replay::Reject("v9_member_unrecognized");
    };
    if !source_bound(definition, request, &member) {
        return Replay::Reject("v9_source_revision_mismatch");
    }
    if !invocation_bound(request, &member) {
        return Replay::Reject("v9_criterion_invocation_mismatch");
    }
    let Some(observed_host) = host(result) else {
        return Replay::Inconclusive("v9_host_unobserved");
    };
    let cases = member.cases();
    let mut own_samples = Vec::with_capacity(cases.len());
    for case in &cases {
        let Some(values) = samples(bundle, member.group, case) else {
            return Replay::Reject("v9_criterion_samples_missing_or_invalid");
        };
        own_samples.push(values);
    }
    let dependency = match member.dependency() {
        None if dependencies.is_empty() => return Replay::Accept,
        None => return Replay::Reject("v9_dependency_mismatch"),
        Some(expected) if dependencies.len() == 1 && dependencies[0].member == expected => {
            &dependencies[0]
        }
        Some(_) => return Replay::Reject("v9_dependency_mismatch"),
    };
    let Ok(dependency_result_bytes) = fs::read(&dependency.result_path) else {
        return Replay::Reject("v9_dependency_result_invalid");
    };
    if super::check(&dependency.member, "", &dependency_result_bytes).verdict != "accept" {
        return Replay::Reject("v9_dependency_execution_unproved");
    }
    let Ok(dependency_result) = serde_json::from_slice::<Value>(&dependency_result_bytes) else {
        return Replay::Reject("v9_dependency_result_invalid");
    };
    let Ok(dependency_request) = fs::read(&dependency.request_path)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<Value>(&bytes).ok())
        .ok_or(())
    else {
        return Replay::Reject("v9_dependency_request_invalid");
    };
    let Some(dependency_member) = Member::parse(&dependency.member) else {
        return Replay::Reject("v9_dependency_mismatch");
    };
    if !source_bound(definition, &dependency_request, &dependency_member) {
        return Replay::Reject("v9_dependency_source_revision_mismatch");
    }
    if !invocation_bound(&dependency_request, &dependency_member) {
        return Replay::Reject("v9_dependency_invocation_mismatch");
    }
    let Some(previous_host) = host(&dependency_result) else {
        return Replay::Inconclusive("v9_dependency_host_unobserved");
    };
    if observed_host != previous_host {
        return Replay::Inconclusive("v9_host_mismatch");
    }
    if request["producer"]["name"] != dependency_request["producer"]["name"]
        || request["producer"]["version"] != dependency_request["producer"]["version"]
        || request["producer"]["executableDigest"]
            != dependency_request["producer"]["executableDigest"]
    {
        return Replay::Inconclusive("v9_toolchain_mismatch");
    }
    if !member.candidate {
        return Replay::Accept;
    }
    let Ok(previous_bundle) =
        super::sealed_raw_bundle(&dependency.raw_bundle_path, &dependency.raw_bundle_digest)
    else {
        return Replay::Reject("v9_dependency_samples_unsealed");
    };
    let mut overlap = false;
    let mut repeat_required = false;
    for (case, candidate_samples) in cases.iter().zip(own_samples) {
        let Some(baseline_samples) = samples(&previous_bundle, member.group, case) else {
            return Replay::Reject("v9_baseline_samples_missing_or_invalid");
        };
        let seed = format!("{name}/{case}");
        let (lower, upper) =
            bootstrap_bounds(&baseline_samples, &candidate_samples, seed.as_bytes());
        if lower > 0.20 {
            repeat_required = true;
        } else if upper > 0.20 {
            overlap = true;
        }
    }
    if repeat_required {
        Replay::Inconclusive("v9_repeat_required_above_20pct")
    } else if overlap {
        Replay::Inconclusive("v9_threshold_overlap")
    } else {
        Replay::Accept
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // Synthetic Criterion trees exercise the sealed replay contract. They are
    // not EA runs or a V9 campaign receipt.
    fn fixture(rate: f64, observed_machine: &str, revision: &str) -> (Value, Value, RawBundle) {
        let arguments: Vec<Value> = [
            "bench",
            "--locked",
            "--offline",
            "--bench",
            "rewrite_rules",
            "--",
            "--noplot",
        ]
        .iter()
        .map(|value| json!({"kind":"literal","value":value}))
        .collect();
        let request = json!({
            "producer": {"name":"cargo", "version":"1.98.1", "sourceRevision":revision,
                "executableDigest":"a".repeat(64)},
            "procedure":"direct",
            "arguments":arguments,
            "environment":{"CARGO_NET_OFFLINE":"true","CARGO_TARGET_DIR":".quoin-target"},
            "inputs":[], "outputs":[],
            "outputTrees":[{"role":"criterion","path":".quoin-target/criterion","required":true}]
        });
        let result = json!({
            "protocol":super::super::RESULT_PROTOCOL,
            "requestIdentity":{"digest":"b".repeat(64)},
            "state":{"kind":"completed"},
            "process":{"terminalStatus":{"kind":"exit_code","value":0},
                "stdout":{"bytes":[],"digest":super::super::sha256(&[]),"truncated":false},
                "stderr":{"bytes":[],"digest":super::super::sha256(&[]),"truncated":false}},
            "observedHost":{
            "machineDigest":observed_machine, "os":"linux", "kernelRelease":"6.0",
            "architecture":"x86_64", "cpuModel":"test cpu", "logicalCpus":8,
            "memoryBytes":1024, "runtimeClass":"native"
        }});
        let mut artifacts = Vec::new();
        for case in REWRITE_CASES {
            let prefix = format!("criterion/rewrite_rules/{case}/new");
            for (suffix, value) in [
                (
                    "sample.json",
                    json!({"iters":vec![1; SAMPLES],"times":vec![rate; SAMPLES]}),
                ),
                ("estimates.json", json!({"median":{"point_estimate":rate}})),
            ] {
                let bytes = serde_json::to_vec(&value).unwrap();
                artifacts.push(super::super::RawArtifactBytes {
                    role: format!("{prefix}/{suffix}"),
                    digest: super::super::sha256(&bytes),
                    bytes,
                });
            }
        }
        artifacts.sort_by(|a, b| a.role.cmp(&b.role));
        (
            request,
            result,
            RawBundle {
                schema: "quoin.raw-artifact-bundle/v1".into(),
                artifacts,
            },
        )
    }

    fn definition() -> Value {
        json!({"sourceGraph":[
            {"repository":"tl-rewrite-baseline","revision":"base"},
            {"repository":"tl-rewrite","revision":"candidate"}
        ]})
    }

    fn make_dependency(
        dir: &std::path::Path,
        request: &Value,
        result: &Value,
        bundle: &RawBundle,
    ) -> DependencyResult {
        let request_path = dir.join("request.json");
        let result_path = dir.join("result.json");
        let raw_bundle_path = dir.join("bundle.json");
        fs::write(&request_path, serde_json::to_vec(request).unwrap()).unwrap();
        fs::write(&result_path, serde_json::to_vec(result).unwrap()).unwrap();
        let bundle_value = serde_json::to_value(bundle).unwrap();
        fs::write(&raw_bundle_path, serde_json::to_vec(&bundle_value).unwrap()).unwrap();
        DependencyResult {
            member: "V9.rewrite_pair1_baseline".into(),
            index: 1,
            request_digest: String::new(),
            request_path: request_path.to_string_lossy().into(),
            result_digest: String::new(),
            result_path: result_path.to_string_lossy().into(),
            raw_bundle_digest: super::super::canonical_digest(&bundle_value).unwrap(),
            raw_bundle_path: raw_bundle_path.to_string_lossy().into(),
        }
    }

    #[test]
    fn synthetic_sealed_samples_pass_only_when_source_host_and_change_are_bounded() {
        let directory = tempfile::tempdir_in("/private/tmp").unwrap();
        let (base_request, base_result, base_bundle) = fixture(100.0, "host-a", "base");
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_baseline",
                &definition(),
                &base_request,
                &base_result,
                &base_bundle,
                &[]
            ),
            Replay::Accept
        ));
        let dependency =
            make_dependency(directory.path(), &base_request, &base_result, &base_bundle);
        let (request, result, bundle) = fixture(110.0, "host-a", "candidate");
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_candidate",
                &definition(),
                &request,
                &result,
                &bundle,
                &[dependency]
            ),
            Replay::Accept
        ));
        let dependency =
            make_dependency(directory.path(), &base_request, &base_result, &base_bundle);
        let (_, _, slow_bundle) = fixture(130.0, "host-a", "candidate");
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_candidate",
                &definition(),
                &request,
                &result,
                &slow_bundle,
                &[dependency]
            ),
            Replay::Inconclusive("v9_repeat_required_above_20pct")
        ));
        let dependency =
            make_dependency(directory.path(), &base_request, &base_result, &base_bundle);
        let (_, other_host, _) = fixture(110.0, "host-b", "candidate");
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_candidate",
                &definition(),
                &request,
                &other_host,
                &bundle,
                &[dependency]
            ),
            Replay::Inconclusive("v9_host_mismatch")
        ));
    }

    #[test]
    fn synthetic_missing_or_unbound_criterion_evidence_rejects() {
        let (request, result, mut bundle) = fixture(100.0, "host-a", "base");
        bundle
            .artifacts
            .retain(|artifact| !artifact.role.ends_with("sample.json"));
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_baseline",
                &definition(),
                &request,
                &result,
                &bundle,
                &[]
            ),
            Replay::Reject("v9_criterion_samples_missing_or_invalid")
        ));
        let (mut request, result, bundle) = fixture(100.0, "host-a", "base");
        request["producer"]["sourceRevision"] = json!("candidate");
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_baseline",
                &definition(),
                &request,
                &result,
                &bundle,
                &[]
            ),
            Replay::Reject("v9_source_revision_mismatch")
        ));
        request["producer"]["sourceRevision"] = json!("base");
        request["arguments"][2]["value"] = json!("--release");
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_baseline",
                &definition(),
                &request,
                &result,
                &bundle,
                &[]
            ),
            Replay::Reject("v9_criterion_invocation_mismatch")
        ));
    }
}
