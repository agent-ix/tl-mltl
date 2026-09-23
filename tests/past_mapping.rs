use std::collections::BTreeSet;

use tl_mltl::{
    map_past_to_c2po, MappingSourceIdentity, MappingSourceState, PastMappingError,
    TargetOriginContract, ToolIdentity,
};
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PastOperatorKind,
    PropositionBinding, PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain,
    SignalId,
};

fn source() -> MappingSourceIdentity {
    MappingSourceIdentity {
        revision: "source".to_owned(),
        state: MappingSourceState::Clean,
    }
}

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

fn contract(operators: &[PastOperatorKind]) -> TargetOriginContract {
    TargetOriginContract {
        target: ToolIdentity {
            name: "C2PO".to_owned(),
            version: "4.1.0-test-fixture".to_owned(),
            executable_sha256: "a".repeat(64),
            configuration_sha256: "b".repeat(64),
        },
        evidence_sha256: "c".repeat(64),
        admitted_operators: operators.iter().copied().collect::<BTreeSet<_>>(),
    }
}

fn render(
    nodes: &[Node],
    origin: &TargetOriginContract,
) -> Result<tl_mltl::PastMappingManifest, PastMappingError> {
    let formula = Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap();
    map_past_to_c2po(
        formula,
        "past",
        b"formula-v2",
        source(),
        &catalog(),
        origin,
        100,
    )
}

fn p() -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    })
}
fn q() -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(1),
    })
}
fn interval(a: u32, b: u32) -> Interval {
    Interval::new(a, b).unwrap()
}

// Trace: TC-160, TC-165; FR-038-AC-1, FR-039-AC-1
#[test]
fn admitted_once_historically_and_previous_render_exact_past_forms() {
    let origin = contract(&[
        PastOperatorKind::Once,
        PastOperatorKind::Historically,
        PastOperatorKind::StrongPrevious,
    ]);
    let once = render(
        &[
            p(),
            Node::new(NodeKind::Once {
                interval: interval(0, 2),
                operand: NodeId(0),
            }),
        ],
        &origin,
    )
    .unwrap();
    assert_eq!(once.expression, "O[0,2](p)");
    assert_eq!(once.clock, "event_position");
    assert_eq!(once.profile, "mltl.origin-complete-history/v1");
    assert_eq!(once.target_origin_evidence_sha256, origin.evidence_sha256);
    let historically = render(
        &[
            p(),
            Node::new(NodeKind::Historically {
                interval: interval(1, 2),
                operand: NodeId(0),
            }),
        ],
        &origin,
    )
    .unwrap();
    assert_eq!(historically.expression, "H[1,2](p)");
    let previous = render(
        &[
            p(),
            Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
        ],
        &origin,
    )
    .unwrap();
    assert_eq!(previous.expression, "O[1,1](p)");
}

