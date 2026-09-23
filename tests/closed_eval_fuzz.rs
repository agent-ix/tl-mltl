use std::{fs, path::Path};

use tl_mltl::future::{evaluate_closed_at, EvaluationLimits};
use tl_syntax::{Formula, FormulaDocument, PropositionId, SemanticProfile, SyntaxArtifactLimits};

// Trace: TC-181; FR-046-AC-1
#[test]
fn checked_closed_evaluation_seeds_reach_strict_reader_and_evaluator() {
    let corpus = Path::new("fuzz/corpus/closed_eval");
    let seeds: Vec<_> = fs::read_dir(corpus)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.file_name() != "SHA256SUMS")
        .collect();
    assert_eq!(seeds.len(), 4);
    let trace = [
        vec![PropositionId(0)],
        vec![PropositionId(1)],
        vec![PropositionId(0), PropositionId(1)],
    ];
    let mut admitted = 0;
    let mut rejected = 0;
    for seed in seeds {
        let bytes = fs::read(seed.path()).unwrap();
        match FormulaDocument::from_json_bytes(&bytes, SyntaxArtifactLimits::default()) {
            Ok(document) => {
                assert_eq!(document.semantic_profile(), SemanticProfile::ClosedTraceV1);
                let formula = Formula::new(
                    document.semantic_profile(),
                    document.root(),
                    document.nodes(),
                )
                .unwrap();
                let report = evaluate_closed_at(
                    formula,
                    "seed-formula",
                    &trace,
                    "seed-trace",
                    0,
                    EvaluationLimits::default(),
                )
                .unwrap();
                assert_eq!(report.trace_length, 3);
                admitted += 1;
            }
            Err(_) => {
                assert_eq!(seed.file_name(), "malformed");
                rejected += 1;
            }
        }
    }
    assert_eq!((admitted, rejected), (3, 1));
}
