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
    assert_eq!(revision, env!("TL_MLTL_SOURCE_REVISION"));
    assert_eq!(manifest["sourceState"], env!("TL_MLTL_SOURCE_STATE"));
    // The compiled dependency identity, not the corpus basis. `tl_mltl` exposes
    // both as separate constants precisely so a reader cannot mistake one for
    // the other, and the mapping manifest carries the compiled one.
    assert_eq!(manifest["syntaxRevision"], tl_mltl::TL_SYNTAX_REVISION);
    assert_eq!(
        tl_mltl::TL_SYNTAX_REVISION,
        "953ee825e5060335b4c79682f5f41a78c5a1bfae"
    );
    assert_ne!(tl_mltl::TL_SYNTAX_REVISION, tl_mltl::TL_SYNTAX_CORPUS_BASIS);
}