// Trace: TC-160, TC-161, TC-166; FR-038-AC-1, FR-039-AC-1
#[test]
fn since_and_triggered_use_explicit_target_forms() {
    let origin = contract(&[PastOperatorKind::Since, PastOperatorKind::Triggered]);
    let since = render(
        &[
            p(),
            q(),
            Node::new(NodeKind::Since {
                interval: interval(0, 1),
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
        &origin,
    )
    .unwrap();
    assert_eq!(since.expression, "(p S[0,1] q)");
    let trigger = render(
        &[
            p(),
            q(),
            Node::new(NodeKind::Triggered {
                interval: interval(0, 1),
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
        &origin,
    )
    .unwrap();
    assert_eq!(trigger.expression, "(!((!p) S[0,1] (!q)))");
}

// Trace: TC-166, TC-167; FR-039-AC-1, FR-039-AC-2
#[test]
fn target_4_2_origin_mismatch_intervals_refuse_without_artifact() {
    let origin = contract(&[
        PastOperatorKind::Once,
        PastOperatorKind::Historically,
        PastOperatorKind::Since,
        PastOperatorKind::Triggered,
    ]);
    for (kind, operator, bounds) in [
        (
            NodeKind::Once {
                interval: interval(2, 2),
                operand: NodeId(0),
            },
            PastOperatorKind::Once,
            interval(2, 2),
        ),
        (
            NodeKind::Historically {
                interval: interval(2, 2),
                operand: NodeId(0),
            },
            PastOperatorKind::Historically,
            interval(2, 2),
        ),
        (
            NodeKind::Since {
                interval: interval(0, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
            PastOperatorKind::Since,
            interval(0, 2),
        ),
        (
            NodeKind::Triggered {
                interval: interval(1, 1),
                left: NodeId(0),
                right: NodeId(1),
            },
            PastOperatorKind::Triggered,
            interval(1, 1),
        ),
    ] {
        let nodes = [p(), q(), Node::new(kind)];
        assert_eq!(
            render(&nodes, &origin),
            Err(PastMappingError::TargetOriginIntervalMismatch {
                operator,
                interval: bounds
            })
        );
    }
}

// Trace: TC-162, TC-167, TC-173; FR-038-AC-2, FR-039-AC-2, FR-041-AC-2
#[test]
fn absent_or_operator_incomplete_origin_evidence_refuses_without_artifact() {
    let nodes = [
        p(),
        Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
    ];
    assert_eq!(
        render(&nodes, &contract(&[])),
        Err(PastMappingError::TargetOriginUnverified(
            PastOperatorKind::StrongPrevious
        ))
    );
    let mut invalid = contract(&[PastOperatorKind::StrongPrevious]);
    invalid.evidence_sha256.clear();
    assert_eq!(
        render(&nodes, &invalid),
        Err(PastMappingError::MissingOriginEvidence)
    );
}

// Trace: TC-162, TC-167; FR-038-AC-2, FR-039-AC-2
#[test]
fn each_target_origin_identity_field_is_required() {
    let nodes = [p()];
    let corruptions: [fn(&mut TargetOriginContract); 5] = [
        |origin: &mut TargetOriginContract| origin.target.name.clear(),
        |origin: &mut TargetOriginContract| origin.target.version.clear(),
        |origin: &mut TargetOriginContract| origin.evidence_sha256 = "C".repeat(64),
        |origin: &mut TargetOriginContract| origin.target.executable_sha256.clear(),
        |origin: &mut TargetOriginContract| origin.target.configuration_sha256.clear(),
    ];
    for corrupt in corruptions {
        let mut invalid = contract(&[]);
        corrupt(&mut invalid);
        assert_eq!(
            render(&nodes, &invalid),
            Err(PastMappingError::MissingOriginEvidence)
        );
    }
}

// Trace: TC-162, TC-167; FR-038-AC-2, FR-039-AC-2
#[test]
fn past_mapping_refuses_wrong_profile_and_exhausted_render_budget() {
    let nodes = [p()];
    let origin = contract(&[]);
    let map = |profile, limit| {
        let formula = Formula::new(profile, NodeId(0), &nodes).unwrap();
        map_past_to_c2po(formula, "p", b"p", source(), &catalog(), &origin, limit)
    };
    assert_eq!(
        map(SemanticProfile::ClosedTraceV1, 1),
        Err(PastMappingError::UnsupportedProfile)
    );
    assert_eq!(
        map(SemanticProfile::OriginCompleteHistoryV1, 0),
        Err(PastMappingError::ResourceIncomplete)
    );
}

// Trace: TC-162, TC-167; FR-038-AC-2, FR-039-AC-2
#[test]
fn past_mapping_refuses_signal_names_with_no_target_identifier() {
    let nodes = [p()];
    let formula =
        Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(0), &nodes).unwrap();
    let catalog = SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(1),
            "p.signal".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(0), SignalId(1))],
    )
    .unwrap();
    assert_eq!(
        map_past_to_c2po(formula, "p", b"p", source(), &catalog, &contract(&[]), 10),
        Err(PastMappingError::UnsupportedSignal(PropositionId(0)))
    );
}
