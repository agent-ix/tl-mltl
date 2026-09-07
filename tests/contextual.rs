use tl_mltl::{
    analyze_horizon_with_context, compare_external_with_context, evaluate_prefix_with_context,
    map_to_c2po_with_context, ContextualComparisonStatus, ContextualExternalVerdict,
    ContextualExternalVerdictSchemaVersion, EvaluationLimits, ExternalStatus,
    MappingSourceIdentity, MappingSourceState, ToolIdentity,
};
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, RequirementContextDocument, SemanticProfile, SignalCatalogDocument,
    SignalDomain, SignalId, SourceSpan,
};

fn overlay_nodes() -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(7),
        }),
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(8),
        }),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 2).unwrap(),
            operand: NodeId(1),
        }),
        Node::new(NodeKind::Implies {
            left: NodeId(0),
            right: NodeId(2),
        }),
    ]
}

fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(
                SignalId(10),
                "overlay_change_accepted".to_owned(),
                SignalDomain::Boolean,
            ),
            OwnedSignalDeclaration::new(
                SignalId(11),
                "response_within_2_cycles".to_owned(),
                SignalDomain::Boolean,
            ),
        ],
        vec![
            PropositionBinding::new(PropositionId(7), SignalId(10)),
            PropositionBinding::new(PropositionId(8), SignalId(11)),
        ],
    )
    .unwrap()
}

fn context() -> RequirementContextDocument {
    RequirementContextDocument::new(
        "agent-ix/quire-contract-ir#57".to_owned(),
        "overlay-example-v1".to_owned(),
        "response-window".to_owned(),
        "overlay-change-accepted-response-within-n".to_owned(),
        SourceSpan::new(11, 42).unwrap(),
    )
    .unwrap()
}

fn tool() -> ToolIdentity {
    ToolIdentity {
        name: "external-fixture".to_owned(),
        version: "1".to_owned(),
        executable_sha256: "a".repeat(64),
        configuration_sha256: "b".repeat(64),
    }
}

// Trace: TC-031, FR-007-AC-7, StR-003-VC-1, NFR-002-AC-4
#[test]
fn overlay_response_context_flows_through_every_contextual_operation() {
    let nodes = overlay_nodes();
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), &nodes).unwrap();
    let catalog = catalog();
    let context = context();
    let trace = vec![vec![PropositionId(7)], vec![PropositionId(8)]];
    let evaluation = evaluate_prefix_with_context(
        formula,
        "overlay-response",
        &trace,
        "overlay-trace",
        false,
        EvaluationLimits::default(),
        &catalog,
        Some(&context),
    )
    .unwrap();
    let horizon =
        analyze_horizon_with_context(formula, "overlay-response", &catalog, Some(&context))
            .unwrap();
    let mapping = map_to_c2po_with_context(
        formula,
        "overlay-response",
        b"overlay-response",
        MappingSourceIdentity {
            revision: "fixture".to_owned(),
            state: MappingSourceState::Clean,
        },
        None,
        100,
        &catalog,
        Some(&context),
    )
    .unwrap();
    let external = ContextualExternalVerdict {
        schema_version: ContextualExternalVerdictSchemaVersion::V2,
        tool: tool(),
        formula_id: evaluation.formula_id.clone(),
        trace_id: evaluation.trace_id.clone(),
        signal_catalog_sha256: evaluation.signal_catalog_sha256.clone(),
        requirement_context: Some(context.clone()),
        status: ExternalStatus::Conclusive,
        value: Some(true),
        verdict_time: Some(evaluation.verdict_time),
        detail: None,
    };
    let differential = compare_external_with_context(&evaluation, external).unwrap();
    assert_eq!(differential.status, ContextualComparisonStatus::Agreement);
    assert_eq!(evaluation.requirement_context, Some(context.clone()));
    assert_eq!(horizon.requirement_context, Some(context.clone()));
    assert_eq!(mapping.requirement_context, Some(context));
    assert!(mapping.expression.contains("overlay_change_accepted"));
    assert!(mapping.expression.contains("response_within_2_cycles"));
}
