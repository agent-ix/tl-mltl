use std::{
    io::Write,
    process::{Command, Stdio},
};

use serde_json::{json, Value};

fn run(request: &Value) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tl-mltl"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(serde_json::to_string(request).unwrap().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn analyze_request() -> Value {
    json!({
        "schemaVersion": "tl-mltl.command/v1",
        "operation": "analyze",
        "formulaId": "future-zero",
        "formula": {
            "schema_version": "tl-syntax.formula/v1",
            "semantic_profile": "mltl.closed-trace/v1",
            "root": 1,
            "nodes": [
                {"kind": "proposition", "proposition": 0},
                {"kind": "future", "interval": {"start": 0, "end": 2}, "operand": 0}
            ]
        },
        "trace": null
    })
}

fn mapping_request() -> Value {
    json!({
        "schemaVersion": "tl-mltl.command/v1",
        "operation": "map_c2po",
        "formulaId": "future-zero",
        "formula": {
            "schema_version": "tl-syntax.formula/v1",
            "semantic_profile": "mltl.online-prefix/v1",
            "root": 1,
            "nodes": [
                {"kind": "proposition", "proposition": 0},
                {"kind": "future", "interval": {"start": 0, "end": 2}, "operand": 0}
            ]
        },
        "trace": null
    })
}

// Trace: TC-014, FR-005-AC-1, NFR-001-AC-1, NFR-002-AC-1, StR-002-VC-2
#[test]
fn cli_is_deterministic_and_rejects_unknown_command_schema() {
    let first = run(&analyze_request());
    let second = run(&analyze_request());
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(first.stdout, second.stdout);
    let report: Value = serde_json::from_slice(&first.stdout).unwrap();
    assert_eq!(report["schemaVersion"], "tl-mltl.horizon/v1");
    assert_eq!(report["lookahead"], 2);

    let mut invalid = analyze_request();
    invalid["schemaVersion"] = json!("tl-mltl.command/v2");
    let rejected = run(&invalid);
    assert_eq!(rejected.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("unknown variant"));

    let mapped = run(&mapping_request());
    assert!(
        mapped.status.success(),
        "{}",
        String::from_utf8_lossy(&mapped.stderr)
    );
    let manifest: Value = serde_json::from_slice(&mapped.stdout).unwrap();
    let revision = manifest["sourceRevision"].as_str().unwrap();
    assert_eq!(revision.len(), 40);
    assert!(revision.bytes().all(|byte| byte.is_ascii_hexdigit()));
    assert_ne!(revision, "uncommitted");
    assert_eq!(
        manifest["syntaxRevision"],
        "740182f13b84858008d6f176f75136737d405c1b"
    );
}
