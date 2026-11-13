//! TL-371: behavior tests for the past-profile C2PO mapping, which the first full cargo-mutants run
//! showed were not asserted at the render-work boundary or for binary operators and messages.

use tl_mltl::{
    map_past_to_c2po, MappingSourceIdentity, MappingSourceState, PastMappingError,
    TargetOriginContract,
};
use tl_syntax::{
    Formula, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding, PropositionId,
    SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};

fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "p".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "q".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .unwrap()
}

type Kind = fn(NodeId, NodeId) -> NodeKind;

fn map(nodes: &[Node], limit: u64) -> Result<String, PastMappingError> {
    let formula = Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap();
    map_past_to_c2po(
        formula,
        "f",
        b"bytes",
        MappingSourceIdentity {
            revision: "r".to_owned(),
            state: MappingSourceState::Clean,
        },
        &catalog(),
        &TargetOriginContract::reviewed_r2u2_4_2(),
        limit,
    )
    .map(|manifest| manifest.expression)
}

fn prop(id: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(id),
    })
}

// Trace: TC-160
#[test]
fn the_render_work_limit_is_inclusive_and_binary_forms_render_both_operands() {
    assert_eq!(map(&[prop(0)], 1).unwrap(), "p");
    assert_eq!(
        map(&[prop(0)], 0),
        Err(PastMappingError::ResourceIncomplete)
    );
    let and = |kind: Kind| vec![prop(0), prop(1), Node::new(kind(NodeId(0), NodeId(1)))];
    let cases: [(Kind, &str); 4] = [
        (|left, right| NodeKind::And { left, right }, "(p && q)"),
        (|left, right| NodeKind::Or { left, right }, "(p || q)"),
        (|left, right| NodeKind::Implies { left, right }, "(p -> q)"),
        (
            |left, right| NodeKind::Equivalent { left, right },
            "(p <-> q)",
        ),
    ];
    for (kind, expected) in cases {
        assert_eq!(map(&and(kind), 100).unwrap(), expected);
    }
    // Three nodes need exactly three visits.
    assert!(map(&and(cases[0].0), 3).is_ok());
    assert_eq!(
        map(&and(cases[0].0), 2),
        Err(PastMappingError::ResourceIncomplete)
    );
}

// Trace: TC-160
#[test]
fn mapping_errors_render_their_variant() {
    let message = PastMappingError::TargetOriginMismatch.to_string();
    assert!(message.contains("TargetOriginMismatch"), "{message}");
}
