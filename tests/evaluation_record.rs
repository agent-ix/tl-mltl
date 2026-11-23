//! TC-187: the evaluation record is read from the request file, taken from the
//! evaluator, and held to the checked-in record schema.

use std::{collections::BTreeSet, env, path::PathBuf, process::Command};

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

fn json_type_of(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "integer",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn type_allows(property: &Value, value: &Value) -> bool {
    if let Some(constant) = property.get("const") {
        return constant == value;
    }
    if let Some(options) = property.get("enum") {
        return options.as_array().unwrap().contains(value);
    }
    if let Some(options) = property.get("oneOf") {
        return options
            .as_array()
            .unwrap()
            .iter()
            .any(|option| type_allows(option, value));
    }
    match &property["type"] {
        Value::String(name) => name == json_type_of(value),
        Value::Array(names) => names.iter().any(|name| name == json_type_of(value)),
        _ => false,
    }
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn every_record_value_satisfies_its_schema_property() {
    let schema = schema();
    for name in ["true", "false", "pending"] {
        let record = record_of(name);
        for (field, value) in record.as_object().unwrap() {
            let property = &schema["properties"][field];
            assert!(type_allows(property, value), "{name} {field}: {value}");
            if let Some(nested) = property.get("properties") {
                for (inner, inner_value) in value.as_object().unwrap() {
                    assert!(
                        type_allows(&nested[inner], inner_value),
                        "{name} {field}.{inner}"
                    );
                }
            }
        }
    }
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn schema_enums_name_every_verdict_and_profile_the_tool_emits() {
    let schema = schema();
    let enum_of = |field: &str| -> BTreeSet<String> {
        schema["properties"][field]["enum"]
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().to_owned())
            .collect()
    };
    let owned = |items: &[&str]| items.iter().map(|item| (*item).to_owned()).collect();
    assert_eq!(enum_of("verdict"), owned(&["True", "False", "Pending"]));
    assert_eq!(
        enum_of("semanticProfile"),
        owned(&["ClosedTraceV1", "OnlinePrefixV1"])
    );
    assert_eq!(
        enum_of("semanticProfileId"),
        owned(&["mltl.closed-trace/v1", "mltl.online-prefix/v1"])
    );
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn formula_interval_is_the_root_operator_interval() {
    for name in ["true", "false", "pending"] {
        let record = record_of(name);
        assert_eq!(record["formulaInterval"]["start"], 1, "{name}");
        assert_eq!(record["formulaInterval"]["end"], 2, "{name}");
    }
    let mut request: Value =
        serde_json::from_slice(&std::fs::read(fixture("true")).unwrap()).unwrap();
    request["formula"]["root"] = serde_json::json!(0);
    let record = build_record(&serde_json::to_vec(&request).unwrap()).unwrap();
    assert!(serde_json::to_value(record).unwrap()["formulaInterval"].is_null());
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn a_non_evaluate_request_is_refused() {
    let mut request: Value =
        serde_json::from_slice(&std::fs::read(fixture("true")).unwrap()).unwrap();
    request["operation"] = serde_json::json!("analyze");
    let error = build_record(&serde_json::to_vec(&request).unwrap()).unwrap_err();
    assert_eq!(error, "evaluation record requires operation evaluate");
}

fn run_example(arguments: &[&std::ffi::OsStr]) -> std::process::Output {
    Command::new(env::var_os("CARGO").unwrap())
        .args(["run", "--quiet", "--example", "evaluation_record", "--"])
        .args(arguments)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap()
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn the_example_writes_the_same_record_to_stdout_and_to_a_named_file() {
    let request = fixture("pending");
    let stdout = run_example(&[request.as_os_str()]);
    assert!(stdout.status.success());
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("record.json");
    let status = run_example(&[request.as_os_str(), "--out".as_ref(), path.as_os_str()]);
    assert!(status.status.success());
    let printed: Value = serde_json::from_slice(&stdout.stdout).unwrap();
    let written: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(printed, written);
    assert_eq!(printed, record_of("pending"));
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn a_refusal_exits_two_with_one_stderr_line_and_no_stdout() {
    let directory = tempfile::tempdir().unwrap();
    let missing = directory.path().join("absent.json");
    let output = run_example(&[missing.as_os_str()]);
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.starts_with("evaluation_record: "), "{stderr}");
}

fn corpus_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/r2u2-v4.2")
}

// Trace: TC-187, FR-005-AC-4
#[test]
fn corpus_records_are_regenerated_from_the_corpus_requests() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(corpus_dir().join("manifest.json")).unwrap())
            .unwrap();
    let mut seen = 0;
    for entry in std::fs::read_dir(corpus_dir().join("requests")).unwrap() {
        let path = entry.unwrap().path();
        let id = path.file_stem().unwrap().to_str().unwrap().to_owned();
        let bytes = std::fs::read(&path).unwrap();
        let request: Value = serde_json::from_slice(&bytes).unwrap();
        let case = manifest["cases"]
            .as_array()
            .unwrap()
            .iter()
            .find(|case| case["id"] == id.as_str())
            .unwrap_or_else(|| panic!("{id} is not a manifest case"));
        assert_eq!(request["formula"], case["formula"], "{id} formula drifted");
        assert_eq!(
            request["trace"]["instants"], manifest["trace"],
            "{id} trace drifted"
        );
        let expected = format!(
            "{}\n",
            serde_json::to_string_pretty(&build_record(&bytes).unwrap()).unwrap()
        );
        let committed =
            std::fs::read_to_string(corpus_dir().join("records").join(format!("{id}.json")))
                .unwrap();
        assert_eq!(
            committed, expected,
            "{id} record is stale: run make corpus-records"
        );
        seen += 1;
    }
    assert!(seen > 0);
}
