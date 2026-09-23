use std::{collections::BTreeSet, fs};

use tl_mltl::{
    map_past_to_c2po, MappingSourceIdentity, MappingSourceState, PastMappingError,
    TargetOriginContract, ToolIdentity,
};
use tl_syntax::{
    Formula, FormulaDocument, OwnedSignalDeclaration, PastOperatorKind, PropositionBinding,
    PropositionId, SignalCatalogDocument, SignalDomain, SignalId, SyntaxArtifactLimits,
};

// Trace: TC-174; FR-042-AC-2
#[test]
fn every_fuzz_seed_reaches_the_strict_reader_and_real_c2po_mapper() {
    let root = "fuzz/corpus/c2po_map";
    let seeds: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(Result::unwrap)
        .filter(|entry| entry.file_name() != "SHA256SUMS")
        .collect();
    assert_eq!(seeds.len(), 8);
    let catalog = SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "p".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "q".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .unwrap();
    let origin = TargetOriginContract {
        target: ToolIdentity {
            name: "C2PO-fuzz-fixture".to_owned(),
            version: "v1".to_owned(),
            executable_sha256: "a".repeat(64),
            configuration_sha256: "b".repeat(64),
        },
        evidence_sha256: "c".repeat(64),
        admitted_operators: [
            PastOperatorKind::Once,
            PastOperatorKind::Historically,
            PastOperatorKind::StrongPrevious,
            PastOperatorKind::Since,
            PastOperatorKind::Triggered,
        ]
        .into_iter()
        .collect::<BTreeSet<_>>(),
    };
    for seed in &seeds {
        let bytes = fs::read(seed.path()).unwrap();
        let document = FormulaDocument::from_json_bytes(&bytes, SyntaxArtifactLimits::default())
            .unwrap_or_else(|error| panic!("{}: {error:?}", seed.path().display()));
        let formula = Formula::new(
            document.semantic_profile(),
            document.root(),
            document.nodes(),
        )
        .unwrap();
        let mapped = map_past_to_c2po(
            formula,
            "fuzz-seed",
            &bytes,
            MappingSourceIdentity {
                revision: "fuzz-seed".to_owned(),
                state: MappingSourceState::Clean,
            },
            &catalog,
            &origin,
            10_000,
        );
        assert!(
            mapped.is_ok()
                || matches!(
                    mapped,
                    Err(PastMappingError::TargetOriginIntervalMismatch { .. })
                ),
            "{}: {mapped:?}",
            seed.path().display()
        );
    }
    let record: serde_json::Value =
        serde_json::from_slice(&fs::read("fuzz/runs/2026-09-22-c2po_map.json").unwrap()).unwrap();
    assert_eq!(record["target"], "c2po_map");
    assert_eq!(record["seed"], 174);
    assert_eq!(record["completedRuns"], 200);
    assert_eq!(
        record["initialCorpusCount"].as_u64(),
        Some(seeds.len() as u64)
    );
    assert_eq!(record["crashes"], 0);
    assert_eq!(record["generatedSeedsRetained"], 0);
}
