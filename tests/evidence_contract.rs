use std::{fs, path::PathBuf, process::Command};

use sha2::{Digest, Sha256};

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
        "quire validate --scope . 'spec/**/*.md'",
        "quire coverage --scope . --strict",
        "RUSTDOCFLAGS=-Dwarnings",
        "doc --no-deps --all-features",
        "verify_evidence.sh",
    ] {
        assert!(ci_commands.contains(command), "make ci omits {command}");
    }
    for command in [
        "make ci",
        "make spec",
        "quire coverage --scope . --strict",
        "PGM01_SCHEMA",
        "PGM01_PYTHON",
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
    for (manifest, expected) in [
        (
            "tl-mltl-v01-5072994619f8-20260831T022058Z.sha256",
            "3ef7708da61afc06968b71a9b74cdc959dc2230fb9f963d3497d78c5766de861",
        ),
        (
            "tl-mltl-v01-786d3932a5e5-20260831T041429Z.sha256",
            "daf272959ed787fbcfe5726d6a63d4d4e3332e95f42050ba1481b56c830b852e",
        ),
        (
            "tl-mltl-v01-a9b7847199c1-20260831T022156Z.sha256",
            "502f996f24b95714f9caa0adbac4d0a4fee5b27a02e54fb8e273300dda03872f",
        ),
        (
            "tl-mltl-v01-fced0e687f99-20260831T041645Z.sha256",
            "dd1b29c45df064cfe531aa044f9b5c669a8f79ce198f6ec3c2c30b81b03c543b",
        ),
    ] {
        let actual = format!(
            "{:x}",
            Sha256::digest(fs::read(root.join("evidence").join(manifest)).unwrap())
        );
        assert_eq!(
            actual, expected,
            "retained archive anchor changed: {manifest}"
        );
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
