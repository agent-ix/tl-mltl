use std::{collections::HashSet, fs, path::Path};

use sha2::{Digest, Sha256};
use tl_mltl::future::{evaluate_closed_at, EvaluationLimits};
use tl_syntax::{Formula, FormulaDocument, PropositionId, SemanticProfile, SyntaxArtifactLimits};

// Trace: TC-181; FR-046-AC-1
#[test]
fn checked_closed_evaluation_seeds_reach_strict_reader_and_evaluator() {
    let corpus = Path::new("fuzz/corpus/closed_eval");
    let manifest = fs::read_to_string(corpus.join("SHA256SUMS")).unwrap();
    let mut names = HashSet::new();
    let seeds: Vec<_> = manifest
        .lines()
        .map(|row| {
            let (expected, name) = row.split_once("  ").expect("digest row");
            assert!(names.insert(name), "duplicate seed {name}");
            let bytes = fs::read(corpus.join(name)).expect("listed seed");
            assert_eq!(format!("{:x}", Sha256::digest(&bytes)), expected, "{name}");
            (name.to_owned(), bytes)
        })
        .collect();
    assert_eq!(seeds.len(), 4);
    assert_eq!(
        fs::read_dir(corpus).unwrap().count(),
        seeds.len() + 1,
        "every seed must be listed in SHA256SUMS"
    );
    let trace = [
        vec![PropositionId(0)],
        vec![PropositionId(1)],
        vec![PropositionId(0), PropositionId(1)],
    ];
    let mut admitted = 0;
    let mut rejected = 0;
    for (name, bytes) in seeds {
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
                assert_eq!(name, "malformed");
                rejected += 1;
            }
        }
    }
    assert_eq!((admitted, rejected), (3, 1));
}
