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

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DomainVerdict {
    schema: &'static str,
    member: String,
    verdict: &'static str,
    reasons: Vec<String>,
    definition_digest: String,
    request_digest: String,
    result_digest: String,
    stdout_digest: Option<String>,
    stderr_digest: Option<String>,
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
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
        "V11.lasso_population_census" => Some("lasso-partition"),
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
                    match String::from_utf8(process.stdout.bytes) {
                        Err(_) => reasons.push("non_utf8_native_output".into()),
                        Ok(raw) => {
                            let (minimum, summaries) = match parser {
                                "finite-partition" => (2, 1),
                                "full-domain" => (3, 1),
                                _ => (1, 1),
                            };
                            if !cargo_summaries(&raw, minimum, summaries)
                                || (parser != "cargo-test" && !population(parser, &raw))
                            {
                                reasons.push("native_result_unproved".into());
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
        request_digest,
        result_digest: sha256(result_bytes),
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

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let member = argument(&args, "--member")?;
    let definition_digest = argument(&args, "--definition-digest")?;
    if definition_digest.len() != 64
        || !definition_digest
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
    {
        return Err("definition digest must be lowercase SHA-256".into());
    }
    let result_path = argument(&args, "--result")?;
    let output_path = argument(&args, "--output")?;
    let bytes = fs::read(Path::new(&result_path)).map_err(|e| e.to_string())?;
    let verdict = check(&member, &definition_digest, &bytes);
    let encoded = serde_json::to_vec_pretty(&verdict).map_err(|e| e.to_string())?;
    fs::write(output_path, encoded).map_err(|e| e.to_string())?;
    if verdict.verdict == "accept" {
        Ok(())
    } else {
        Err(verdict.reasons.join(", "))
    }
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
