use std::{fs, path::PathBuf};

use serde::Deserialize;
use tl_mltl::{analyze_horizon, evaluate_closed, EvaluationLimits, TruthValue};
use tl_syntax::{FormulaDocument, PropositionId};

#[derive(Deserialize)]
struct Manifest {
    corpus_revision: String,
    fixtures: Vec<Fixture>,
}

#[derive(Deserialize)]
struct Fixture {
    id: String,
    formula: String,
    expected_validation: String,
    #[serde(default)]
    trace: Vec<Vec<PropositionId>>,
    expected_horizon: Option<u64>,
    expected_closed_trace: Option<bool>,
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/tl-syntax-v1")
}

// Trace: TC-005, FR-002-AC-1
#[test]
fn shared_corpus_horizons_and_closed_verdicts_match() {
    let root = corpus_root();
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    assert_eq!(manifest.corpus_revision, "tl-syntax-corpus/v1");

    for fixture in manifest
        .fixtures
        .iter()
        .filter(|fixture| fixture.expected_validation == "valid")
    {
        let document: FormulaDocument =
            serde_json::from_slice(&fs::read(root.join(&fixture.formula)).unwrap()).unwrap();
        let formula = document.validate().unwrap();
        let horizon = analyze_horizon(formula, fixture.id.clone()).unwrap();
        assert_eq!(
            Some(horizon.lookahead),
            fixture.expected_horizon,
            "{}",
            fixture.id
        );

        if let Some(expected) = fixture.expected_closed_trace {
            let report = evaluate_closed(
                formula,
                fixture.id.clone(),
                &fixture.trace,
                format!("{}-trace", fixture.id),
                EvaluationLimits::default(),
            )
            .unwrap();
            assert_eq!(
                report.verdict,
                if expected {
                    TruthValue::True
                } else {
                    TruthValue::False
                },
                "{}",
                fixture.id
            );
        }
    }
}

// Trace: TC-012, NFR-002-AC-1
#[test]
fn shared_malformed_documents_remain_rejected() {
    let root = corpus_root();
    let manifest: Manifest =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    for fixture in manifest
        .fixtures
        .iter()
        .filter(|fixture| fixture.expected_validation == "invalid")
    {
        let bytes = fs::read(root.join(&fixture.formula)).unwrap();
        let parsed = serde_json::from_slice::<FormulaDocument>(&bytes);
        assert!(
            parsed
                .map(|document| document.validate().is_err())
                .unwrap_or(true),
            "{}",
            fixture.id
        );
    }
}
