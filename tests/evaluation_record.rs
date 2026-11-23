//! TC-187: the evaluation record is read from the request file, taken from the
//! evaluator, and held to the checked-in record schema.

use std::{collections::BTreeSet, path::PathBuf, process::Command};

use serde_json::Value;
use tl_mltl::record::{build_record, RECORD_SCHEMA_BYTES};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/evaluation-records")
        .join(format!("{name}.json"))
}

fn record_of(name: &str) -> Value {
    let bytes = std::fs::read(fixture(name)).unwrap();
    serde_json::to_value(build_record(&bytes).unwrap()).unwrap()
}

fn keys(value: &Value) -> BTreeSet<String> {
    value.as_object().unwrap().keys().cloned().collect()
}

fn schema() -> Value {
    serde_json::from_slice(RECORD_SCHEMA_BYTES).unwrap()
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn verdicts_match_the_three_example_outcomes() {
    for (name, verdict, profile) in [
        ("true", "True", "ClosedTraceV1"),
        ("false", "False", "ClosedTraceV1"),
        ("pending", "Pending", "OnlinePrefixV1"),
    ] {
        let record = record_of(name);
        assert_eq!(record["verdict"], verdict, "{name}");
        assert_eq!(record["semanticProfile"], profile, "{name}");
        assert_eq!(record["toolchain"]["version"], env!("CARGO_PKG_VERSION"));
        assert_eq!(record["evaluationPosition"], 0);
        assert_eq!(record["window"]["start"], 0);
        assert_eq!(record["window"]["end"], 2);
    }
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn trace_and_identity_come_from_the_request_file() {
    for name in ["true", "false", "pending"] {
        let bytes = std::fs::read(fixture(name)).unwrap();
        let request: Value = serde_json::from_slice(&bytes).unwrap();
        let record = record_of(name);
        assert_eq!(record["trace"]["instants"], request["trace"]["instants"]);
        assert_eq!(record["trace"]["traceId"], request["trace"]["traceId"]);
        assert_eq!(record["trace"]["closed"], request["trace"]["closed"]);
        assert_eq!(record["formulaId"], request["formulaId"]);
        let digest = tl_mltl_sha256_hex(&bytes);
        assert_eq!(record["inputSha256"], digest);
    }
}

fn tl_mltl_sha256_hex(bytes: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn a_changed_trace_changes_the_verdict() {
    let mut request: Value =
        serde_json::from_slice(&std::fs::read(fixture("true")).unwrap()).unwrap();
    request["trace"]["instants"] = serde_json::json!([[], [], []]);
    let record = build_record(&serde_json::to_vec(&request).unwrap()).unwrap();
    assert_eq!(serde_json::to_value(record).unwrap()["verdict"], "False");
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn record_fields_equal_the_schema_fields() {
    let schema = schema();
    let declared = keys(&schema["properties"]);
    let required: BTreeSet<String> = schema["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| name.as_str().unwrap().to_owned())
        .collect();
    assert_eq!(declared, required);
    for name in ["true", "false", "pending"] {
        let record = record_of(name);
        assert_eq!(keys(&record), declared, "{name} top level");
        for nested in ["toolchain", "trace", "window"] {
            assert_eq!(
                keys(&record[nested]),
                keys(&schema["properties"][nested]["properties"]),
                "{name} {nested}"
            );
        }
        assert_eq!(
            record["schemaVersion"],
            schema["properties"]["schemaVersion"]["const"]
        );
    }
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn the_example_writes_the_same_record_to_stdout_and_to_a_named_file() {
    let example = PathBuf::from(env!("CARGO_BIN_EXE_tl-mltl"))
        .parent()
        .unwrap()
        .join("examples")
        .join(format!("evaluation_record{}", std::env::consts::EXE_SUFFIX));
    let request = fixture("pending");
    let stdout = Command::new(&example).arg(&request).output().unwrap();
    assert!(stdout.status.success());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("record.json");
    let status = Command::new(&example)
        .arg(&request)
        .arg("--out")
        .arg(&path)
        .status()
        .unwrap();
    assert!(status.success());
    let printed: Value = serde_json::from_slice(&stdout.stdout).unwrap();
    let written: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(printed, written);
    assert_eq!(printed, record_of("pending"));
}
