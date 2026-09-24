//! TL domain checker for one EA-executed Campaign member.
//!
//! This program never launches a producer. EA executes each declared native
//! command directly; Quoin retains that result and supplies its exact bytes.
//! Trace: FR-055-AC-1, FR-055-AC-2, TC-197, TC-198.

use std::{fs, path::Path};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};

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
    result_path: String,
    result_digest: String,
    raw_artifacts: Vec<RawArtifact>,
}

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RawArtifact {
    role: String,
    path: String,
    digest: String,
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
        _ => None,
    }
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
                    if process
                        .terminal_status
                        .as_ref()
                        .is_none_or(|s| s.kind != "exit_code" || s.value != 0)
                    {
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
                } else if matches!(
                    result.state.kind.as_str(),
                    "unavailable" | "refused" | "timed_out" | "containment_failure" | "cancelled"
                ) {
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
    let mut verdict = check(&input.member, &input.definition_digest, &bytes);
    verdict.plan_id = input.plan_id.clone();
    verdict.definition_version = input.definition_version.clone();
    verdict.source_graph_digest = input.source_graph_digest.clone();
    verdict.request_digest = input.request_digest.clone();
    verdict.result_digest = canonical_digest(&result_value)?;
    let artifact_inventory =
        serde_json::to_value(&input.raw_artifacts).map_err(|e| e.to_string())?;
    verdict.raw_artifacts_digest = canonical_digest(&artifact_inventory)?;
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
    if verdict.result_digest != input.result_digest {
        binding_reasons.push("result_digest_mismatch".into());
    }
    if input.raw_artifacts.len() != result_value["artifacts"].as_array().map_or(0, Vec::len) {
        binding_reasons.push("artifact_inventory_mismatch".into());
    } else {
        for artifact in &input.raw_artifacts {
            let artifact_bytes = fs::read(&artifact.path).map_err(|e| e.to_string())?;
            if sha256(&artifact_bytes) != artifact.digest
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
    use super::{cargo_summaries, check, member_parser};
    use serde_json::{json, Value};
    use std::fs;

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

    #[test]
    fn sealed_checker_input_rejects_stale_definition_digest() {
        let directory = tempfile::tempdir().unwrap();
        let definition_path = directory.path().join("definition.json");
        let result_path = directory.path().join("result.json");
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
        let result: Value = serde_json::from_slice(&result(raw, "completed")).unwrap();
        fs::write(&result_path, serde_json::to_vec_pretty(&result).unwrap()).unwrap();
        let mut input = json!({
            "schema":"quoin.domain-check-input/v1",
            "definitionPath":definition_path,
            "definitionDigest":super::canonical_digest(&definition).unwrap(),
            "member":"V1.independent_oracle",
            "planId":"MP-008",
            "definitionVersion":"tl.v1.v1-independent-oracle/v1",
            "sourceGraphDigest":super::canonical_digest(&definition["sourceGraph"]).unwrap(),
            "requestDigest":"a".repeat(64),
            "resultPath":result_path,
            "resultDigest":super::canonical_digest(&result).unwrap(),
            "rawArtifacts":[]
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

    #[test]
    fn fixed_member_inventory_refuses_unimplemented_lane() {
        assert_eq!(member_parser("V1.independent_oracle"), Some("cargo-test"));
        assert_eq!(member_parser("V2.full_domain_census"), Some("full-domain"));
        assert!(member_parser("V4.fuzz_replay").is_none());
        let verdict = check("V4.fuzz_replay", &"b".repeat(64), &result("", "completed"));
        assert_eq!(verdict.verdict, "inconclusive");
    }

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
