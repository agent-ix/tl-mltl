use std::{fs, path::Path};

// Trace: TC-016, FR-005-AC-3, NFR-002-AC-2, SUITE-001, SUITE-002, SUITE-003
// Trace: SUITE-004, SUITE-005, SUITE-006, SUITE-007
#[test]
fn evidence_contract_is_complete_and_wired_to_gates() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let collector = fs::read_to_string(root.join("scripts/collect_evidence.sh")).unwrap();
    let builder = fs::read_to_string(root.join("scripts/build_evidence_envelope.py")).unwrap();
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    let suites = fs::read_to_string(root.join("spec/evidence/suites.md")).unwrap();

    for target in ["ci:", "spec:", "check-corpus:", "evidence-tool:"] {
        assert!(makefile.contains(target), "missing Make target {target}");
    }
    for command in [
        "make ci",
        "make spec",
        "quire coverage --scope . --strict",
        "PGM01_SCHEMA",
        "PGM01_VALIDATOR",
    ] {
        assert!(
            collector.contains(command) || builder.contains(command),
            "{command}"
        );
    }
    for identity in [
        "5e59a26d71b4b5d79623850cda50010e18a90dad",
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
    }
}
