use std::fs;

use sha2::{Digest, Sha256};
use tl_mltl::{
    map_past_to_c2po, MappingSourceIdentity, MappingSourceState, PastMappingError,
    TargetOriginContract,
};
use tl_syntax::{
    Formula, FormulaDocument, OwnedSignalDeclaration, PropositionBinding, PropositionId,
    SignalCatalogDocument, SignalDomain, SignalId, SyntaxArtifactLimits,
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
    let checksums = fs::read_to_string(format!("{root}/SHA256SUMS")).unwrap();
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
    let origin = TargetOriginContract::reviewed_r2u2_4_2();
    for seed in &seeds {
        let bytes = fs::read(seed.path()).unwrap();
        let name = seed.file_name();
        let expected = format!("{:x}  {}", Sha256::digest(&bytes), name.to_string_lossy());
        assert!(checksums.lines().any(|line| line == expected));
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
                    Err(PastMappingError::TargetOriginIntervalMismatch { .. }
                        | PastMappingError::TargetOriginShapeUnverified(_))
                ),
            "{}: {mapped:?}",
            seed.path().display()
        );
    }
}
