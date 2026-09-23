#![no_main]

use std::collections::BTreeSet;

use libfuzzer_sys::fuzz_target;
use sha2::{Digest, Sha256};
use tl_mltl::{
    map_past_to_c2po, MappingSourceIdentity, MappingSourceState, TargetOriginContract,
    ToolIdentity,
};
use tl_syntax::{
    Formula, FormulaDocument, FormulaSchemaVersion, OwnedSignalDeclaration, PastOperatorKind,
    PropositionBinding, PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain,
    SignalId, SyntaxArtifactLimits,
};

fuzz_target!(|data: &[u8]| {
    let Ok(document) = FormulaDocument::from_json_bytes(data, SyntaxArtifactLimits::default()) else {
        return;
    };
    if document.schema_version() != FormulaSchemaVersion::V2
        || document.semantic_profile() != SemanticProfile::OriginCompleteHistoryV1
    {
        return;
    }
    let formula = Formula::new(document.semantic_profile(), document.root(), document.nodes())
        .expect("strict owner reader admitted a malformed formula");
    let catalog = SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "p".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "q".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    ).unwrap();
    // Isolated parser/renderer robustness fixture. No external target behavior
    // is inferred from these synthetic identities.
    let origin = TargetOriginContract {
        target: ToolIdentity {
            name: "C2PO-fuzz-fixture".to_owned(),
            version: "v1".to_owned(),
            executable_sha256: "a".repeat(64),
            configuration_sha256: "b".repeat(64),
        },
        evidence_sha256: "c".repeat(64),
        admitted_operators: [
            PastOperatorKind::Once, PastOperatorKind::Historically,
            PastOperatorKind::StrongPrevious, PastOperatorKind::Since,
            PastOperatorKind::Triggered,
        ].into_iter().collect::<BTreeSet<_>>(),
    };
    if let Ok(manifest) = map_past_to_c2po(
        formula, "fuzz", data,
        MappingSourceIdentity { revision: "fuzz".to_owned(), state: MappingSourceState::Clean },
        &catalog, &origin, 10_000,
    ) {
        assert_eq!(manifest.profile, SemanticProfile::OriginCompleteHistoryV1.as_str());
        assert_eq!(manifest.input_sha256, format!("{:x}", Sha256::digest(data)));
        assert_eq!(manifest.output_sha256,
            format!("{:x}", Sha256::digest(manifest.expression.as_bytes())));
        assert!(!manifest.expression.is_empty());
    }
});
