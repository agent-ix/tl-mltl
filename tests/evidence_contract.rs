use std::{fs, path::PathBuf, process::Command};

// Trace: TC-016, FR-005-AC-3, NFR-002-AC-2, SUITE-001, SUITE-002, SUITE-003
// Trace: SUITE-004, SUITE-005, SUITE-006, SUITE-007
#[test]
fn evidence_contract_is_complete_and_wired_to_gates() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let collector = fs::read_to_string(root.join("scripts/collect_evidence.sh")).unwrap();
    let builder = fs::read_to_string(root.join("scripts/build_evidence_envelope.py")).unwrap();
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    let suites = fs::read_to_string(root.join("spec/evidence/suites.md")).unwrap();

    let dry_run = Command::new("make")
        .args(["--no-print-directory", "-n", "ci"])
        .current_dir(&root)
        .output()
        .unwrap();
    assert!(
        dry_run.status.success(),
        "make -n ci failed: {}",
        String::from_utf8_lossy(&dry_run.stderr)
    );
    let ci_commands = String::from_utf8(dry_run.stdout).unwrap();

    for target in [
        "ci:",
        "spec:",
        "check-corpus:",
        "evidence-tool:",
        "verify-evidence:",
    ] {
        assert!(makefile.contains(target), "missing Make target {target}");
    }
    for command in [
        "cargo fmt --all -- --check",
        "cargo clippy --all-targets --all-features -- -D warnings",
        "cargo test --all-targets --all-features",
        "sha256sum --check corpus/tl-syntax-v1.sha256",
        "sha256sum --check SHA256SUMS",
        "cargo deny check licenses",
        "cargo deny check sources",
        "check_unsafe_comments.sh",
        "test_evidence_tool.py",
        "quire validate",
        "quire coverage",
        "cargo doc --no-deps --all-features",
        "verify_evidence.sh",
    ] {
        assert!(ci_commands.contains(command), "make ci omits {command}");
    }
    for command in [
        "make ci",
        "make spec",
        "quire coverage --scope . --strict",
        "PGM01_SCHEMA",
        "PGM01_VALIDATOR",
    ] {
        assert!(collector.contains(command), "collector omits {command}");
    }
    for identity in [
        "740182f13b84858008d6f176f75136737d405c1b",
        "7dac9d8c19952412b56a0347387666e2ca81e01d",
        "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a",
        "tl-mltl.evidence-input/v1",
        "quire.derivation-evidence/v1",
    ] {
        assert!(
            builder.contains(identity),
            "missing evidence identity {identity}"
        );
    }
    for suite in 1..=7 {
        assert!(suites.contains(&format!("SUITE-{suite:03}")));
    }
    for schema in [
        "schemas/tl-mltl-evidence-input-v1.schema.json",
        "schemas/tl-mltl-evidence-manifest-v1.schema.json",
    ] {
        let value: serde_json::Value =
            serde_json::from_slice(&fs::read(root.join(schema)).unwrap()).unwrap();
        assert_eq!(value["$schema"], "http://json-schema.org/draft-07/schema#");
        assert_eq!(value["additionalProperties"], false);
    }
}

// Trace: TC-016, NFR-002-AC-3
#[test]
fn evidence_producer_rejects_false_success_classifications() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = Command::new("python3")
        .arg("scripts/test_evidence_tool.py")
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "evidence behavior test failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
