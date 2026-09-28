//! Replay sealed V9 Criterion samples against the direct, source-bound pair.
//! Trace: FR-051, NFR-009, TC-190, MP-062..MP-073.

use std::{collections::BTreeMap, fs};

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
    ConfirmedRegression(String),
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

    fn dependencies(&self) -> Vec<String> {
        match (self.pair, self.candidate) {
            (1, false) => vec![],
            (1, true) => vec![format!("V9.{}_pair1_baseline", self.crate_name)],
            (2, false) => vec![format!("V9.{}_pair1_candidate", self.crate_name)],
            (2, true) => vec![
                format!("V9.{}_pair1_baseline", self.crate_name),
                format!("V9.{}_pair1_candidate", self.crate_name),
                format!("V9.{}_pair2_baseline", self.crate_name),
            ],
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
        && harness_inputs(request, member).is_some()
        && request["outputs"].as_array().is_some_and(Vec::is_empty)
        && request["outputTrees"].as_array().is_some_and(|trees| {
            trees.len() == 1
                && trees[0]["role"] == "criterion"
                && trees[0]["path"] == ".quoin-target/criterion"
                && trees[0]["required"] == true
        })
}

fn required_harness_paths(member: &Member<'_>) -> &'static [&'static str] {
    match member.crate_name {
        "parse" => &[
            "benches/parser_roundtrip.rs",
            "benches/inputs/SHA256SUMS",
            "benches/inputs/bounded-small.txt",
            "benches/inputs/infinite-fairness.txt",
            "benches/inputs/infinite-small.txt",
            "benches/inputs/past-small.txt",
            "benches/inputs/shared-median.txt",
            "benches/inputs/shared-near-node-cap.txt",
        ],
        "rewrite" => &["benches/rewrite_rules.rs", "benches/input-digests.json"],
        _ => &["benches/v9_workloads.rs", "benches/input-digests.json"],
    }
}

fn harness_inputs(
    request: &Value,
    member: &Member<'_>,
) -> Option<BTreeMap<String, (String, bool)>> {
    let mut all_paths = std::collections::BTreeSet::new();
    let mut harness = BTreeMap::new();
    for input in request["inputs"].as_array()? {
        let role = input["role"].as_str()?;
        let (path, executable) = role
            .strip_prefix("source/")
            .map(|path| (path, false))
            .or_else(|| role.strip_prefix("source-exec/").map(|path| (path, true)))?;
        let digest = input["digest"].as_str()?;
        let serialized_executable = match input.get("executable") {
            None => false,
            Some(Value::Bool(value)) => *value,
            _ => return None,
        };
        if path != input["path"].as_str()?
            || path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
            || serialized_executable != executable
            || digest.len() != 64
            || !digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !all_paths.insert(path)
        {
            return None;
        }
        if required_harness_paths(member).contains(&path) {
            harness.insert(path.to_owned(), (digest.to_owned(), executable));
        }
    }
    required_harness_paths(member)
        .iter()
        .all(|path| harness.contains_key(*path))
        .then_some(harness)
}

fn host(result: &Value) -> Option<&Value> {
    let host = result.get("observedHost")?;
    let object = host.as_object()?;
    let digest = |value: &Value| {
        value.as_str().is_some_and(|value| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        })
    };
    if !object.get("machineDigest").is_some_and(digest) {
        return None;
    }
    match object.get("identitySource") {
        None => {
            if object.contains_key("cpuAffinityDigest") {
                return None;
            }
        }
        Some(Value::String(source)) if source == "kernel_boot_id" => {
            if !object.get("cpuAffinityDigest").is_some_and(digest) {
                return None;
            }
        }
        Some(_) => return None,
    }
    if object
        .get("cpuModelSource")
        .is_some_and(|source| source != "arm_cpu_id")
    {
        return None;
    }
    for key in [
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
    let expected = member.dependencies();
    if dependencies.len() != expected.len()
        || dependencies
            .iter()
            .zip(&expected)
            .any(|(dependency, expected)| dependency.member != *expected)
    {
        return Replay::Reject("v9_dependency_mismatch");
    }
    let own_harness = harness_inputs(request, &member).expect("invocation was checked");
    let mut previous_bundles = BTreeMap::new();
    for dependency in dependencies {
        let Ok(dependency_result_bytes) = fs::read(&dependency.result_path) else {
            return Replay::Reject("v9_dependency_result_invalid");
        };
        if super::check(&dependency.member, "", &dependency_result_bytes).verdict != "accept" {
            return Replay::Reject("v9_dependency_execution_unproved");
        }
        let Ok(dependency_result) = serde_json::from_slice::<Value>(&dependency_result_bytes)
        else {
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
        if own_harness
            != harness_inputs(&dependency_request, &dependency_member)
                .expect("invocation was checked")
        {
            return Replay::Reject("v9_harness_input_mismatch");
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
        let Ok(previous_bundle) =
            super::sealed_raw_bundle(&dependency.raw_bundle_path, &dependency.raw_bundle_digest)
        else {
            return Replay::Reject("v9_dependency_samples_unsealed");
        };
        if cases
            .iter()
            .any(|case| samples(&previous_bundle, member.group, case).is_none())
        {
            return Replay::Reject("v9_dependency_samples_missing_or_invalid");
        }
        previous_bundles.insert(dependency.member.as_str(), previous_bundle);
    }
    if !member.candidate {
        return Replay::Accept;
    }
    let baseline_name = format!("V9.{}_pair{}_baseline", member.crate_name, member.pair);
    let Some(previous_bundle) = previous_bundles.get(baseline_name.as_str()) else {
        return Replay::Reject("v9_dependency_mismatch");
    };
    let mut overlap = false;
    let mut repeat_required = false;
    for (case, candidate_samples) in cases.iter().zip(own_samples) {
        let Some(baseline_samples) = samples(previous_bundle, member.group, case) else {
            return Replay::Reject("v9_baseline_samples_missing_or_invalid");
        };
        let seed = format!("{name}/{case}");
        let (lower, upper) =
            bootstrap_bounds(&baseline_samples, &candidate_samples, seed.as_bytes());
        if lower > 0.20 {
            if member.pair == 2 {
                let first_baseline = format!("V9.{}_pair1_baseline", member.crate_name);
                let first_candidate = format!("V9.{}_pair1_candidate", member.crate_name);
                let earlier_base = previous_bundles
                    .get(first_baseline.as_str())
                    .expect("required direct dependency");
                let earlier_candidate = previous_bundles
                    .get(first_candidate.as_str())
                    .expect("required direct dependency");
                let earlier_base_samples =
                    samples(earlier_base, member.group, case).expect("validated samples");
                let earlier_candidate_samples =
                    samples(earlier_candidate, member.group, case).expect("validated samples");
                let first_seed = format!("V9.{}_pair1_candidate/{case}", member.crate_name);
                let (first_lower, _) = bootstrap_bounds(
                    &earlier_base_samples,
                    &earlier_candidate_samples,
                    first_seed.as_bytes(),
                );
                if first_lower > 0.20 {
                    return Replay::ConfirmedRegression(format!("{}/{case}", member.group));
                }
            }
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
        let harness_inputs: Vec<Value> =
            required_harness_paths(&Member::parse("V9.rewrite_pair1_baseline").unwrap())
                .iter()
                .map(|path| {
                    json!({
                        "role":format!("source/{path}"), "path":path,
                        "digest":"c".repeat(64)
                    })
                })
                .collect();
        let request = json!({
            "producer": {"name":"cargo", "version":"1.98.1", "sourceRevision":revision,
                "executableDigest":"a".repeat(64)},
            "procedure":"direct",
            "arguments":arguments,
            "environment":{"CARGO_NET_OFFLINE":"true","CARGO_TARGET_DIR":".quoin-target"},
            "inputs":harness_inputs, "outputs":[],
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
            "machineDigest":super::super::sha256(observed_machine.as_bytes()), "os":"linux", "kernelRelease":"6.0",
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
        member: &str,
        request: &Value,
        result: &Value,
        bundle: &RawBundle,
    ) -> DependencyResult {
        let request_path = dir.join(format!("{member}-request.json"));
        let result_path = dir.join(format!("{member}-result.json"));
        let raw_bundle_path = dir.join(format!("{member}-bundle.json"));
        let mut request = request.clone();
        request["protocol"] = json!("engineering-assurance.producer-execution-request/v1");
        let request_digest = super::super::canonical_digest(&request).unwrap();
        let mut result = result.clone();
        result["producer"] = request["producer"].clone();
        result["requestIdentity"]["digest"] = json!(request_digest);
        result["artifacts"] = json!(bundle
            .artifacts
            .iter()
            .map(|artifact| json!({
                "role":artifact.role.clone(),"digest":artifact.digest.clone()
            }))
            .collect::<Vec<_>>());
        let result_digest = super::super::canonical_digest(&result).unwrap();
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        fs::write(&result_path, serde_json::to_vec(&result).unwrap()).unwrap();
        let bundle_value = serde_json::to_value(bundle).unwrap();
        fs::write(&raw_bundle_path, serde_json::to_vec(&bundle_value).unwrap()).unwrap();
        DependencyResult {
            member: member.into(),
            index: 1,
            request_digest,
            request_path: request_path.to_string_lossy().into(),
            result_digest,
            result_path: result_path.to_string_lossy().into(),
            raw_bundle_digest: super::super::canonical_digest(&bundle_value).unwrap(),
            raw_bundle_path: raw_bundle_path.to_string_lossy().into(),
        }
    }

    #[test]
    // Trace: TC-190, FR-051-AC-1
    fn boot_scoped_host_requires_exact_affinity_and_known_source() {
        let (_, mut result, _) = fixture(100.0, "fictional-host", "base");
        assert!(host(&result).is_some());
        result["observedHost"]["identitySource"] = json!("kernel_boot_id");
        assert!(host(&result).is_none());
        result["observedHost"]["cpuAffinityDigest"] = json!("a".repeat(64));
        assert!(host(&result).is_some());
        result["observedHost"]["cpuAffinityDigest"] = json!("wrong");
        assert!(host(&result).is_none());
        result["observedHost"]["cpuAffinityDigest"] = json!("a".repeat(64));
        result["observedHost"]["identitySource"] = json!("unrecognized");
        assert!(host(&result).is_none());
        result["observedHost"]["identitySource"] = json!("kernel_boot_id");
        result["observedHost"]["cpuModelSource"] = json!("arm_cpu_id");
        assert!(host(&result).is_some());
        result["observedHost"]["machineDigest"] = json!("A".repeat(64));
        assert!(host(&result).is_none());
    }

    #[test]
    // Trace: TC-190, FR-051-AC-1
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
        let dependency = make_dependency(
            directory.path(),
            "V9.rewrite_pair1_baseline",
            &base_request,
            &base_result,
            &base_bundle,
        );
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
        let dependency = make_dependency(
            directory.path(),
            "V9.rewrite_pair1_baseline",
            &base_request,
            &base_result,
            &base_bundle,
        );
        let mut extra_report = request.clone();
        extra_report["inputs"].as_array_mut().unwrap().push(json!({
            "role":"source/benches/README.md","path":"benches/README.md","digest":"e".repeat(64)
        }));
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_candidate",
                &definition(),
                &extra_report,
                &result,
                &bundle,
                &[dependency]
            ),
            Replay::Accept
        ));
        let dependency = make_dependency(
            directory.path(),
            "V9.rewrite_pair1_baseline",
            &base_request,
            &base_result,
            &base_bundle,
        );
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
        let dependency = make_dependency(
            directory.path(),
            "V9.rewrite_pair1_baseline",
            &base_request,
            &base_result,
            &base_bundle,
        );
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
        let dependency = make_dependency(
            directory.path(),
            "V9.rewrite_pair1_baseline",
            &base_request,
            &base_result,
            &base_bundle,
        );
        let mut different_harness = request.clone();
        different_harness["inputs"][0]["digest"] = json!("d".repeat(64));
        assert!(matches!(
            replay(
                "V9.rewrite_pair1_candidate",
                &definition(),
                &different_harness,
                &result,
                &bundle,
                &[dependency]
            ),
            Replay::Reject("v9_harness_input_mismatch")
        ));
    }

    #[test]
    // Trace: TC-190, FR-051-AC-1
    fn synthetic_two_pairs_name_a_confirmed_regression() {
        let directory = tempfile::tempdir_in("/private/tmp").unwrap();
        let (base_request, base_result, base_bundle) = fixture(100.0, "host-a", "base");
        let (candidate_request, candidate_result, candidate_bundle) =
            fixture(130.0, "host-a", "candidate");
        let dependencies = vec![
            make_dependency(
                directory.path(),
                "V9.rewrite_pair1_baseline",
                &base_request,
                &base_result,
                &base_bundle,
            ),
            make_dependency(
                directory.path(),
                "V9.rewrite_pair1_candidate",
                &candidate_request,
                &candidate_result,
                &candidate_bundle,
            ),
            make_dependency(
                directory.path(),
                "V9.rewrite_pair2_baseline",
                &base_request,
                &base_result,
                &base_bundle,
            ),
        ];
        assert!(
            matches!(replay("V9.rewrite_pair2_candidate", &definition(), &candidate_request,
            &candidate_result, &candidate_bundle, &dependencies),
            Replay::ConfirmedRegression(case) if case == "rewrite_rules/small_1")
        );
        let (_, _, recovered_bundle) = fixture(110.0, "host-a", "candidate");
        assert!(matches!(
            replay(
                "V9.rewrite_pair2_candidate",
                &definition(),
                &candidate_request,
                &candidate_result,
                &recovered_bundle,
                &dependencies
            ),
            Replay::Accept
        ));
    }

    fn sealed_pair2_receipt(root: &std::path::Path, current_rate: f64) -> (Value, bool) {
        let (base_request, base_result, base_bundle) = fixture(100.0, "host-a", "base");
        let (candidate_request, candidate_result, first_candidate_bundle) =
            fixture(130.0, "host-a", "candidate");
        let (_, _, second_candidate_bundle) = fixture(current_rate, "host-a", "candidate");
        let dependencies = vec![
            make_dependency(
                root,
                "V9.rewrite_pair1_baseline",
                &base_request,
                &base_result,
                &base_bundle,
            ),
            make_dependency(
                root,
                "V9.rewrite_pair1_candidate",
                &candidate_request,
                &candidate_result,
                &first_candidate_bundle,
            ),
            make_dependency(
                root,
                "V9.rewrite_pair2_baseline",
                &base_request,
                &base_result,
                &base_bundle,
            ),
        ];
        let name = "V9.rewrite_pair2_candidate";
        let current = make_dependency(
            root,
            name,
            &candidate_request,
            &candidate_result,
            &second_candidate_bundle,
        );
        let mut definition = definition();
        definition["schemaVersion"] = json!("engineering-assurance.campaign-definition/v1");
        definition["members"] = json!([{
            "name":name,"planId":"MP-069","definitionVersion":"tl.v9.test/v1","required":true,
            "dependsOn":dependencies.iter().map(|dependency| dependency.member.clone()).collect::<Vec<_>>()
        }]);
        let definition_path = root.join("definition.json");
        fs::write(&definition_path, serde_json::to_vec(&definition).unwrap()).unwrap();
        let input = json!({
            "schema":"quoin.domain-check-input/v1",
            "definitionPath":definition_path,
            "definitionDigest":super::super::canonical_digest(&definition).unwrap(),
            "member":name,"planId":"MP-069","definitionVersion":"tl.v9.test/v1",
            "sourceGraphDigest":super::super::canonical_digest(&definition["sourceGraph"]).unwrap(),
            "requestDigest":current.request_digest,"requestPath":current.request_path,
            "resultDigest":current.result_digest,"resultPath":current.result_path,
            "rawArtifacts":second_candidate_bundle.artifacts.iter().map(|artifact| json!({
                "role":artifact.role,"digest":artifact.digest
            })).collect::<Vec<_>>(),
            "rawBundlePath":current.raw_bundle_path,"rawBundleDigest":current.raw_bundle_digest,
            "dependencies":dependencies
        });
        let input_path = root.join("check-input.json");
        let output_path = root.join("verdict.json");
        fs::write(&input_path, serde_json::to_vec(&input).unwrap()).unwrap();
        let status = super::super::run_args(&[
            "--input".into(),
            input_path.to_string_lossy().into(),
            "--output".into(),
            output_path.to_string_lossy().into(),
        ])
        .is_ok();
        let receipt = serde_json::from_slice(&fs::read(output_path).unwrap()).unwrap();
        (receipt, status)
    }

    #[test]
    // Trace: TC-190, FR-051-AC-1
    fn synthetic_ea_style_source_projection_reaches_sealed_pair2_verdict() {
        let directory = tempfile::tempdir_in("/private/tmp").unwrap();
        let (accepted, status) = sealed_pair2_receipt(directory.path(), 110.0);
        assert!(status, "{accepted:#}");
        assert_eq!(accepted["verdict"], "accept");
        assert_eq!(accepted["reasons"], json!([]));
        let (regression, status) = sealed_pair2_receipt(directory.path(), 130.0);
        assert!(!status);
        assert_eq!(regression["verdict"], "reject");
        assert_eq!(
            regression["reasons"],
            json!(["v9_confirmed_regression_above_20pct:rewrite_rules/small_1"])
        );
    }

    #[test]
    // Trace: TC-190, FR-051-AC-1
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
        request["arguments"][2]["value"] = json!("--offline");
        request["inputs"][0]["executable"] = json!(true);
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
