//! TL domain checker for one EA-executed Campaign member.
//!
//! This program never launches a producer. EA executes each declared native
//! command directly; Quoin retains that result and supplies its exact bytes.
//! Trace: FR-055-AC-1, FR-055-AC-2, TC-197, TC-198.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

#[path = "tl_campaign_check/v10_replay.rs"]
mod v10_replay;
#[path = "tl_campaign_check/v10_static.rs"]
mod v10_static;
#[path = "tl_campaign_check/v9_replay.rs"]
mod v9_replay;

const RESULT_PROTOCOL: &str = "engineering-assurance.producer-execution-result/v1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RequestIdentity {
    digest: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CapturedStream {
    bytes: Vec<u8>,
    digest: String,
    truncated: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalStatus {
    kind: String,
    value: i32,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ProcessEvidence {
    terminal_status: Option<TerminalStatus>,
    stdout: CapturedStream,
    stderr: CapturedStream,
}

#[derive(Deserialize)]
struct ExecutionState {
    kind: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ExecutionResult {
    protocol: String,
    request_identity: RequestIdentity,
    process: Option<ProcessEvidence>,
    state: ExecutionState,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CheckInput {
    schema: String,
    definition_path: String,
    definition_digest: String,
    member: String,
    plan_id: String,
    definition_version: String,
    source_graph_digest: String,
    request_digest: String,
    request_path: String,
    result_path: String,
    result_digest: String,
    raw_artifacts: Vec<RawArtifact>,
    raw_bundle_path: String,
    raw_bundle_digest: String,
    #[serde(default)]
    dependencies: Vec<DependencyResult>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawArtifact {
    role: String,
    digest: String,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawArtifactBytes {
    role: String,
    digest: String,
    bytes: Vec<u8>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawBundle {
    schema: String,
    artifacts: Vec<RawArtifactBytes>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DependencyResult {
    member: String,
    index: u64,
    request_digest: String,
    request_path: String,
    result_digest: String,
    result_path: String,
    raw_bundle_digest: String,
    raw_bundle_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DomainVerdict {
    schema: &'static str,
    member: String,
    verdict: &'static str,
    reasons: Vec<String>,
    definition_digest: String,
    plan_id: String,
    definition_version: String,
    source_graph_digest: String,
    request_digest: String,
    result_digest: String,
    raw_artifacts_digest: String,
    raw_bundle_digest: String,
    dependencies_digest: String,
    stdout_digest: Option<String>,
    stderr_digest: Option<String>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn canonical_digest(value: &Value) -> Result<String, String> {
    serde_json_canonicalizer::to_vec(value)
        .map(|bytes| sha256(&bytes))
        .map_err(|error| format!("cannot canonicalize JSON: {error}"))
}

fn raw_bytes<'a>(bundle: &'a RawBundle, role: &str) -> Option<&'a [u8]> {
    let mut matches = bundle.artifacts.iter().filter(|item| item.role == role);
    let item = matches.next()?;
    matches.next().is_none().then_some(item.bytes.as_slice())
}

fn sealed_raw_bundle(path: &str, digest: &str) -> Result<RawBundle, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let value: Value = serde_json::from_slice(&bytes).map_err(|error| error.to_string())?;
    if canonical_digest(&value)? != digest {
        return Err("raw bundle canonical digest mismatch".into());
    }
    let bundle: RawBundle = serde_json::from_value(value).map_err(|error| error.to_string())?;
    if bundle.schema != "quoin.raw-artifact-bundle/v1"
        || bundle
            .artifacts
            .windows(2)
            .any(|pair| pair[0].role >= pair[1].role)
        || bundle
            .artifacts
            .iter()
            .any(|item| sha256(&item.bytes) != item.digest)
    {
        return Err("raw bundle bytes or inventory mismatch".into());
    }
    Ok(bundle)
}

fn member_parser(member: &str) -> Option<&'static str> {
    match member {
        "V1.independent_oracle"
        | "V1.oracle_fault_injection"
        | "V1.oracle_dependency_boundary"
        | "V1.production_finite_faults"
        | "V1.production_infinite_faults"
        | "V1.finite_lasso_oracle"
        | "V11.infinite_oracle"
        | "V11.infinite_trace_behavior"
        | "V11.oracle_semantic_laws" => Some("cargo-test"),
        "V2.finite_small_partition" => Some("finite-partition"),
        "V2.full_domain_census" => Some("full-domain"),
        "V3.semantic_properties" => Some("semantic-properties"),
        "V11.lasso_population_census" => Some("lasso-partition"),
        "V4.syntax_infinite_wire_decode"
        | "V4.parse_unbounded_roundtrip"
        | "V4.rewrite_infinite_rewrite"
        | "V4.mltl_c2po_map"
        | "V4.mltl_closed_eval" => Some("libfuzzer"),
        "V7.embedded_core" | "V7.embedded_alloc" | "V7.embedded_serde" => Some("embedded-build"),
        "V7.syntax_formula_limits"
        | "V7.syntax_borrowed_ownership"
        | "V7.parse_limits_utf8"
        | "V7.mltl_lasso_limits"
        | "V7.mltl_prefix_limits"
        | "V7.rewrite_budgets"
        | "V7.rewrite_record_limits"
        | "V7.oracle_limits" => Some("miri"),
        "V6.syntax_interval_proof" | "V6.mltl_horizon_proof" => Some("kani-clean"),
        "V6.seeded_false_claim" => Some("kani-false"),
        "V6.ordinary_counterexample_replay" => Some("kani-replay"),
        "V8.parse_example_prep" => Some("embedded-build"),
        "V8.syntax_core"
        | "V8.syntax_alloc"
        | "V8.syntax_serde"
        | "V8.parse_default"
        | "V8.mltl_default"
        | "V8.mltl_infinite"
        | "V8.rewrite_default"
        | "V8.rewrite_infinite" => Some("llvm-cov"),
        "V5.parse_discovery"
        | "V5.syntax_discovery"
        | "V5.mltl_discovery"
        | "V5.rewrite_discovery" => Some("mutants-discovery"),
        "V5.parse_mutation" | "V5.syntax_mutation" | "V5.mltl_mutation" | "V5.rewrite_mutation" => {
            Some("mutants-run")
        }
        "V5.parse_restored_control"
        | "V5.syntax_restored_control"
        | "V5.mltl_restored_control"
        | "V5.rewrite_restored_control" => Some("mutants-restored"),
        "V10.inputs" => Some("v10-inputs"),
        _ => None,
    }
    .or_else(|| {
        if member.starts_with("V10.compile.") {
            Some("v10-compile")
        } else if member.starts_with("V10.monitor.") {
            Some("v10-monitor")
        } else if v9_replay::Member::parse(member).is_some() {
            Some("criterion")
        } else {
            None
        }
    })
}

fn cargo_summaries(raw: &str, minimum_total: u64, expected_summaries: usize) -> bool {
    let lines: Vec<_> = raw
        .lines()
        .filter(|line| line.starts_with("test result: "))
        .collect();
    if lines.len() != expected_summaries {
        return false;
    }
    let mut total = 0_u64;
    for line in lines {
        let Some(rest) = line.strip_prefix("test result: ok. ") else {
            return false;
        };
        let words: Vec<_> = rest.split_whitespace().collect();
        if words.len() < 11
            || words[1..11]
                != [
                    "passed;",
                    "0",
                    "failed;",
                    "0",
                    "ignored;",
                    "0",
                    "measured;",
                    "0",
                    "filtered",
                    "out;",
                ]
        {
            return false;
        }
        let Ok(passed) = words[0].parse::<u64>() else {
            return false;
        };
        let Some(next) = total.checked_add(passed) else {
            return false;
        };
        total = next;
    }
    total >= minimum_total
}

fn marker(raw: &str, prefix: &str) -> Option<Value> {
    let markers: Vec<_> = raw
        .lines()
        .filter_map(|line| line.split_once(prefix).map(|(_, value)| value))
        .collect();
    if markers.len() != 1 {
        return None;
    }
    serde_json::from_str(markers[0]).ok()
}

fn population(parser: &str, raw: &str) -> bool {
    let (prefix, schema, scope, declared) = match parser {
        "finite-partition" => (
            "TL_CAMPAIGN_POPULATION ",
            "tl-mltl.finite-partition/v1",
            "depth1_atom1_closed0_2_words1_3",
            12_750_u64,
        ),
        "full-domain" => (
            "TL_CAMPAIGN_FULL_DOMAIN ",
            "tl-mltl.full-domain-census/v1",
            "depth3_atom1_closed0_4_words1_6_with_depth1_partition",
            1_344_017_183_059_893_186,
        ),
        "lasso-partition" => (
            "TL_CAMPAIGN_V11_POPULATION ",
            "tl-mltl.v11-lasso-partition/v1",
            "formulas30_words372_fair3_anchors4",
            133_920,
        ),
        _ => return false,
    };
    let Some(value) = marker(raw, prefix) else {
        return false;
    };
    if value["schema"] != schema
        || value["scope"] != scope
        || value["declared"].as_u64() != Some(declared)
        || value["failed"].as_u64() != Some(0)
        || value["full_target_complete"] != false
    {
        return false;
    }
    let (Some(visited), Some(refused)) = (value["visited"].as_u64(), value["refused"].as_u64())
    else {
        return false;
    };
    let unvisited = if parser == "full-domain" {
        value["unvisited"].as_u64()
    } else {
        Some(0)
    };
    let Some(unvisited) = unvisited else {
        return false;
    };
    if visited == 0
        || visited
            .checked_add(refused)
            .and_then(|n| n.checked_add(unvisited))
            != Some(declared)
    {
        return false;
    }
    match parser {
        "finite-partition" => {
            visited == 12_750
                && refused == 0
                && value["formulas"] == 375
                && value["word_positions"] == 34
                && value["max_depth"] == 1
                && value["interval_max"] == 2
                && value["trace_max_len"] == 3
        }
        "full-domain" => {
            visited == 518_094
                && refused == 0
                && unvisited == 1_344_017_183_059_375_092
                && value["completed_partition"]["visited"] == 518_094
                && value["completed_partition"]["formulas"] == 807
                && value["completed_partition"]["word_positions"] == 642
        }
        "lasso-partition" => {
            visited == 108_720
                && refused == 25_200
                && value["formula_count"] == 30
                && value["word_count"] == 372
                && value["fairness_modes"] == 3
                && value["anchors"] == serde_json::json!([0, 1, 3, 6])
        }
        _ => false,
    }
}

const V3_CRITERIA: &[&str] = &[
    "FR-027-AC-1",
    "FR-027-AC-2",
    "FR-027-AC-3",
    "FR-028-AC-1",
    "FR-028-AC-2",
    "FR-028-AC-3",
    "FR-029-AC-1",
    "FR-029-AC-2",
    "FR-029-AC-3",
    "FR-030-AC-1",
    "FR-030-AC-2",
    "FR-030-AC-3",
    "FR-031-AC-1",
    "FR-031-AC-2",
    "FR-031-AC-3",
    "FR-032-AC-1",
    "FR-032-AC-2",
    "FR-032-AC-3",
    "FR-033-AC-1",
    "FR-033-AC-2",
    "FR-033-AC-3",
    "FR-034-AC-1",
    "FR-034-AC-2",
    "FR-034-AC-3",
    "FR-038-AC-1",
    "FR-038-AC-2",
    "FR-038-AC-3",
    "FR-039-AC-1",
    "FR-039-AC-2",
    "FR-040-AC-1",
    "FR-040-AC-2",
    "FR-040-AC-3",
    "FR-041-AC-1",
    "FR-041-AC-2",
    "FR-042-AC-1",
    "FR-042-AC-2",
    "FR-043-AC-1",
    "FR-043-AC-2",
    "FR-044-AC-1",
    "FR-044-AC-2",
    "FR-045-AC-1",
    "FR-045-AC-2",
    "FR-046-AC-1",
    "FR-046-AC-2",
    "FR-047-AC-1",
    "FR-047-AC-2",
    "FR-048-AC-1",
    "FR-048-AC-2",
    "FR-049-AC-1",
    "FR-049-AC-2",
    "FR-050-AC-1",
    "FR-051-AC-1",
    "FR-052-AC-1",
    "FR-052-AC-2",
    "FR-053-AC-1",
    "FR-053-AC-2",
    "FR-054-AC-1",
    "FR-054-AC-2",
    "FR-055-AC-1",
    "FR-055-AC-2",
    "FR-055-AC-3",
];

fn semantic_properties(raw: &str) -> bool {
    let Some(value) = marker(raw, "TL_CAMPAIGN_PROPERTIES ") else {
        return false;
    };
    if value["schema"] != "tl-mltl.semantic-properties/v1"
        || value["scope"] != "tl_mltl_v1_semantic_laws_and_owner_wires"
        || value["seed_hex"] != "45".repeat(32)
        || value["generated"] != 64
        || value["accepted"] != 64
        || value["rejected"] != 0
        || value["wire_checks"] != 24
        || value["rewrite_equivalence_owner"] != "tl-rewrite"
        || !raw.contains("native_semantic_laws_and_strict_round_trips ... TL_CAMPAIGN_PROPERTIES")
        || !raw.contains("seeded_law_fault_is_detected ... ok")
    {
        return false;
    }
    let Some(cases) = value["law_cases"].as_object() else {
        return false;
    };
    let laws = [
        "bounded_embedding",
        "duality",
        "fairness_weakening",
        "finite_prefix_refutation",
        "lasso_unrolling",
        "partial_information_monotonicity",
    ];
    if cases.len() != laws.len()
        || laws
            .iter()
            .any(|law| cases.get(*law).and_then(Value::as_u64) != Some(64))
    {
        return false;
    }
    let Some(classes) = value["classifications"].as_object() else {
        return false;
    };
    if classes.len() != V3_CRITERIA.len() || V3_CRITERIA.iter().any(|id| !classes.contains_key(*id))
    {
        return false;
    }
    let mut properties = 0;
    let mut examples = 0;
    for class in classes.values() {
        let (Some(kind), Some(evidence)) = (class["kind"].as_str(), class["evidence"].as_str())
        else {
            return false;
        };
        if evidence.is_empty() {
            return false;
        }
        match kind {
            "property" if laws.contains(&evidence) || evidence == "strict_round_trips" => {
                properties += 1
            }
            "example" if raw.contains(&format!("{evidence} ... ok")) => examples += 1,
            "excluded" if evidence.contains(':') => (),
            _ => return false,
        }
    }
    properties > 0 && examples > 0
}

fn libfuzzer(member: &str, stdout: &[u8], stderr: &[u8]) -> bool {
    let target = match member {
        "V4.syntax_infinite_wire_decode" => "infinite_wire_decode",
        "V4.parse_unbounded_roundtrip" => "unbounded_parse_roundtrip",
        "V4.rewrite_infinite_rewrite" => "infinite_rewrite",
        "V4.mltl_c2po_map" => "c2po_map",
        "V4.mltl_closed_eval" => "closed_eval",
        _ => return false,
    };
    let raw = String::from_utf8_lossy(stdout).to_string() + &String::from_utf8_lossy(stderr);
    raw.contains("INFO: Seed: 181")
        && raw
            .lines()
            .filter(|line| line.starts_with("#1000\tDONE "))
            .count()
            == 1
        && raw.contains(&format!("/{target} "))
        && !raw.contains("ERROR: libFuzzer")
        && !raw.contains("SUMMARY: AddressSanitizer")
}

fn native_v7(member: &str, raw: &str) -> bool {
    if member_parser(member) == Some("embedded-build") {
        return raw.contains("Finished `dev` profile") || raw.contains("Finished `test` profile");
    }
    let test = match member {
        "V7.syntax_formula_limits" => {
            "strict_unbounded_reader_honors_lowered_node_and_depth_limits"
        }
        "V7.syntax_borrowed_ownership" => {
            "borrowed_and_owned_catalogs_round_trip_with_distinct_identities"
        }
        "V7.parse_limits_utf8" => {
            "parse_and_format_limits_are_exact_and_hostile_utf8_never_unwinds"
        }
        "V7.mltl_lasso_limits" => "every_lasso_resource_dimension_refuses_one_over_without_panic",
        "V7.mltl_prefix_limits" => "prefix_resource_dimensions_refuse_one_over_without_panic",
        "V7.rewrite_budgets" => "iteration_application_and_work_budgets_fail_closed",
        "V7.rewrite_record_limits" => {
            "tc_053_report_replay_and_all_work_limits_are_exact_and_fail_closed"
        }
        "V7.oracle_limits" => "tc_188_oracle_limit_edges_are_typed",
        _ => return false,
    };
    let summaries: Vec<_> = raw
        .lines()
        .filter(|line| line.starts_with("test result: "))
        .collect();
    raw.contains("running 1 test")
        && raw.contains(&format!("test {test} ... ok"))
        && summaries.len() == 1
        && summaries[0].starts_with("test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; ")
        && summaries[0].contains(" filtered out;")
}

fn kani_clean(member: &str, raw: &str) -> bool {
    let harness = match member {
        "V6.syntax_interval_proof" => {
            "formula::graph::kani_proofs::interval_cardinality_matches_wide_arithmetic"
        }
        "V6.mltl_horizon_proof" => {
            "future::horizon::kani_proofs::horizon_bound_addition_matches_checked_add"
        }
        _ => return false,
    };
    let mut checks = 0_u64;
    let mut assertions = 0_u64;
    let mut statuses = 0_u64;
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix("Check ") {
            let Some((number, name)) = rest.split_once(": ") else {
                return false;
            };
            checks += 1;
            if number.parse::<u64>().ok() != Some(checks) {
                return false;
            }
            if name.contains(harness) && name.contains(".assertion.") {
                assertions += 1;
            }
        } else if line.trim_start().starts_with("- Status: ") {
            if line.trim() != "- Status: SUCCESS" {
                return false;
            }
            statuses += 1;
        }
    }
    checks > 0
        && statuses == checks
        && assertions > 0
        && raw.contains(&format!("Checking harness {harness}..."))
        && raw.contains(&format!(" ** 0 of {checks} failed"))
        && raw.contains("Solving with CaDiCaL ")
        && raw.contains("CBMC version ")
        && raw.contains("VERIFICATION:- SUCCESSFUL")
        && raw.contains("Complete - 1 successfully verified harnesses, 0 failures, 1 total.")
}

fn kani_false(raw: &str) -> bool {
    let expected = ["vec![0, 0, 0, 128]", "vec![0, 0, 0, 192]"];
    let Some(playback) =
        raw.split_once("Concrete playback unit test for `seeded_false_cardinality_claim`:")
    else {
        return false;
    };
    let mut bytes = Vec::new();
    for line in playback.1.lines() {
        let line = line.trim();
        let Some(inner) = line
            .strip_prefix("vec![")
            .and_then(|s| s.strip_suffix("],"))
        else {
            continue;
        };
        let values: Option<Vec<u8>> = inner
            .split(',')
            .map(|number| number.trim().parse::<u8>().ok())
            .collect();
        let Some(values) = values else {
            return false;
        };
        bytes.push(values);
    }
    let expected_start = u32::from_le_bytes([0, 0, 0, 128]);
    let expected_end = u32::from_le_bytes([0, 0, 0, 192]);
    let replay = tl_syntax::Interval::new(expected_start, expected_end)
        .ok()
        .and_then(|interval| interval.cardinality())
        == Some(1_073_741_825);
    bytes == vec![vec![0, 0, 0, 128], vec![0, 0, 0, 192]]
        && expected.iter().all(|token| raw.contains(token))
        && replay
        && raw.contains("Checking harness seeded_false_cardinality_claim...")
        && raw.lines().any(|line| {
            line.trim()
                .strip_prefix("** 1 of ")
                .and_then(|rest| rest.strip_suffix(" failed"))
                .and_then(|count| count.parse::<u64>().ok())
                .is_some_and(|count| count > 0)
        })
        && raw.contains("assertion failed: interval.cardinality() == Some(1)")
        && raw.contains("Solving with CaDiCaL ")
        && raw.contains("VERIFICATION:- FAILED")
        && raw.contains("Complete - 0 successfully verified harnesses, 1 failures, 1 total.")
}

fn coverage_required_files(member: &str) -> Option<&'static [&'static str]> {
    match member {
        "V8.syntax_core" | "V8.syntax_alloc" => Some(&["src/future.rs", "src/formula/infinite.rs"]),
        "V8.syntax_serde" => Some(&[
            "src/future.rs",
            "src/formula/infinite.rs",
            "src/contracts/reader.rs",
        ]),
        "V8.parse_default" => Some(&[
            "src/parser.rs",
            "src/formatter.rs",
            "src/dialect/v4.rs",
            "src/infinite.rs",
            "src/lexer.rs",
        ]),
        "V8.mltl_default" => Some(&[
            "src/future/evaluate.rs",
            "src/past/mod.rs",
            "src/mapping/past.rs",
            "src/wire/command.rs",
            "src/wire/common.rs",
            "src/wire/trace.rs",
        ]),
        "V8.mltl_infinite" => Some(&[
            "src/future/evaluate.rs",
            "src/past/mod.rs",
            "src/infinite/periodic.rs",
            "src/infinite/export.rs",
            "src/mapping/past.rs",
            "src/wire/command.rs",
            "src/wire/common.rs",
            "src/wire/trace.rs",
        ]),
        "V8.rewrite_default" => Some(&[
            "src/engine/future.rs",
            "src/engine/past.rs",
            "src/report.rs",
            "src/replay.rs",
            "src/disposition.rs",
        ]),
        "V8.rewrite_infinite" => Some(&[
            "src/engine/future.rs",
            "src/engine/past.rs",
            "src/infinite.rs",
            "src/report.rs",
            "src/replay.rs",
            "src/disposition.rs",
        ]),
        _ => None,
    }
}

fn coverage_accept(member: &str, bytes: &[u8]) -> bool {
    let Ok(export) = serde_json::from_slice::<Value>(bytes) else {
        return false;
    };
    if export["type"] != "llvm.coverage.json.export" {
        return false;
    }
    let Some(data) = export["data"].as_array() else {
        return false;
    };
    if data.len() != 1 {
        return false;
    }
    let Some(files) = data[0]["files"].as_array() else {
        return false;
    };
    let Some(required) = coverage_required_files(member) else {
        return false;
    };
    let mut measured = BTreeMap::new();
    for file in files {
        let Some(filename) = file["filename"].as_str() else {
            return false;
        };
        let relative = filename
            .rsplit_once("/src/")
            .map(|(_, tail)| format!("src/{tail}"));
        let Some(relative) = relative else { continue };
        if measured.insert(relative, file).is_some() {
            return false;
        }
    }
    for path in required {
        let Some(file) = measured.get(*path) else {
            return false;
        };
        let (Some(branch_count), Some(branch_covered)) = (
            file["summary"]["branches"]["count"].as_u64(),
            file["summary"]["branches"]["covered"].as_u64(),
        ) else {
            return false;
        };
        if branch_covered > branch_count {
            return false;
        }
        if branch_count == 0 {
            if !matches!(*path, "src/dialect/v4.rs" | "src/disposition.rs")
                || ["lines", "functions", "regions"].iter().any(|metric| {
                    file["summary"][metric]["covered"]
                        .as_u64()
                        .is_none_or(|value| value == 0)
                })
            {
                return false;
            }
            continue;
        }
        if branch_covered != branch_count {
            return false;
        }
        let Some(branches) = file["branches"].as_array() else {
            return false;
        };
        if branches.is_empty() {
            return false;
        }
        let mut sites: BTreeMap<Vec<i64>, (u64, u64)> = BTreeMap::new();
        for branch in branches {
            let Some(values) = branch.as_array() else {
                return false;
            };
            if values.len() != 9 {
                return false;
            }
            let numbers: Option<Vec<i64>> = values.iter().map(Value::as_i64).collect();
            let Some(numbers) = numbers else { return false };
            if numbers.iter().any(|value| *value < 0) {
                return false;
            }
            let site = [numbers[..4].to_vec(), numbers[6..].to_vec()].concat();
            let entry = sites.entry(site).or_default();
            let (Ok(true_count), Ok(false_count)) =
                (u64::try_from(numbers[4]), u64::try_from(numbers[5]))
            else {
                return false;
            };
            let (Some(next_true), Some(next_false)) = (
                entry.0.checked_add(true_count),
                entry.1.checked_add(false_count),
            ) else {
                return false;
            };
            *entry = (next_true, next_false);
        }
        if sites
            .values()
            .any(|(true_count, false_count)| *true_count == 0 || *false_count == 0)
        {
            return false;
        }
    }
    true
}

fn v5_selection(member: &str) -> Option<(&'static str, &'static str)> {
    match member.split('.').nth(1)?.split('_').next()? {
        "parse" => Some(("src/infinite.rs", "parse_clean_ascii_v4|Parser.*::interval|Parser.*::lower_derived")),
        "syntax" => Some(("src/formula/infinite.rs", "TemporalInterval::start|select_infinite_profile|validate_resource_limits|preflight_resource_limits|InfiniteFormulaDocument::content_identity")),
        "mltl" => Some(("src/infinite/mod.rs", "evaluate_trace|evaluate_lasso")),
        "rewrite" => Some(("src/infinite.rs", "check_infinite_rewrite|classify_infinite_results")),
        _ => None,
    }
}

fn discovered_names(member: &str, raw: &[u8]) -> Option<(Vec<String>, Vec<String>)> {
    let discovered: Value = serde_json::from_slice(raw).ok()?;
    let rows = discovered.as_array()?;
    let (file, pattern) = v5_selection(member)?;
    let selector = Regex::new(pattern).ok()?;
    let mut all = Vec::new();
    let mut selected = Vec::new();
    for row in rows {
        let name = row["name"].as_str()?;
        let row_file = row["file"].as_str()?;
        if name.is_empty() || row_file != file || all.iter().any(|seen| seen == name) {
            return None;
        }
        all.push(name.to_owned());
        if selector.is_match(name) {
            selected.push(name.to_owned());
        }
    }
    if all.is_empty() || selected.is_empty() {
        return None;
    }
    Some((all, selected))
}

fn mutants_restored(raw: &str) -> bool {
    let count = raw
        .lines()
        .filter(|line| line.starts_with("test result: "))
        .count();
    count > 0 && cargo_summaries(raw, 1, count)
}

fn v5_tail(member: &str) -> Option<&'static [&'static str]> {
    match member.split('.').nth(1)?.split('_').next()? {
        "parse" => Some(&[
            "--test",
            "infinite_v4",
            "--test",
            "infinite_trace_corpus",
            "--test",
            "owner_infinite_corpus",
        ]),
        "syntax" => Some(&[
            "--test",
            "infinite_formula",
            "--test",
            "infinite_trace",
            "--test",
            "infinite_trace_corpus",
        ]),
        "mltl" => Some(&[
            "--lib",
            "--test",
            "infinite_trace",
            "--test",
            "infinite_oracle",
        ]),
        "rewrite" => Some(&[
            "--lib",
            "--test",
            "infinite_conformance",
            "--test",
            "infinite_rules",
            "--test",
            "infinite_owner_corpus",
        ]),
        _ => None,
    }
}

fn failed_status(status: &Value) -> bool {
    status["Failure"].as_i64().is_some_and(|code| code != 0)
}

fn mutation_accept(input: &CheckInput, bundle: &RawBundle) -> bool {
    let prefix = input.member.split('_').next().unwrap_or("");
    let discovery_member = format!("{prefix}_discovery");
    let Some(dependency) = input
        .dependencies
        .iter()
        .find(|row| row.member == discovery_member)
    else {
        return false;
    };
    let Ok(dependency_bytes) = fs::read(&dependency.result_path) else {
        return false;
    };
    let Ok(discovery_result) = serde_json::from_slice::<ExecutionResult>(&dependency_bytes) else {
        return false;
    };
    if discovery_result.state.kind != "completed" {
        return false;
    }
    let Some(discovery_process) = discovery_result.process else {
        return false;
    };
    if discovery_process
        .terminal_status
        .as_ref()
        .is_none_or(|status| status.kind != "exit_code" || status.value != 0)
        || discovery_process.stdout.truncated
        || sha256(&discovery_process.stdout.bytes) != discovery_process.stdout.digest
    {
        return false;
    }
    let Some((_, expected_selected)) =
        discovered_names(&discovery_member, &discovery_process.stdout.bytes)
    else {
        return false;
    };
    let artifact = |suffix: &str| -> Option<Value> {
        let role = format!("mutants/mutants.out/{suffix}");
        serde_json::from_slice(raw_bytes(bundle, &role)?).ok()
    };
    let Some(selected) = artifact("mutants.json") else {
        return false;
    };
    let Some(selected_rows) = selected.as_array() else {
        return false;
    };
    let names: Option<Vec<&str>> = selected_rows
        .iter()
        .map(|row| row["name"].as_str())
        .collect();
    let Some(names) = names else { return false };
    if names.len() != expected_selected.len()
        || names
            .iter()
            .any(|name| !expected_selected.iter().any(|expected| expected == name))
        || names
            .iter()
            .any(|name| names.iter().filter(|other| *other == name).count() != 1)
    {
        return false;
    }
    let Some(native) = artifact("outcomes.json") else {
        return false;
    };
    if native["cargo_mutants_version"] != "27.0.0"
        || native["total_mutants"].as_u64() != Some(names.len() as u64)
    {
        return false;
    }
    let Some(outcomes) = native["outcomes"].as_array() else {
        return false;
    };
    if outcomes.len() != names.len() + 1
        || outcomes[0]["scenario"] != "Baseline"
        || outcomes[0]["summary"] != "Success"
    {
        return false;
    }
    let Some(baseline_phases) = outcomes[0]["phase_results"].as_array() else {
        return false;
    };
    if baseline_phases.len() != 2
        || baseline_phases[0]["phase"] != "Build"
        || baseline_phases[1]["phase"] != "Test"
        || baseline_phases
            .iter()
            .any(|phase| phase["process_status"] != "Success")
    {
        return false;
    }
    let Some(test_argv) = baseline_phases[1]["argv"].as_array() else {
        return false;
    };
    let Some(tail) = v5_tail(&input.member) else {
        return false;
    };
    if test_argv.len() < tail.len()
        || test_argv[test_argv.len() - tail.len()..]
            .iter()
            .zip(tail.iter())
            .any(|(actual, expected)| actual != expected)
    {
        return false;
    }
    let mut caught = 0_u64;
    let mut unviable = 0_u64;
    let mut observed_names = BTreeSet::new();
    for outcome in outcomes.iter().skip(1) {
        let Some(name) = outcome["scenario"]["Mutant"]["name"].as_str() else {
            return false;
        };
        if !names.contains(&name) || !observed_names.insert(name) {
            return false;
        }
        let Some(phases) = outcome["phase_results"].as_array() else {
            return false;
        };
        let valid = match outcome["summary"].as_str() {
            Some("CaughtMutant") => {
                caught += 1;
                phases.len() == 2
                    && phases[0]["phase"] == "Build"
                    && phases[0]["process_status"] == "Success"
                    && phases[1]["phase"] == "Test"
                    && failed_status(&phases[1]["process_status"])
                    && phases[0]["argv"] == baseline_phases[0]["argv"]
                    && phases[1]["argv"] == baseline_phases[1]["argv"]
            }
            Some("Unviable") => {
                unviable += 1;
                phases.len() == 1
                    && phases[0]["phase"] == "Build"
                    && failed_status(&phases[0]["process_status"])
                    && phases[0]["argv"] == baseline_phases[0]["argv"]
            }
            _ => false,
        };
        if !valid {
            return false;
        }
        for key in ["log_path", "diff_path"] {
            if let Some(relative) = outcome[key].as_str() {
                let role = format!("mutants/mutants.out/{relative}");
                if !input.raw_artifacts.iter().any(|item| item.role == role) {
                    return false;
                }
            }
        }
    }
    caught > 0
        && native["caught"].as_u64() == Some(caught)
        && native["unviable"].as_u64() == Some(unviable)
        && native["missed"].as_u64() == Some(0)
        && native["timeout"].as_u64() == Some(0)
}

fn v10_target_rows(raw: &[u8]) -> Option<BTreeMap<(usize, usize), bool>> {
    let text = std::str::from_utf8(raw).ok()?;
    let mut rows = BTreeMap::new();
    for line in text.lines() {
        let (identity, value) = line.split_once(',')?;
        let (formula_text, position_text) = identity.split_once(':')?;
        let formula = formula_text.parse::<usize>().ok()?;
        let position = position_text.parse::<usize>().ok()?;
        if formula.to_string() != formula_text || position.to_string() != position_text {
            return None;
        }
        let verdict = match value {
            "T" => true,
            "F" => false,
            _ => return None,
        };
        if rows.insert((formula, position), verdict).is_some() {
            return None;
        }
    }
    (!rows.is_empty()).then_some(rows)
}

fn v10_inputs(input: &CheckInput, bundle: &RawBundle) -> bool {
    let manifests: Vec<_> = input
        .raw_artifacts
        .iter()
        .filter(|artifact| artifact.role == "manifest")
        .collect();
    let Some(manifest_artifact) = manifests.first() else {
        return false;
    };
    if manifests.len() != 1 {
        return false;
    }
    let Some(bytes) = raw_bytes(bundle, &manifest_artifact.role) else {
        return false;
    };
    let Ok(manifest) = serde_json::from_slice::<Value>(bytes) else {
        return false;
    };
    if manifest["schema"] != "tl-mltl.v10-input-manifest/v1"
        || manifest["formulaTraceCases"] != 225
        || manifest["perStepCells"] != 1350
    {
        return false;
    }
    let Some(cases) = manifest["cases"].as_array() else {
        return false;
    };
    if cases.len() != 22 || input.raw_artifacts.len() != 45 {
        return false;
    }
    let mut names = BTreeSet::new();
    let mut cells = 0_u64;
    for (index, case) in cases.iter().enumerate() {
        let Some(id) = case["id"].as_str() else {
            return false;
        };
        if !names.insert(id)
            || !id
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
        {
            return false;
        }
        let Some(formulas) = case["formulas"].as_u64() else {
            return false;
        };
        let Some(positions) = case["tracePositions"].as_u64() else {
            return false;
        };
        if index < 21 {
            if !matches!((formulas, positions), (12, 6) | (3, 6)) {
                return false;
            }
            cells += formulas * positions;
        } else if id != "safety" || (formulas, positions) != (1, 2) {
            return false;
        }
        for (extension, key) in [("c2po", "specDigest"), ("csv", "traceDigest")] {
            let role = format!("inputs/{id}.{extension}");
            let matching: Vec<_> = input
                .raw_artifacts
                .iter()
                .filter(|item| item.role == role)
                .collect();
            if matching.len() != 1
                || case[key].as_str() != Some(matching[0].digest.as_str())
                || raw_bytes(bundle, &role).is_none()
            {
                return false;
            }
        }
    }
    cells == 1350
}

fn selected_input_digests(request: &Value) -> Option<BTreeMap<String, String>> {
    let mut selected = BTreeMap::new();
    for input in request["inputs"].as_array()? {
        let role = input["role"].as_str()?;
        if role.starts_with("source/") || role.starts_with("source-exec/") {
            continue;
        }
        let path = input["path"].as_str()?;
        let digest = input["digest"].as_str()?;
        if path.is_empty()
            || digest.len() != 64
            || selected
                .insert(role.to_owned(), digest.to_owned())
                .is_some()
        {
            return None;
        }
    }
    Some(selected)
}

fn dependency_artifact_digest(input: &CheckInput, name: &str, role: &str) -> Option<String> {
    let mut matches = input
        .dependencies
        .iter()
        .filter(|dependency| dependency.member == name);
    let dependency = matches.next()?;
    if matches.next().is_some() || dependency.index != 1 {
        return None;
    }
    let bundle =
        sealed_raw_bundle(&dependency.raw_bundle_path, &dependency.raw_bundle_digest).ok()?;
    let artifact = bundle
        .artifacts
        .iter()
        .find(|artifact| artifact.role == role)?;
    let result: Value = serde_json::from_slice(&fs::read(&dependency.result_path).ok()?).ok()?;
    if result["artifacts"]
        .as_array()?
        .iter()
        .filter(|item| item["role"] == role && item["digest"] == artifact.digest)
        .count()
        != 1
    {
        return None;
    }
    Some(artifact.digest.clone())
}

fn v10_compile_inputs(input: &CheckInput, request: &Value, source_root: &Path) -> bool {
    let Some(case) = input.member.strip_prefix("V10.compile.") else {
        return false;
    };
    let expected: &[(&str, &str, &str)] = match case {
        "bounded" => &[
            ("spec", "v10/input.c2po", "corpus/r2u2-v4.2/formulas.c2po"),
            ("map", "v10/input.map", "corpus/r2u2-v4.2/signals.map"),
        ],
        "past" => &[
            (
                "spec",
                "v10/input.c2po",
                "corpus/past-c2po-v1/target-4.2/past.c2po",
            ),
            (
                "trace",
                "v10/input.csv",
                "corpus/past-c2po-v1/target-4.2/trace.csv",
            ),
        ],
        "unsafe-since" => &[
            (
                "spec",
                "v10/input.c2po",
                "corpus/past-c2po-v1/target-4.2/unsafe-since.c2po",
            ),
            (
                "trace",
                "v10/input.csv",
                "corpus/past-c2po-v1/target-4.2/unsafe-since.csv",
            ),
        ],
        _ if case
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-') =>
        {
            &[
                ("spec", "v10/input.c2po", ""),
                ("trace", "v10/input.csv", ""),
            ]
        }
        _ => return false,
    };
    let Some(selected) = selected_input_digests(request) else {
        return false;
    };
    if selected.len() != expected.len() {
        return false;
    }
    for (role, path, tracked) in expected {
        let observed = request["inputs"]
            .as_array()
            .and_then(|items| items.iter().find(|item| item["role"] == *role));
        let Some(observed) = observed else {
            return false;
        };
        if observed["path"] != *path {
            return false;
        }
        let digest = if tracked.is_empty() {
            let extension = if *role == "spec" { "c2po" } else { "csv" };
            dependency_artifact_digest(input, "V10.inputs", &format!("inputs/{case}.{extension}"))
        } else {
            fs::read(source_root.join(tracked))
                .ok()
                .map(|bytes| sha256(&bytes))
        };
        if digest.as_deref() != selected.get(*role).map(String::as_str) {
            return false;
        }
    }
    true
}

fn v10_monitor_inputs(input: &CheckInput, request: &Value, source_root: &Path) -> bool {
    let Some(case) = input.member.strip_prefix("V10.monitor.") else {
        return false;
    };
    if case.is_empty()
        || !case
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'-')
    {
        return false;
    }
    let Some(selected) = selected_input_digests(request) else {
        return false;
    };
    if selected.len() != 2
        || request["inputs"].as_array().is_none_or(|items| {
            !items
                .iter()
                .any(|item| item["role"] == "binary" && item["path"] == "v10/compiled.bin")
                || !items
                    .iter()
                    .any(|item| item["role"] == "trace" && item["path"] == "v10/trace.csv")
        })
    {
        return false;
    }
    if dependency_artifact_digest(input, &format!("V10.compile.{case}"), "binary").as_deref()
        != selected.get("binary").map(String::as_str)
    {
        return false;
    }
    let trace_digest = match case {
        "bounded" => fs::read(source_root.join("corpus/r2u2-v4.2/trace.csv"))
            .ok()
            .map(|bytes| sha256(&bytes)),
        "past" => fs::read(source_root.join("corpus/past-c2po-v1/target-4.2/trace.csv"))
            .ok()
            .map(|bytes| sha256(&bytes)),
        "unsafe-since" => {
            fs::read(source_root.join("corpus/past-c2po-v1/target-4.2/unsafe-since.csv"))
                .ok()
                .map(|bytes| sha256(&bytes))
        }
        _ => dependency_artifact_digest(input, "V10.inputs", &format!("inputs/{case}.csv")),
    };
    trace_digest.as_deref() == selected.get("trace").map(String::as_str)
}

fn v10_reviewed_inputs(input: &CheckInput, case: &str, run: &v10_replay::Run) -> bool {
    let expected_spec = sha256(&run.spec);
    let expected_trace = sha256(&run.trace);
    dependency_artifact_digest(input, "V10.inputs", &format!("inputs/{case}.c2po")).as_deref()
        == Some(expected_spec.as_str())
        && dependency_artifact_digest(input, "V10.inputs", &format!("inputs/{case}.csv")).as_deref()
            == Some(expected_trace.as_str())
}

fn v10_static_provenance(input: &CheckInput, case: &str, source_root: &Path) -> bool {
    let Some(expected) = v10_static::inputs(case) else {
        return false;
    };
    let mut matches = input
        .dependencies
        .iter()
        .filter(|dependency| dependency.member == format!("V10.compile.{case}"));
    let Some(compile) = matches.next() else {
        return false;
    };
    if matches.next().is_some() || compile.index != 1 {
        return false;
    }
    let Ok(bytes) = fs::read(&compile.request_path) else {
        return false;
    };
    let Ok(request) = serde_json::from_slice::<Value>(&bytes) else {
        return false;
    };
    let Some(selected) = selected_input_digests(&request) else {
        return false;
    };
    let expected_roles: &[(&str, &str, &[u8])] = if let Some(map) = expected.map {
        &[
            ("spec", "v10/input.c2po", expected.spec),
            ("map", "v10/input.map", map),
        ]
    } else {
        &[
            ("spec", "v10/input.c2po", expected.spec),
            ("trace", "v10/input.csv", expected.trace),
        ]
    };
    if selected.len() != expected_roles.len() {
        return false;
    }
    for (role, path, expected_bytes) in expected_roles {
        if request["inputs"].as_array().is_none_or(|inputs| {
            inputs
                .iter()
                .filter(|item| item["role"] == *role && item["path"] == *path)
                .count()
                != 1
        }) || selected.get(*role).map(String::as_str) != Some(sha256(expected_bytes).as_str())
        {
            return false;
        }
    }
    let tracked: &[(&str, &[u8])] = match case {
        "bounded" => &[
            ("corpus/r2u2-v4.2/formulas.c2po", expected.spec),
            ("corpus/r2u2-v4.2/signals.map", expected.map.unwrap()),
            ("corpus/r2u2-v4.2/trace.csv", expected.trace),
        ],
        "past" => &[
            ("corpus/past-c2po-v1/target-4.2/past.c2po", expected.spec),
            ("corpus/past-c2po-v1/target-4.2/trace.csv", expected.trace),
        ],
        "unsafe-since" => &[
            (
                "corpus/past-c2po-v1/target-4.2/unsafe-since.c2po",
                expected.spec,
            ),
            (
                "corpus/past-c2po-v1/target-4.2/unsafe-since.csv",
                expected.trace,
            ),
        ],
        "safety" => &[],
        _ => return false,
    };
    if tracked.iter().any(|(path, expected_bytes)| {
        fs::read(source_root.join(path)).ok().as_deref() != Some(*expected_bytes)
    }) {
        return false;
    }
    case != "safety"
        || [
            ("inputs/safety.c2po", expected.spec),
            ("inputs/safety.csv", expected.trace),
        ]
        .iter()
        .all(|(role, bytes)| {
            dependency_artifact_digest(input, "V10.inputs", role).as_deref()
                == Some(sha256(bytes).as_str())
        })
}

fn v8_parse_example_input(input: &CheckInput, request: &Value) -> bool {
    let Some(selected) = selected_input_digests(request) else {
        return false;
    };
    selected.len() == 1
        && request["inputs"].as_array().is_some_and(|items| {
            items.iter().any(|item| {
                item["role"] == "example"
                    && item["path"] == ".quoin-target/debug/examples/fuzz_campaign"
                    && item["executable"] == true
            })
        })
        && dependency_artifact_digest(input, "V8.parse_example_prep", "example").as_deref()
            == selected.get("example").map(String::as_str)
}

fn check(member: &str, definition_digest: &str, result_bytes: &[u8]) -> DomainVerdict {
    let mut reasons = Vec::new();
    let mut request_digest = String::new();
    let mut stdout_digest = None;
    let mut stderr_digest = None;
    let verdict = match member_parser(member) {
        None => {
            reasons.push("member_checker_unimplemented".into());
            "inconclusive"
        }
        Some(parser) => match serde_json::from_slice::<ExecutionResult>(result_bytes) {
            Err(_) => {
                reasons.push("malformed_execution_result".into());
                "reject"
            }
            Ok(result) => {
                request_digest = result.request_identity.digest;
                if result.protocol != RESULT_PROTOCOL {
                    reasons.push("wrong_execution_protocol".into());
                }
                if result.state.kind != "completed" {
                    reasons.push(format!("execution_{}", result.state.kind));
                }
                if let Some(process) = result.process {
                    stdout_digest = Some(process.stdout.digest.clone());
                    stderr_digest = Some(process.stderr.digest.clone());
                    if process.stdout.truncated
                        || process.stderr.truncated
                        || sha256(&process.stdout.bytes) != process.stdout.digest
                        || sha256(&process.stderr.bytes) != process.stderr.digest
                    {
                        reasons.push("raw_capture_mismatch".into());
                    }
                    if process.terminal_status.as_ref().is_none_or(|s| {
                        s.kind != "exit_code"
                            || if parser == "kani-false" {
                                s.value == 0
                            } else {
                                s.value != 0
                            }
                    }) {
                        reasons.push("native_command_failed".into());
                    }
                    if parser == "libfuzzer" {
                        if !libfuzzer(member, &process.stdout.bytes, &process.stderr.bytes) {
                            reasons.push("libfuzzer_budget_or_clean_exit_unproved".into());
                        }
                    } else if parser == "embedded-build" || parser == "miri" {
                        let raw = String::from_utf8_lossy(&process.stdout.bytes).to_string()
                            + &String::from_utf8_lossy(&process.stderr.bytes);
                        if !native_v7(member, &raw) {
                            reasons.push("native_v7_probe_unproved".into());
                        }
                    } else if parser == "kani-clean" {
                        let raw = String::from_utf8_lossy(&process.stdout.bytes).to_string()
                            + &String::from_utf8_lossy(&process.stderr.bytes);
                        if !kani_clean(member, &raw) {
                            reasons.push("kani_proof_unproved".into());
                        }
                    } else if parser == "kani-false" {
                        let raw = String::from_utf8_lossy(&process.stdout.bytes).to_string()
                            + &String::from_utf8_lossy(&process.stderr.bytes);
                        if !kani_false(&raw) {
                            reasons.push("kani_false_control_unproved".into());
                        }
                    } else if parser == "kani-replay" {
                        let raw = String::from_utf8_lossy(&process.stdout.bytes).to_string()
                            + &String::from_utf8_lossy(&process.stderr.bytes);
                        if !raw
                            .contains("test seeded_false_cardinality_counterexample_replays ... ok")
                            || !cargo_summaries(&raw, 1, 1)
                        {
                            reasons.push("kani_counterexample_replay_unproved".into());
                        }
                    } else if parser == "llvm-cov" {
                        // The decisive export is a required retained output artifact.
                    } else if parser == "mutants-discovery" {
                        if discovered_names(member, &process.stdout.bytes).is_none() {
                            reasons.push("mutant_discovery_unproved".into());
                        }
                    } else if parser == "mutants-run" {
                        // The decisive outcome ledger is a bounded output tree.
                    } else if parser == "mutants-restored" {
                        let raw = String::from_utf8_lossy(&process.stdout.bytes).to_string()
                            + &String::from_utf8_lossy(&process.stderr.bytes);
                        if !mutants_restored(&raw) {
                            reasons.push("mutant_restoration_unproved".into());
                        }
                    } else if parser == "v10-monitor" {
                        if v10_target_rows(&process.stdout.bytes).is_none() {
                            reasons.push("malformed_r2u2_target_rows".into());
                        }
                        if member.strip_prefix("V10.monitor.").is_none_or(|case| {
                            v10_replay::Run::for_member(case).is_none()
                                && v10_static::inputs(case).is_none()
                        }) {
                            reasons.push("v10_semantic_replay_pending".into());
                        }
                    } else if parser == "criterion" {
                        // Sealed Criterion bytes and host context are replayed in run_args.
                    } else if parser == "v10-inputs" || parser == "v10-compile" {
                        // Decisive bytes are required EA output artifacts checked below.
                    } else {
                        match String::from_utf8(process.stdout.bytes) {
                            Err(_) => reasons.push("non_utf8_native_output".into()),
                            Ok(raw) => {
                                let (minimum, summaries) = match parser {
                                    "finite-partition" => (2, 1),
                                    "full-domain" => (3, 1),
                                    "semantic-properties" => (3, 3),
                                    _ => (1, 1),
                                };
                                if !cargo_summaries(&raw, minimum, summaries)
                                    || (parser == "semantic-properties"
                                        && !semantic_properties(&raw))
                                    || (parser != "cargo-test"
                                        && parser != "semantic-properties"
                                        && !population(parser, &raw))
                                {
                                    reasons.push("native_result_unproved".into());
                                }
                            }
                        }
                    }
                } else {
                    reasons.push("missing_process_evidence".into());
                }
                if reasons.is_empty() {
                    "accept"
                } else if parser == "v10-monitor"
                    || matches!(
                        result.state.kind.as_str(),
                        "unavailable"
                            | "refused"
                            | "timed_out"
                            | "containment_failure"
                            | "cancelled"
                    )
                {
                    "inconclusive"
                } else {
                    "reject"
                }
            }
        },
    };
    DomainVerdict {
        schema: "tl-mltl.domain-verdict/v1",
        member: member.into(),
        verdict,
        reasons,
        definition_digest: definition_digest.into(),
        plan_id: String::new(),
        definition_version: String::new(),
        source_graph_digest: String::new(),
        request_digest,
        result_digest: String::new(),
        raw_artifacts_digest: String::new(),
        raw_bundle_digest: String::new(),
        dependencies_digest: String::new(),
        stdout_digest,
        stderr_digest,
    }
}

fn argument(args: &[String], name: &str) -> Result<String, String> {
    let index = args
        .iter()
        .position(|arg| arg == name)
        .ok_or(format!("missing {name}"))?;
    args.get(index + 1)
        .cloned()
        .ok_or(format!("missing value for {name}"))
}

fn run_args(args: &[String]) -> Result<(), String> {
    let input_path = argument(args, "--input")?;
    let input_bytes = fs::read(Path::new(&input_path)).map_err(|e| e.to_string())?;
    let input: CheckInput = serde_json::from_slice(&input_bytes).map_err(|e| e.to_string())?;
    if input.schema != "quoin.domain-check-input/v1" {
        return Err("unsupported checker input schema".into());
    }
    let definition_bytes =
        fs::read(Path::new(&input.definition_path)).map_err(|e| e.to_string())?;
    let definition: Value = serde_json::from_slice(&definition_bytes).map_err(|e| e.to_string())?;
    let member = definition["members"].as_array().and_then(|members| {
        let mut matches = members.iter().filter(|entry| entry["name"] == input.member);
        let first = matches.next()?;
        matches.next().is_none().then_some(first)
    });
    if definition["schemaVersion"] != "engineering-assurance.campaign-definition/v1"
        || member.is_none_or(|entry| entry["required"] != true)
    {
        return Err("campaign definition does not require exactly one named member".into());
    }
    let member = member.ok_or("missing named campaign member")?;
    let output_path = argument(args, "--output")?;
    let bytes = fs::read(Path::new(&input.result_path)).map_err(|e| e.to_string())?;
    let result_value: Value = serde_json::from_slice(&bytes).map_err(|e| e.to_string())?;
    let request_bytes = fs::read(Path::new(&input.request_path)).map_err(|e| e.to_string())?;
    let request_value: Value = serde_json::from_slice(&request_bytes).map_err(|e| e.to_string())?;
    let mut verdict = check(&input.member, &input.definition_digest, &bytes);
    verdict.plan_id = input.plan_id.clone();
    verdict.definition_version = input.definition_version.clone();
    verdict.source_graph_digest = input.source_graph_digest.clone();
    verdict.request_digest = input.request_digest.clone();
    verdict.result_digest = canonical_digest(&result_value)?;
    let artifact_inventory =
        serde_json::to_value(&input.raw_artifacts).map_err(|e| e.to_string())?;
    verdict.raw_artifacts_digest = canonical_digest(&artifact_inventory)?;
    let raw_bundle_bytes = fs::read(&input.raw_bundle_path).map_err(|e| e.to_string())?;
    let raw_bundle_value: Value =
        serde_json::from_slice(&raw_bundle_bytes).map_err(|e| e.to_string())?;
    let raw_bundle: RawBundle =
        serde_json::from_value(raw_bundle_value.clone()).map_err(|e| e.to_string())?;
    verdict.raw_bundle_digest = canonical_digest(&raw_bundle_value)?;
    let dependency_inventory =
        serde_json::to_value(&input.dependencies).map_err(|e| e.to_string())?;
    verdict.dependencies_digest = canonical_digest(&dependency_inventory)?;
    let mut binding_reasons = Vec::new();
    if canonical_digest(&definition)? != input.definition_digest {
        binding_reasons.push("definition_digest_mismatch".into());
    }
    if canonical_digest(&definition["sourceGraph"])? != input.source_graph_digest {
        binding_reasons.push("source_graph_digest_mismatch".into());
    }
    if member["planId"] != input.plan_id || member["definitionVersion"] != input.definition_version
    {
        binding_reasons.push("member_plan_binding_mismatch".into());
    }
    if result_value["requestIdentity"]["digest"] != input.request_digest {
        binding_reasons.push("request_digest_mismatch".into());
    }
    if canonical_digest(&request_value)? != input.request_digest
        || request_value["protocol"] != "engineering-assurance.producer-execution-request/v1"
        || request_value["producer"] != result_value["producer"]
        || definition["sourceGraph"].as_array().is_none_or(|sources| {
            !sources
                .iter()
                .any(|source| source["revision"] == request_value["producer"]["sourceRevision"])
        })
    {
        binding_reasons.push("request_bytes_or_producer_mismatch".into());
    }
    if verdict.result_digest != input.result_digest {
        binding_reasons.push("result_digest_mismatch".into());
    }
    if raw_bundle.schema != "quoin.raw-artifact-bundle/v1"
        || verdict.raw_bundle_digest != input.raw_bundle_digest
    {
        binding_reasons.push("raw_bundle_identity_mismatch".into());
    }
    let expected_dependencies: Vec<_> = member["dependsOn"]
        .as_array()
        .map(|items| items.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    if expected_dependencies.len() != input.dependencies.len()
        || input.dependencies.windows(2).any(|pair| {
            (pair[0].member.as_str(), pair[0].index) >= (pair[1].member.as_str(), pair[1].index)
        })
        || input.dependencies.iter().any(|dependency| {
            dependency.index != 1
                || expected_dependencies
                    .iter()
                    .filter(|name| **name == dependency.member)
                    .count()
                    != 1
        })
    {
        binding_reasons.push("dependency_inventory_mismatch".into());
    }
    for dependency in &input.dependencies {
        let dependency_bytes = fs::read(&dependency.result_path).map_err(|e| e.to_string())?;
        let dependency_request_bytes =
            fs::read(&dependency.request_path).map_err(|e| e.to_string())?;
        let dependency_value: Value =
            serde_json::from_slice(&dependency_bytes).map_err(|e| e.to_string())?;
        let dependency_request: Value =
            serde_json::from_slice(&dependency_request_bytes).map_err(|e| e.to_string())?;
        if canonical_digest(&dependency_value)? != dependency.result_digest
            || dependency_value["requestIdentity"]["digest"] != dependency.request_digest
            || canonical_digest(&dependency_request)? != dependency.request_digest
            || dependency_request["producer"] != dependency_value["producer"]
            || sealed_raw_bundle(&dependency.raw_bundle_path, &dependency.raw_bundle_digest)
                .is_err()
        {
            binding_reasons.push("dependency_result_mismatch".into());
        }
    }
    if input.raw_artifacts.len() != result_value["artifacts"].as_array().map_or(0, Vec::len)
        || input.raw_artifacts.len() != raw_bundle.artifacts.len()
    {
        binding_reasons.push("artifact_inventory_mismatch".into());
    } else {
        for (index, artifact) in input.raw_artifacts.iter().enumerate() {
            let bundled = &raw_bundle.artifacts[index];
            if bundled.role != artifact.role
                || bundled.digest != artifact.digest
                || sha256(&bundled.bytes) != artifact.digest
                || (index > 0 && raw_bundle.artifacts[index - 1].role >= bundled.role)
                || result_value["artifacts"].as_array().is_none_or(|items| {
                    items
                        .iter()
                        .filter(|item| {
                            item["role"] == artifact.role && item["digest"] == artifact.digest
                        })
                        .count()
                        != 1
                })
            {
                binding_reasons.push("artifact_digest_mismatch".into());
            }
        }
    }
    if member_parser(&input.member) == Some("libfuzzer") && !input.raw_artifacts.is_empty() {
        binding_reasons.push("libfuzzer_crash_artifact_present".into());
    }
    if member_parser(&input.member) == Some("llvm-cov") {
        let exports: Vec<_> = input
            .raw_artifacts
            .iter()
            .filter(|artifact| artifact.role == "coverage")
            .collect();
        if exports.len() != 1
            || raw_bytes(&raw_bundle, &exports[0].role)
                .is_none_or(|bytes| !coverage_accept(&input.member, bytes))
        {
            binding_reasons.push("critical_coverage_unproved".into());
        }
    }
    if member_parser(&input.member) == Some("mutants-run") && !mutation_accept(&input, &raw_bundle)
    {
        binding_reasons.push("mutant_outcome_population_unproved".into());
    }
    if member_parser(&input.member) == Some("v10-inputs") && !v10_inputs(&input, &raw_bundle) {
        binding_reasons.push("v10_input_census_unproved".into());
    }
    if member_parser(&input.member) == Some("v10-compile") {
        let binaries: Vec<_> = input
            .raw_artifacts
            .iter()
            .filter(|item| item.role == "binary")
            .collect();
        if binaries.len() != 1
            || raw_bytes(&raw_bundle, &binaries[0].role).is_none_or(|bytes| bytes.is_empty())
            || !v10_compile_inputs(&input, &request_value, Path::new("."))
        {
            binding_reasons.push("v10_compiled_binary_unproved".into());
        }
    }
    if member_parser(&input.member) == Some("v10-monitor")
        && !v10_monitor_inputs(&input, &request_value, Path::new("."))
    {
        binding_reasons.push("v10_monitor_inputs_unproved".into());
    }
    if member_parser(&input.member) == Some("v10-monitor") && verdict.verdict == "accept" {
        let Some(case) = input.member.strip_prefix("V10.monitor.") else {
            unreachable!("fixed monitor member prefix")
        };
        let generated = v10_replay::Run::for_member(case);
        let static_case = v10_static::inputs(case).is_some();
        let inputs_proved = generated
            .as_ref()
            .is_some_and(|run| v10_reviewed_inputs(&input, case, run))
            || (static_case && v10_static_provenance(&input, case, Path::new(".")));
        if !inputs_proved {
            verdict.verdict = "reject";
            verdict
                .reasons
                .push("v10_reviewed_input_bytes_unproved".into());
        } else {
            let result: ExecutionResult =
                serde_json::from_slice(&bytes).expect("accepted result was parsed by check");
            let stdout = &result
                .process
                .expect("accepted result has process")
                .stdout
                .bytes;
            let rows = v10_target_rows(stdout).expect("accepted target rows were parsed by check");
            let replay = if let Some(run) = generated {
                run.replay(&rows)
            } else {
                v10_static::replay(case, &rows)
            };
            match replay {
                v10_replay::Replay::Accept { .. } => {}
                v10_replay::Replay::Reject(reason) => {
                    verdict.verdict = "reject";
                    verdict.reasons.push(reason.into());
                }
                v10_replay::Replay::Inconclusive(reason) => {
                    verdict.verdict = "inconclusive";
                    verdict.reasons.push(reason.into());
                }
            }
        }
    }
    if member_parser(&input.member) == Some("criterion") && verdict.verdict == "accept" {
        match v9_replay::replay(
            &input.member,
            &definition,
            &request_value,
            &result_value,
            &raw_bundle,
            &input.dependencies,
        ) {
            v9_replay::Replay::Accept => {}
            v9_replay::Replay::Reject(reason) => {
                verdict.verdict = "reject";
                verdict.reasons.push(reason.into());
            }
            v9_replay::Replay::Inconclusive(reason) => {
                verdict.verdict = "inconclusive";
                verdict.reasons.push(reason.into());
            }
        }
    }
    if input.member == "V8.parse_default" && !v8_parse_example_input(&input, &request_value) {
        binding_reasons.push("v8_example_input_provenance_unproved".into());
    }
    if !binding_reasons.is_empty() {
        verdict.verdict = "reject";
        verdict.reasons.extend(binding_reasons);
    }
    let encoded = serde_json::to_vec_pretty(&verdict).map_err(|e| e.to_string())?;
    fs::write(output_path, encoded).map_err(|e| e.to_string())?;
    if verdict.verdict == "accept" {
        Ok(())
    } else {
        Err(verdict.reasons.join(", "))
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    run_args(&args)
}

fn main() {
    if let Err(error) = run() {
        eprintln!("tl_campaign_check: {error}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        cargo_summaries, check, member_parser, v10_compile_inputs, v10_monitor_inputs,
        v8_parse_example_input, CheckInput,
    };
    use serde_json::{json, Value};
    use std::fs;
    use std::path::Path;

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn v10_compile_refuses_substituted_tracked_spec_and_generated_input() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let corpus = root.join("corpus/r2u2-v4.2");
        fs::create_dir_all(&corpus).unwrap();
        fs::write(corpus.join("formulas.c2po"), b"pinned formula").unwrap();
        fs::write(corpus.join("signals.map"), b"pinned map").unwrap();
        let mut input: CheckInput = serde_json::from_value(json!({
            "schema":"quoin.domain-check-input/v1", "definitionPath":"definition.json",
            "definitionDigest":"a".repeat(64), "member":"V10.compile.bounded",
            "planId":"MP-117", "definitionVersion":"v1", "sourceGraphDigest":"b".repeat(64),
            "requestDigest":"c".repeat(64), "requestPath":"request.json",
            "resultPath":"result.json", "resultDigest":"d".repeat(64),
            "rawArtifacts":[], "rawBundlePath":"raw.json", "rawBundleDigest":"e".repeat(64)
        }))
        .unwrap();
        let mut request = json!({"inputs":[
            {"role":"spec","path":"v10/input.c2po","digest":super::sha256(b"pinned formula")},
            {"role":"map","path":"v10/input.map","digest":super::sha256(b"pinned map")}
        ]});
        assert!(v10_compile_inputs(&input, &request, root));
        request["inputs"][0]["digest"] = json!(super::sha256(b"substituted formula"));
        assert!(!v10_compile_inputs(&input, &request, root));

        input.member = "V10.compile.zero-upper-all-true".into();
        let raw = b"generated formula";
        let trace = b"time,signal\n0,1\n";
        let bundle = json!({"schema":"quoin.raw-artifact-bundle/v1","artifacts":[
            {"role":"inputs/zero-upper-all-true.c2po","digest":super::sha256(raw),"bytes":raw},
            {"role":"inputs/zero-upper-all-true.csv","digest":super::sha256(trace),"bytes":trace}
        ]});
        let bundle_path = root.join("dependency-raw.json");
        fs::write(&bundle_path, serde_json::to_vec(&bundle).unwrap()).unwrap();
        let result_path = root.join("dependency-result.json");
        fs::write(
            &result_path,
            serde_json::to_vec(&json!({"artifacts":[
                {"role":"inputs/zero-upper-all-true.c2po","digest":super::sha256(raw)},
                {"role":"inputs/zero-upper-all-true.csv","digest":super::sha256(trace)}
            ]}))
            .unwrap(),
        )
        .unwrap();
        input.dependencies = vec![serde_json::from_value(json!({
            "member":"V10.inputs", "index":1, "requestDigest":"f".repeat(64),
            "requestPath":"dependency-request.json", "resultDigest":"0".repeat(64),
            "resultPath":result_path, "rawBundleDigest":super::canonical_digest(&bundle).unwrap(),
            "rawBundlePath":bundle_path
        }))
        .unwrap()];
        let mut generated = json!({"inputs":[
            {"role":"spec","path":"v10/input.c2po","digest":super::sha256(raw)},
            {"role":"trace","path":"v10/input.csv","digest":super::sha256(trace)}
        ]});
        assert!(v10_compile_inputs(&input, &generated, root));
        generated["inputs"][0]["digest"] = json!(super::sha256(b"substituted formula"));
        assert!(!v10_compile_inputs(&input, &generated, root));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn v10_monitor_binds_binary_and_trace_to_exact_dependencies() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let binary = b"compiled monitor";
        let trace = b"# p,q\n1,0\n";
        let dependency = |member: &str, role: &str, bytes: &[u8]| -> super::DependencyResult {
            let bundle = json!({"schema":"quoin.raw-artifact-bundle/v1","artifacts":[
                {"role":role,"digest":super::sha256(bytes),"bytes":bytes}
            ]});
            let bundle_path = root.join(format!("{member}-raw.json"));
            let result_path = root.join(format!("{member}-result.json"));
            fs::write(&bundle_path, serde_json::to_vec(&bundle).unwrap()).unwrap();
            fs::write(
                &result_path,
                serde_json::to_vec(&json!({"artifacts":[
                    {"role":role,"digest":super::sha256(bytes)}
                ]}))
                .unwrap(),
            )
            .unwrap();
            serde_json::from_value(json!({
                "member":member,"index":1,"requestDigest":"f".repeat(64),
                "requestPath":"dependency-request.json","resultDigest":"0".repeat(64),
                "resultPath":result_path,
                "rawBundleDigest":super::canonical_digest(&bundle).unwrap(),
                "rawBundlePath":bundle_path
            }))
            .unwrap()
        };
        let mut input: CheckInput = serde_json::from_value(json!({
            "schema":"quoin.domain-check-input/v1", "definitionPath":"definition.json",
            "definitionDigest":"a".repeat(64),
            "member":"V10.monitor.zero-singleton-all-true",
            "planId":"MP-117", "definitionVersion":"v1",
            "sourceGraphDigest":"b".repeat(64), "requestDigest":"c".repeat(64),
            "requestPath":"request.json", "resultPath":"result.json",
            "resultDigest":"d".repeat(64), "rawArtifacts":[],
            "rawBundlePath":"raw.json", "rawBundleDigest":"e".repeat(64),
            "dependencies":[
                dependency("V10.compile.zero-singleton-all-true", "binary", binary),
                dependency("V10.inputs", "inputs/zero-singleton-all-true.csv", trace)
            ]
        }))
        .unwrap();
        let mut request = json!({"inputs":[
            {"role":"binary","path":"v10/compiled.bin","digest":super::sha256(binary)},
            {"role":"trace","path":"v10/trace.csv","digest":super::sha256(trace)}
        ]});
        assert!(v10_monitor_inputs(&input, &request, root));
        request["inputs"][0]["digest"] = json!(super::sha256(b"substitute"));
        assert!(!v10_monitor_inputs(&input, &request, root));
        request["inputs"][0]["digest"] = json!(super::sha256(binary));
        request["inputs"][1]["digest"] = json!(super::sha256(b"other trace"));
        assert!(!v10_monitor_inputs(&input, &request, root));
        request["inputs"][1]["digest"] = json!(super::sha256(trace));
        input.member = "V10.monitor.zero-singleton-all-false".into();
        assert!(!v10_monitor_inputs(&input, &request, root));

        let tracked = root.join("corpus/r2u2-v4.2");
        fs::create_dir_all(&tracked).unwrap();
        fs::write(tracked.join("trace.csv"), trace).unwrap();
        input.member = "V10.monitor.bounded".into();
        input.dependencies = vec![dependency("V10.compile.bounded", "binary", binary)];
        assert!(v10_monitor_inputs(&input, &request, root));
        fs::write(tracked.join("trace.csv"), b"changed trace").unwrap();
        assert!(!v10_monitor_inputs(&input, &request, root));
    }

    // Trace: FR-052-AC-1, FR-055-AC-2, TC-191, TC-198
    #[test]
    fn v10_replay_refuses_sealed_but_wrong_generated_spec_or_trace() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let case = "zero-unit-boundary-toggle";
        let run = super::v10_replay::Run::for_member(case).unwrap();
        let make_input = |spec: &[u8], trace: &[u8]| -> CheckInput {
            let spec_role = format!("inputs/{case}.c2po");
            let trace_role = format!("inputs/{case}.csv");
            let bundle = json!({"schema":"quoin.raw-artifact-bundle/v1","artifacts":[
                {"role":spec_role,"digest":super::sha256(spec),"bytes":spec},
                {"role":trace_role,"digest":super::sha256(trace),"bytes":trace}
            ]});
            let result = json!({"artifacts":[
                {"role":spec_role,"digest":super::sha256(spec)},
                {"role":trace_role,"digest":super::sha256(trace)}
            ]});
            let bundle_path = root.join("v10-input-bundle.json");
            let result_path = root.join("v10-input-result.json");
            fs::write(&bundle_path, serde_json::to_vec(&bundle).unwrap()).unwrap();
            fs::write(&result_path, serde_json::to_vec(&result).unwrap()).unwrap();
            serde_json::from_value(json!({
                "schema":"quoin.domain-check-input/v1", "definitionPath":"definition.json",
                "definitionDigest":"a".repeat(64),
                "member":format!("V10.monitor.{case}"),
                "planId":"MP-117", "definitionVersion":"v1",
                "sourceGraphDigest":"b".repeat(64), "requestDigest":"c".repeat(64),
                "requestPath":"request.json", "resultPath":"result.json",
                "resultDigest":"d".repeat(64), "rawArtifacts":[],
                "rawBundlePath":"raw.json", "rawBundleDigest":"e".repeat(64),
                "dependencies":[{
                    "member":"V10.inputs", "index":1,
                    "requestDigest":"f".repeat(64), "requestPath":"dependency-request.json",
                    "resultDigest":"0".repeat(64), "resultPath":result_path,
                    "rawBundleDigest":super::canonical_digest(&bundle).unwrap(),
                    "rawBundlePath":bundle_path
                }]
            }))
            .unwrap()
        };
        assert!(super::v10_reviewed_inputs(
            &make_input(&run.spec, &run.trace),
            case,
            &run
        ));
        assert!(!super::v10_reviewed_inputs(
            &make_input(b"different C2PO spec", &run.trace),
            case,
            &run
        ));
        assert!(!super::v10_reviewed_inputs(
            &make_input(&run.spec, b"# p,q\n0,0\n"),
            case,
            &run
        ));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn v10_static_provenance_refuses_substituted_compile_request_and_source_bytes() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let expected = super::v10_static::inputs("bounded").unwrap();
        let corpus = root.join("corpus/r2u2-v4.2");
        fs::create_dir_all(&corpus).unwrap();
        fs::write(corpus.join("formulas.c2po"), expected.spec).unwrap();
        fs::write(corpus.join("signals.map"), expected.map.unwrap()).unwrap();
        fs::write(corpus.join("trace.csv"), expected.trace).unwrap();
        let request_path = root.join("compile-request.json");
        let mut request = json!({"inputs":[
            {"role":"spec","path":"v10/input.c2po","digest":super::sha256(expected.spec)},
            {"role":"map","path":"v10/input.map","digest":super::sha256(expected.map.unwrap())}
        ]});
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        let input: CheckInput = serde_json::from_value(json!({
            "schema":"quoin.domain-check-input/v1", "definitionPath":"definition.json",
            "definitionDigest":"a".repeat(64), "member":"V10.monitor.bounded",
            "planId":"MP-117", "definitionVersion":"v1", "sourceGraphDigest":"b".repeat(64),
            "requestDigest":"c".repeat(64), "requestPath":"request.json",
            "resultDigest":"d".repeat(64), "resultPath":"result.json",
            "rawArtifacts":[], "rawBundlePath":"raw.json", "rawBundleDigest":"e".repeat(64),
            "dependencies":[{
                "member":"V10.compile.bounded", "index":1,
                "requestDigest":"f".repeat(64), "requestPath":request_path,
                "resultDigest":"0".repeat(64), "resultPath":"compile-result.json",
                "rawBundleDigest":"1".repeat(64), "rawBundlePath":"compile-raw.json"
            }]
        }))
        .unwrap();
        assert!(super::v10_static_provenance(&input, "bounded", root));
        request["inputs"][0]["digest"] = json!(super::sha256(b"wrong but sealed spec"));
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        assert!(!super::v10_static_provenance(&input, "bounded", root));
        request["inputs"][0]["digest"] = json!(super::sha256(expected.spec));
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        fs::write(corpus.join("formulas.c2po"), b"wrong tracked formula bytes").unwrap();
        assert!(!super::v10_static_provenance(&input, "bounded", root));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn v10_safety_provenance_refuses_wrong_but_sealed_generated_inputs() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let expected = super::v10_static::inputs("safety").unwrap();
        let compile_request_path = root.join("compile-request.json");
        let compile_request = json!({"inputs":[
            {"role":"spec","path":"v10/input.c2po","digest":super::sha256(expected.spec)},
            {"role":"trace","path":"v10/input.csv","digest":super::sha256(expected.trace)}
        ]});
        fs::write(
            &compile_request_path,
            serde_json::to_vec(&compile_request).unwrap(),
        )
        .unwrap();
        let bundle_path = root.join("inputs-bundle.json");
        let result_path = root.join("inputs-result.json");
        let seal = |spec: &[u8], trace: &[u8]| -> (String, String) {
            let bundle = json!({"schema":"quoin.raw-artifact-bundle/v1","artifacts":[
                {"role":"inputs/safety.c2po","digest":super::sha256(spec),"bytes":spec},
                {"role":"inputs/safety.csv","digest":super::sha256(trace),"bytes":trace}
            ]});
            let result = json!({"artifacts":[
                {"role":"inputs/safety.c2po","digest":super::sha256(spec)},
                {"role":"inputs/safety.csv","digest":super::sha256(trace)}
            ]});
            fs::write(&bundle_path, serde_json::to_vec(&bundle).unwrap()).unwrap();
            fs::write(&result_path, serde_json::to_vec(&result).unwrap()).unwrap();
            (
                super::canonical_digest(&bundle).unwrap(),
                super::canonical_digest(&result).unwrap(),
            )
        };
        let (bundle_digest, result_digest) = seal(expected.spec, expected.trace);
        let mut input: CheckInput = serde_json::from_value(json!({
            "schema":"quoin.domain-check-input/v1", "definitionPath":"definition.json",
            "definitionDigest":"a".repeat(64), "member":"V10.monitor.safety",
            "planId":"MP-117", "definitionVersion":"v1", "sourceGraphDigest":"b".repeat(64),
            "requestDigest":"c".repeat(64), "requestPath":"request.json",
            "resultDigest":"d".repeat(64), "resultPath":"result.json",
            "rawArtifacts":[], "rawBundlePath":"raw.json", "rawBundleDigest":"e".repeat(64),
            "dependencies":[
                {"member":"V10.compile.safety", "index":1,
                 "requestDigest":"f".repeat(64), "requestPath":compile_request_path,
                 "resultDigest":"0".repeat(64), "resultPath":"compile-result.json",
                 "rawBundleDigest":"1".repeat(64), "rawBundlePath":"compile-raw.json"},
                {"member":"V10.inputs", "index":1,
                 "requestDigest":"f".repeat(64), "requestPath":"inputs-request.json",
                 "resultDigest":result_digest, "resultPath":result_path,
                 "rawBundleDigest":bundle_digest, "rawBundlePath":bundle_path}
            ]
        }))
        .unwrap();
        assert!(super::v10_static_provenance(&input, "safety", root));
        let (bundle_digest, result_digest) = seal(b"wrong but sealed safety spec", expected.trace);
        input.dependencies[1].raw_bundle_digest = bundle_digest;
        input.dependencies[1].result_digest = result_digest;
        assert!(!super::v10_static_provenance(&input, "safety", root));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn v8_coverage_refuses_substituted_example_executable() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        let binary = b"example executable";
        let raw_bundle = json!({"schema":"quoin.raw-artifact-bundle/v1","artifacts":[
            {"role":"example","digest":super::sha256(binary),"bytes":binary}
        ]});
        let raw_path = root.join("raw.json");
        let result_path = root.join("result.json");
        fs::write(&raw_path, serde_json::to_vec(&raw_bundle).unwrap()).unwrap();
        fs::write(
            &result_path,
            serde_json::to_vec(&json!({"artifacts":[
                {"role":"example","digest":super::sha256(binary)}
            ]}))
            .unwrap(),
        )
        .unwrap();
        let input: CheckInput = serde_json::from_value(json!({
            "schema":"quoin.domain-check-input/v1", "definitionPath":"definition.json",
            "definitionDigest":"a".repeat(64), "member":"V8.parse_default",
            "planId":"MP-081", "definitionVersion":"v1", "sourceGraphDigest":"b".repeat(64),
            "requestDigest":"c".repeat(64), "requestPath":"request.json",
            "resultPath":"result.json", "resultDigest":"d".repeat(64),
            "rawArtifacts":[], "rawBundlePath":"raw.json", "rawBundleDigest":"e".repeat(64),
            "dependencies":[{"member":"V8.parse_example_prep","index":1,
                "requestDigest":"f".repeat(64),"requestPath":"dep-request.json",
                "resultDigest":"0".repeat(64),"resultPath":result_path,
                "rawBundleDigest":super::canonical_digest(&raw_bundle).unwrap(),
                "rawBundlePath":raw_path}]
        }))
        .unwrap();
        let mut request = json!({"inputs":[{"role":"example",
            "path":".quoin-target/debug/examples/fuzz_campaign", "executable":true,
            "digest":super::sha256(binary)}]});
        assert!(v8_parse_example_input(&input, &request));
        request["inputs"][0]["digest"] = json!(super::sha256(b"different executable"));
        assert!(!v8_parse_example_input(&input, &request));
    }

    fn result(raw: &str, state: &str) -> Vec<u8> {
        let stdout = raw.as_bytes();
        serde_json::to_vec(&json!({
            "protocol": "engineering-assurance.producer-execution-result/v1",
            "requestIdentity": {"digest": "a".repeat(64)},
            "process": {
                "terminalStatus": {"kind": "exit_code", "value": 0},
                "stdout": {"bytes": stdout, "digest": super::sha256(stdout), "truncated": false},
                "stderr": {"bytes": [], "digest": super::sha256(&[]), "truncated": false}
            },
            "state": {"kind": state}
        }))
        .unwrap()
    }

    fn sealed_v10_monitor_receipt(
        root: &Path,
        case: &str,
        target_rows: &str,
        spec: &[u8],
        trace: &[u8],
    ) -> (Value, bool) {
        let member = format!("V10.monitor.{case}");
        let revision = "a".repeat(40);
        let producer = json!({"sourceRevision": revision});
        let binary = b"compiled monitor";
        let dependency = |name: &str, artifacts: Value, input_claims: Option<Value>| -> Value {
            let stem = name.replace('.', "-");
            let request_path = root.join(format!("{stem}-request.json"));
            let result_path = root.join(format!("{stem}-result.json"));
            let bundle_path = root.join(format!("{stem}-bundle.json"));
            let mut dependency_request = json!({
                "protocol":"engineering-assurance.producer-execution-request/v1",
                "producer":producer
            });
            if let Some(claims) = input_claims {
                dependency_request["inputs"] = claims;
            }
            fs::write(
                &request_path,
                serde_json::to_vec(&dependency_request).unwrap(),
            )
            .unwrap();
            let inventory = artifacts
                .as_array()
                .unwrap()
                .iter()
                .map(|artifact| json!({"role":artifact["role"],"digest":artifact["digest"]}))
                .collect::<Vec<_>>();
            let bundle = json!({
                "schema":"quoin.raw-artifact-bundle/v1", "artifacts":artifacts
            });
            fs::write(&bundle_path, serde_json::to_vec(&bundle).unwrap()).unwrap();
            let mut dependency_result: Value =
                serde_json::from_slice(&result("", "completed")).unwrap();
            dependency_result["producer"] = producer.clone();
            dependency_result["requestIdentity"]["digest"] =
                json!(super::canonical_digest(&dependency_request).unwrap());
            dependency_result["artifacts"] = json!(inventory);
            fs::write(
                &result_path,
                serde_json::to_vec(&dependency_result).unwrap(),
            )
            .unwrap();
            json!({
                "member":name, "index":1,
                "requestDigest":super::canonical_digest(&dependency_request).unwrap(),
                "requestPath":request_path,
                "resultDigest":super::canonical_digest(&dependency_result).unwrap(),
                "resultPath":result_path,
                "rawBundleDigest":super::canonical_digest(&bundle).unwrap(),
                "rawBundlePath":bundle_path
            })
        };
        let compile_inputs = super::v10_static::inputs(case).map(|static_inputs| {
            let second = if let Some(map) = static_inputs.map {
                ("map", "v10/input.map", map)
            } else {
                ("trace", "v10/input.csv", trace)
            };
            json!([
                {"role":"spec","path":"v10/input.c2po","digest":super::sha256(spec)},
                {"role":second.0,"path":second.1,"digest":super::sha256(second.2)}
            ])
        });
        let compile = dependency(
            &format!("V10.compile.{case}"),
            json!([{"role":"binary","digest":super::sha256(binary),"bytes":binary}]),
            compile_inputs,
        );
        let mut dependencies = vec![compile];
        let mut depends_on = vec![format!("V10.compile.{case}")];
        if case == "safety" || super::v10_static::inputs(case).is_none() {
            dependencies.push(dependency(
                "V10.inputs",
                json!([
                    {"role":format!("inputs/{case}.c2po"),"digest":super::sha256(spec),"bytes":spec},
                    {"role":format!("inputs/{case}.csv"),"digest":super::sha256(trace),"bytes":trace}
                ]),
                None,
            ));
            depends_on.push("V10.inputs".into());
        }
        let definition = json!({
            "schemaVersion":"engineering-assurance.campaign-definition/v1",
            "sourceGraph":[{"repository":"tl-mltl","revision":revision,"digest":"b".repeat(64)}],
            "members":[{
                "name":member,"planId":"MP-117", "definitionVersion":"tl.v10.v1-monitor/v1",
                "required":true,
                "dependsOn":depends_on
            }]
        });
        let definition_path = root.join("definition.json");
        fs::write(&definition_path, serde_json::to_vec(&definition).unwrap()).unwrap();
        let request = json!({
            "protocol":"engineering-assurance.producer-execution-request/v1",
            "producer":producer,
            "inputs":[
                {"role":"binary","path":"v10/compiled.bin","digest":super::sha256(binary)},
                {"role":"trace","path":"v10/trace.csv","digest":super::sha256(trace)}
            ]
        });
        let request_path = root.join("request.json");
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        let mut monitor_result: Value =
            serde_json::from_slice(&result(target_rows, "completed")).unwrap();
        monitor_result["producer"] = producer;
        monitor_result["requestIdentity"]["digest"] =
            json!(super::canonical_digest(&request).unwrap());
        monitor_result["artifacts"] = json!([]);
        let result_path = root.join("result.json");
        fs::write(&result_path, serde_json::to_vec(&monitor_result).unwrap()).unwrap();
        let bundle = json!({"schema":"quoin.raw-artifact-bundle/v1","artifacts":[]});
        let bundle_path = root.join("bundle.json");
        fs::write(&bundle_path, serde_json::to_vec(&bundle).unwrap()).unwrap();
        let input = json!({
            "schema":"quoin.domain-check-input/v1",
            "definitionPath":definition_path,
            "definitionDigest":super::canonical_digest(&definition).unwrap(),
            "member":member, "planId":"MP-117",
            "definitionVersion":"tl.v10.v1-monitor/v1",
            "sourceGraphDigest":super::canonical_digest(&definition["sourceGraph"]).unwrap(),
            "requestDigest":super::canonical_digest(&request).unwrap(),
            "requestPath":request_path,
            "resultDigest":super::canonical_digest(&monitor_result).unwrap(),
            "resultPath":result_path,
            "rawArtifacts":[],
            "rawBundlePath":bundle_path,
            "rawBundleDigest":super::canonical_digest(&bundle).unwrap(),
            "dependencies":dependencies
        });
        let input_path = root.join("checker-input.json");
        let output_path = root.join("checker-verdict.json");
        fs::write(&input_path, serde_json::to_vec(&input).unwrap()).unwrap();
        let args = vec![
            "--input".into(),
            input_path.display().to_string(),
            "--output".into(),
            output_path.display().to_string(),
        ];
        let accepted = super::run_args(&args).is_ok();
        let receipt = serde_json::from_slice(&fs::read(output_path).unwrap()).unwrap();
        (receipt, accepted)
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn sealed_v10_monitor_replay_emits_fixed_accept_reject_and_inconclusive_receipts() {
        let directory = tempfile::tempdir().unwrap();
        let run = super::v10_replay::Run::for_member("zero-singleton-all-true").unwrap();
        let complete = (0..12)
            .flat_map(|formula| (0..6).map(move |position| format!("{formula}:{position},T\n")))
            .collect::<String>();
        let (accepted, status) = sealed_v10_monitor_receipt(
            directory.path(),
            "zero-singleton-all-true",
            &complete,
            &run.spec,
            &run.trace,
        );
        assert!(status, "{accepted:#}");
        assert_eq!(accepted["schema"], "tl-mltl.domain-verdict/v1");
        assert_eq!(accepted["member"], "V10.monitor.zero-singleton-all-true");
        assert_eq!(accepted["verdict"], "accept");
        assert_eq!(accepted["reasons"], json!([]));
        assert_eq!(accepted["stdoutDigest"], super::sha256(complete.as_bytes()));

        let flipped = complete.replacen("0:0,T\n", "0:0,F\n", 1);
        let (rejected, status) = sealed_v10_monitor_receipt(
            directory.path(),
            "zero-singleton-all-true",
            &flipped,
            &run.spec,
            &run.trace,
        );
        assert!(!status);
        assert_eq!(rejected["verdict"], "reject");
        assert_eq!(rejected["reasons"], json!(["v10_target_semantic_mismatch"]));

        let missing = complete.replacen("0:0,T\n", "", 1);
        let (inconclusive, status) = sealed_v10_monitor_receipt(
            directory.path(),
            "zero-singleton-all-true",
            &missing,
            &run.spec,
            &run.trace,
        );
        assert!(!status);
        assert_eq!(inconclusive["verdict"], "inconclusive");
        assert_eq!(
            inconclusive["reasons"],
            json!(["v10_admitted_target_row_missing"])
        );

        for (spec, trace) in [
            (
                b"wrong but sealed C2PO spec".as_slice(),
                run.trace.as_slice(),
            ),
            (run.spec.as_slice(), b"# p,q\n0,0\n".as_slice()),
        ] {
            let (rejected, status) = sealed_v10_monitor_receipt(
                directory.path(),
                "zero-singleton-all-true",
                &complete,
                spec,
                trace,
            );
            assert!(!status);
            assert_eq!(rejected["verdict"], "reject");
            assert_eq!(
                rejected["reasons"],
                json!(["v10_reviewed_input_bytes_unproved"])
            );
        }
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn sealed_static_v10_monitors_preserve_accept_inconclusive_and_reject() {
        let directory = tempfile::tempdir().unwrap();
        let cases: [(&str, &[u8], &str, bool); 4] = [
            (
                "bounded",
                include_bytes!("../../corpus/r2u2-v4.2/r2u2.stdout"),
                "accept",
                true,
            ),
            (
                "past",
                include_bytes!("../../corpus/past-c2po-v1/target-4.2/r2u2.stdout"),
                "inconclusive",
                false,
            ),
            (
                "unsafe-since",
                include_bytes!("../../corpus/past-c2po-v1/target-4.2/unsafe-since.stdout"),
                "inconclusive",
                false,
            ),
            ("safety", b"0:0,F\n0:1,T\n", "accept", true),
        ];
        for (case, raw, expected_verdict, expected_status) in cases {
            let inputs = super::v10_static::inputs(case).unwrap();
            let target = std::str::from_utf8(raw).unwrap();
            let (receipt, status) = sealed_v10_monitor_receipt(
                directory.path(),
                case,
                target,
                inputs.spec,
                inputs.trace,
            );
            assert_eq!(status, expected_status, "{case}: {receipt:#}");
            assert_eq!(receipt["member"], format!("V10.monitor.{case}"));
            assert_eq!(receipt["verdict"], expected_verdict, "{case}: {receipt:#}");
            assert_eq!(receipt["stdoutDigest"], super::sha256(raw));
        }

        let bounded = super::v10_static::inputs("bounded").unwrap();
        let (receipt, status) = sealed_v10_monitor_receipt(
            directory.path(),
            "bounded",
            std::str::from_utf8(cases[0].1).unwrap(),
            b"wrong but sealed C2PO spec",
            bounded.trace,
        );
        assert!(!status);
        assert_eq!(receipt["verdict"], "reject");
        assert_eq!(
            receipt["reasons"],
            json!(["v10_reviewed_input_bytes_unproved"])
        );
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn sealed_checker_input_rejects_stale_definition_digest() {
        let directory = tempfile::tempdir().unwrap();
        let definition_path = directory.path().join("definition.json");
        let request_path = directory.path().join("request.json");
        let result_path = directory.path().join("result.json");
        let raw_bundle_path = directory.path().join("raw-bundle.json");
        let input_path = directory.path().join("input.json");
        let output_path = directory.path().join("verdict.json");
        let definition = json!({
            "schemaVersion":"engineering-assurance.campaign-definition/v1",
            "sourceGraph":[{"repository":"tl-mltl","revision":"a".repeat(40),"digest":"b".repeat(64)}],
            "members":[{"name":"V1.independent_oracle","planId":"MP-008","definitionVersion":"tl.v1.v1-independent-oracle/v1","required":true}]
        });
        fs::write(
            &definition_path,
            serde_json::to_vec_pretty(&definition).unwrap(),
        )
        .unwrap();
        let raw = "test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        let request = json!({
            "protocol":"engineering-assurance.producer-execution-request/v1",
            "producer":{"sourceRevision":"a".repeat(40)}
        });
        fs::write(&request_path, serde_json::to_vec(&request).unwrap()).unwrap();
        let mut result: Value = serde_json::from_slice(&result(raw, "completed")).unwrap();
        result["requestIdentity"]["digest"] = json!(super::canonical_digest(&request).unwrap());
        result["producer"] = request["producer"].clone();
        fs::write(&result_path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        let raw_bundle = json!({"schema":"quoin.raw-artifact-bundle/v1","artifacts":[]});
        fs::write(&raw_bundle_path, serde_json::to_vec(&raw_bundle).unwrap()).unwrap();
        let mut input = json!({
            "schema":"quoin.domain-check-input/v1",
            "definitionPath":definition_path,
            "definitionDigest":super::canonical_digest(&definition).unwrap(),
            "member":"V1.independent_oracle",
            "planId":"MP-008",
            "definitionVersion":"tl.v1.v1-independent-oracle/v1",
            "sourceGraphDigest":super::canonical_digest(&definition["sourceGraph"]).unwrap(),
            "requestDigest":super::canonical_digest(&request).unwrap(),
            "requestPath":request_path,
            "resultPath":result_path,
            "resultDigest":super::canonical_digest(&result).unwrap(),
            "rawArtifacts":[],
            "rawBundlePath":raw_bundle_path,
            "rawBundleDigest":super::canonical_digest(&raw_bundle).unwrap()
        });
        let args = vec![
            "--input".into(),
            input_path.display().to_string(),
            "--output".into(),
            output_path.display().to_string(),
        ];
        fs::write(&input_path, serde_json::to_vec(&input).unwrap()).unwrap();
        super::run_args(&args).unwrap();
        let accepted: Value = serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
        assert_eq!(accepted["verdict"], "accept");
        input["definitionDigest"] = json!("c".repeat(64));
        fs::write(&input_path, serde_json::to_vec(&input).unwrap()).unwrap();
        assert!(super::run_args(&args).is_err());
        let rejected: Value = serde_json::from_slice(&fs::read(&output_path).unwrap()).unwrap();
        assert_eq!(rejected["verdict"], "reject");
        assert!(rejected["reasons"]
            .as_array()
            .unwrap()
            .contains(&json!("definition_digest_mismatch")));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn kani_checker_rejects_missing_or_failed_property() {
        let raw = "Checking harness formula::graph::kani_proofs::interval_cardinality_matches_wide_arithmetic...\nCBMC version 6.11.0 (cbmc-6.11.0)\nSolving with CaDiCaL 3.0.0\nCheck 1: formula::graph::kani_proofs::interval_cardinality_matches_wide_arithmetic.assertion.1\n - Status: SUCCESS\n ** 0 of 1 failed\nVERIFICATION:- SUCCESSFUL\nComplete - 1 successfully verified harnesses, 0 failures, 1 total.\n";
        assert!(super::kani_clean("V6.syntax_interval_proof", raw));
        assert!(!super::kani_clean(
            "V6.syntax_interval_proof",
            &raw.replace("- Status: SUCCESS", "- Status: FAILURE")
        ));
        assert!(!super::kani_clean(
            "V6.syntax_interval_proof",
            &raw.replace("Check 1:", "Check 2:")
        ));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn false_kani_control_requires_replayable_bytes() {
        let raw = "Checking harness seeded_false_cardinality_claim...\nSolving with CaDiCaL 3.0.0\n ** 1 of 35 failed\nFailed Checks: assertion failed: interval.cardinality() == Some(1)\nVERIFICATION:- FAILED\nConcrete playback unit test for `seeded_false_cardinality_claim`:\nlet concrete_vals: Vec<Vec<u8>> = vec![\nvec![0, 0, 0, 128],\nvec![0, 0, 0, 192],\n];\nComplete - 0 successfully verified harnesses, 1 failures, 1 total.\n";
        assert!(super::kani_false(raw));
        assert!(!super::kani_false(
            &raw.replace("vec![0, 0, 0, 192]", "vec![0, 0, 0, 128]")
        ));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn coverage_checker_requires_both_critical_branch_sides() {
        let file = |name: &str, false_count| {
            json!({
                "filename":format!("/staged/src/{name}"),
                "summary":{"branches":{"count":2,"covered":if false_count == 0 {1} else {2}}},
                "branches":[[1,1,1,2,1,false_count,0,0,0]]
            })
        };
        let good = json!({"type":"llvm.coverage.json.export","data":[{"files":[
            file("future.rs",1),file("formula/infinite.rs",1)
        ]}]});
        assert!(super::coverage_accept(
            "V8.syntax_core",
            &serde_json::to_vec(&good).unwrap()
        ));
        let bad = json!({"type":"llvm.coverage.json.export","data":[{"files":[
            file("future.rs",0),file("formula/infinite.rs",1)
        ]}]});
        assert!(!super::coverage_accept(
            "V8.syntax_core",
            &serde_json::to_vec(&bad).unwrap()
        ));
    }

    // Trace: FR-052-AC-1, TC-191
    #[test]
    fn v10_target_parser_rejects_duplicate_and_unknown_rows() {
        assert_eq!(super::v10_target_rows(b"0:0,T\n0:1,F\n").unwrap().len(), 2);
        assert!(super::v10_target_rows(b"0:0,T\n0:0,F\n").is_none());
        assert!(super::v10_target_rows(b"0:0,unknown\n").is_none());
        assert!(super::v10_target_rows(b"0:0,T\n1:nope,F\n").is_none());
        for noncanonical in [
            b"00:0,T\n".as_slice(),
            b"+0:0,T\n",
            b"0:01,T\n",
            b"0:+1,T\n",
        ] {
            assert!(super::v10_target_rows(noncanonical).is_none());
        }
    }

    // Trace: FR-055-AC-1, TC-197
    #[test]
    fn fixed_member_inventory_refuses_unimplemented_lane() {
        assert_eq!(member_parser("V1.independent_oracle"), Some("cargo-test"));
        assert_eq!(member_parser("V2.full_domain_census"), Some("full-domain"));
        assert!(member_parser("V4.fuzz_replay").is_none());
        let verdict = check("V4.fuzz_replay", &"b".repeat(64), &result("", "completed"));
        assert_eq!(verdict.verdict, "inconclusive");
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn four_static_v10_monitors_have_a_semantic_replay_handler() {
        for case in ["bounded", "past", "unsafe-since", "safety"] {
            assert!(super::v10_static::inputs(case).is_some());
            let verdict = check(
                &format!("V10.monitor.{case}"),
                &"b".repeat(64),
                &result("0:0,T\n", "completed"),
            );
            assert_eq!(verdict.verdict, "accept", "{case}");
            assert!(
                !verdict
                    .reasons
                    .contains(&"v10_semantic_replay_pending".into()),
                "{case}: {:?}",
                verdict.reasons
            );
        }
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn checker_reads_raw_bytes_not_claimed_observation() {
        let good = "running 2 tests\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
        assert!(cargo_summaries(good, 1, 1));
        let accepted = check(
            "V1.independent_oracle",
            &"b".repeat(64),
            &result(good, "completed"),
        );
        assert_eq!(accepted.verdict, "accept");
        let false_claim = result("test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s", "completed");
        assert_eq!(
            check("V1.independent_oracle", &"b".repeat(64), &false_claim).verdict,
            "reject"
        );
        let mut tampered: Value = serde_json::from_slice(&result(good, "completed")).unwrap();
        tampered["process"]["stdout"]["bytes"] = json!([120]);
        assert_eq!(
            check(
                "V1.independent_oracle",
                &"b".repeat(64),
                &serde_json::to_vec(&tampered).unwrap()
            )
            .verdict,
            "reject"
        );
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn unavailable_execution_is_inconclusive() {
        let verdict = check(
            "V1.independent_oracle",
            &"b".repeat(64),
            &result("", "unavailable"),
        );
        assert_eq!(verdict.verdict, "inconclusive");
    }
}
