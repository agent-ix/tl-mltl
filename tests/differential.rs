use std::{collections::BTreeMap, fs, path::PathBuf};

use serde::Deserialize;
use sha2::{Digest, Sha256};
use tl_mltl::{
    compare_external, evaluate_prefix_at, ComparisonStatus, EvaluationLimits, ExternalStatus,
    ExternalVerdict, ToolIdentity, TruthValue,
};
use tl_syntax::{FormulaDocument, PropositionId};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    source: Source,
    tools: Tools,
    artifacts: BTreeMap<String, String>,
    trace: Vec<Vec<PropositionId>>,
    cases: Vec<Case>,
    unsupported_cases: Vec<UnsupportedCase>,
}

#[derive(Deserialize)]
struct Source {
    license: String,
}

#[derive(Deserialize)]
struct Tools {
    r2u2: R2u2,
}

#[derive(Deserialize)]
struct R2u2 {
    version: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    formula_index: u32,
    formula: FormulaDocument,
    expected: Expected,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Expected {
    verdict: bool,
    verdict_time: u64,
}

#[derive(Deserialize)]
struct UnsupportedCase {
    id: String,
    reason: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/r2u2-v4.2")
}

fn sha256(path: PathBuf) -> String {
    Sha256::digest(fs::read(path).unwrap())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn time_indexed_verdicts(stdout: &str) -> BTreeMap<(u32, u64), bool> {
    stdout
        .lines()
        .filter_map(|line| {
            let (identity_time, value) = line.split_once(',')?;
            let (identity, time) = identity_time.split_once(':')?;
            Some(((identity.parse().ok()?, time.parse().ok()?), value == "T"))
        })
        .collect()
}

// Trace: TC-015, FR-005-AC-2, StR-001-VC-2
#[test]
fn retained_r2u2_run_agrees_for_supported_cases() {
    let root = root();
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.source.license, "Apache-2.0");
    let external = time_indexed_verdicts(&fs::read_to_string(root.join("r2u2.stdout")).unwrap());
    assert!(external.len() >= manifest.cases.len());

    for case in manifest.cases {
        let formula = case.formula.validate().unwrap();
        let reference = evaluate_prefix_at(
            formula,
            case.id.clone(),
            &manifest.trace,
            "r2u2-v4.2-trace",
            false,
            case.expected.verdict_time,
            EvaluationLimits::default(),
        )
        .unwrap();
        assert_eq!(reference.verdict_time, case.expected.verdict_time);
        assert_eq!(
            reference.verdict,
            if case.expected.verdict {
                TruthValue::True
            } else {
                TruthValue::False
            }
        );
        let ((_, observed_time), observed_value) = external
            .get_key_value(&(case.formula_index, case.expected.verdict_time))
            .unwrap();
        let external_verdict = ExternalVerdict {
            schema_version: "tl-mltl.external-verdict/v1".to_owned(),
            tool: ToolIdentity {
                name: "r2u2".to_owned(),
                version: manifest.tools.r2u2.version.clone(),
                executable_sha256: manifest.tools.r2u2.sha256.clone(),
                configuration_sha256: manifest.artifacts["spec.bin"].clone(),
            },
            formula_id: case.id,
            trace_id: "r2u2-v4.2-trace".to_owned(),
            status: ExternalStatus::Conclusive,
            value: Some(*observed_value),
            verdict_time: Some(*observed_time),
            detail: None,
        };
        let result = compare_external(&reference, external_verdict.clone());
        assert_eq!(result.status, ComparisonStatus::Agreement);

        let mut wrong_time = external_verdict;
        wrong_time.verdict_time = wrong_time.verdict_time.and_then(|time| time.checked_add(1));
        assert_eq!(
            compare_external(&reference, wrong_time).status,
            ComparisonStatus::Mismatch
        );
    }
}

// Trace: TC-013, TC-015, TC-016, FR-004-AC-3, FR-005-AC-2, FR-005-AC-3, NFR-002-AC-2
#[test]
fn retained_external_inputs_and_nonconclusive_cases_are_complete() {
    let root = root();
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    for (path, expected) in &manifest.artifacts {
        assert_eq!(&sha256(root.join(path)), expected, "{path}");
    }
    assert_eq!(manifest.unsupported_cases.len(), 1);
    assert_eq!(
        manifest.unsupported_cases[0].id,
        "closed-profile-not-mapped-v1"
    );
    assert!(manifest.unsupported_cases[0]
        .reason
        .contains("online-prefix"));

    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("differential-report.json")).unwrap()).unwrap();
    assert_eq!(report["agreements"], 8);
    assert_eq!(report["mismatches"], 0);
    assert_eq!(report["unsupported"], 1);
    assert_eq!(report["releaseDecision"], "pending-human-review");
}
