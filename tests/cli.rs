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
}

const CONTRACT: &str = "tests/fixtures/cli-contract";
const REQUESTS: &str = "tests/fixtures/cli-requests";

fn run_args(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_tl-mltl"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .stdin(Stdio::null())
        .output()
        .unwrap()
}

fn run_stdin_bytes(bytes: &[u8]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tl-mltl"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(bytes).unwrap();
    child.wait_with_output().unwrap()
}

/// A refusal writes nothing to stdout and one `tl-mltl: ` line to stderr.
fn assert_refused_prefix(output: &std::process::Output, message_prefix: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "refusal wrote to stdout");
    let stderr = String::from_utf8(output.stderr.clone()).unwrap();
    assert!(
        stderr.starts_with(&format!("tl-mltl: {message_prefix}")) && stderr.ends_with('\n'),
        "unexpected stderr: {stderr:?}"
    );
    assert_eq!(stderr.lines().count(), 1, "{stderr:?}");
}

/// Like `assert_refused_prefix` for platform-independent text: the whole
/// stderr is exactly `tl-mltl: <message>\n`.
fn assert_refused(output: &std::process::Output, message: &str) {
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty(), "refusal wrote to stdout");
    assert_eq!(
        String::from_utf8(output.stderr.clone()).unwrap(),
        format!("tl-mltl: {message}\n")
    );
}

const USAGE: &str = "usage: tl-mltl REQUEST.json (or - for stdin)\n";

// Trace: TC-014, FR-005-AC-1
#[test]
fn cli_usage_errors_exit_two_with_the_usage_line() {
    let none = run_args(&[]);
    assert_eq!(none.status.code(), Some(2));
    assert!(none.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&none.stderr),
        format!("tl-mltl: {USAGE}")
    );

    let extra = run_args(&["a.json", "b.json"]);
    assert_eq!(extra.status.code(), Some(2));
    assert!(extra.stdout.is_empty());
    assert_eq!(
        String::from_utf8_lossy(&extra.stderr),
        format!("tl-mltl: {USAGE}")
    );
}

// Trace: TC-014, FR-005-AC-1
#[test]
fn cli_missing_file_is_refused_naming_the_path() {
    let missing = run_args(&["tests/fixtures/cli-contract/does-not-exist.json"]);
    assert_refused_prefix(
        &missing,
        "read tests/fixtures/cli-contract/does-not-exist.json: ",
    );
}

// Trace: TC-014, FR-005-AC-1
#[test]
fn cli_directory_as_request_path_is_refused_naming_the_path() {
    let directory = run_args(&[CONTRACT]);
    assert_refused_prefix(&directory, &format!("read {CONTRACT}: "));
}

// Trace: TC-014, FR-005-AC-1
#[cfg(unix)]
#[test]
fn cli_unreadable_stdin_is_refused_as_a_stdin_read_error() {
    let directory = std::fs::File::open(CONTRACT).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_tl-mltl"))
        .arg("-")
        .stdin(Stdio::from(directory))
        .output()
        .unwrap();
    assert_refused_prefix(&output, "read stdin: ");
}

// Trace: TC-014, FR-005-AC-1
#[test]
fn cli_non_utf8_and_malformed_stdin_are_refused_at_parse() {
    // Invalid UTF-8 inside a JSON string.
    let non_utf8 = run_stdin_bytes(b"{\"schemaVersion\":\"\xff\"}");
    assert_refused(
        &non_utf8,
        "parse request: invalid unicode code point at line 1 column 20",
    );

    // Bytes that are not JSON at all.
    let garbage = run_stdin_bytes(b"\xff\xfe");
    assert_refused(&garbage, "parse request: expected value at line 1 column 1");

    // Empty stdin.
    let empty = run_stdin_bytes(b"");
    assert_refused(
        &empty,
        "parse request: EOF while parsing a value at line 1 column 0",
    );
}

// Trace: TC-014, FR-005-AC-1
#[test]
fn cli_evaluates_an_online_prefix_formula_over_an_open_prefix() {
    let output = run_args(&[&format!("{CONTRACT}/cli-online-prefix-evaluate.json")]);
    assert!(output.status.success());
    assert!(output.stderr.is_empty());
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schemaVersion"], "tl-mltl.evaluation/v1");
    assert_eq!(report["semanticProfile"], "mltl.online-prefix/v1");
    assert_eq!(report["traceClosed"], false);
    assert_eq!(report["traceId"], "cli-contract-trace-v1");
    assert_eq!(report["verdict"], "true");
}

// Trace: TC-014, FR-005-AC-1, NFR-002-AC-1
#[test]
fn cli_refuses_profiles_the_v1_wire_does_not_admit() {
    // Origin-complete and infinite-trace formulas never reach evaluation: the
    // v1 formula document refuses them while the request is parsed.
    let origin = run_args(&[&format!("{CONTRACT}/cli-origin-complete-profile.json")]);
    assert_refused(
        &origin,
        "parse request: formula-v1 does not admit semantic profile \
         mltl.origin-complete-history/v1 at line 10 column 3",
    );
    let infinite = run_args(&[&format!("{CONTRACT}/cli-infinite-trace-profile.json")]);
    assert_refused(&infinite,
        "parse request: infinite-trace profile requires tl-syntax.formula-unbounded/v1 at line 10 column 3",
    );
}

// Trace: TC-014, FR-005-AC-1
#[test]
fn cli_refuses_an_invalid_formula_and_an_unmappable_profile() {
    let root = run_args(&[&format!("{CONTRACT}/cli-root-outside-table.json")]);
    assert_refused(
        &root,
        "parse request: formula root 5 is outside node table of length 1 at line 10 column 3",
    );
    let mapping = run_args(&[&format!("{CONTRACT}/cli-map-closed-profile.json")]);
    assert_refused(
        &mapping,
        "map formula: R2U2/C2PO mapping requires mltl.online-prefix/v1, \
         found mltl.closed-trace/v1",
    );
}

// Trace: TC-014, FR-005-AC-1, NFR-002-AC-1
#[test]
fn cli_refuses_each_declared_request_with_its_stable_message() {
    let manifest: Value =
        serde_json::from_slice(&std::fs::read(format!("{REQUESTS}/manifest.json")).unwrap())
            .unwrap();
    let mut refused = 0;
    for case in manifest["cases"].as_array().unwrap() {
        if case["expectedExit"] != 2 {
            continue;
        }
        let file = case["request"].as_str().unwrap();
        let marker = case["stderrMarker"].as_str().unwrap();
        let output = run_args(&[&format!("{REQUESTS}/{file}")]);
        assert_eq!(output.status.code(), Some(2), "{file}");
        assert!(output.stdout.is_empty(), "{file}");
        let stderr = String::from_utf8(output.stderr).unwrap();
        assert!(stderr.starts_with("tl-mltl: "), "{file}: {stderr:?}");
        assert!(stderr.contains(marker), "{file}: {stderr:?}");
        refused += 1;
    }
    assert_eq!(refused, 5);
}
