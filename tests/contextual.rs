use tl_mltl::{
    analyze_horizon, analyze_horizon_with_context, compare_external, compare_external_with_context,
    evaluate_closed_with_context, evaluate_prefix, evaluate_prefix_with_context, map_to_c2po,
    map_to_c2po_with_context, ComparisonStatus, ContextualComparisonStatus,
    ContextualEvaluationError, ContextualExternalVerdict, ContextualExternalVerdictSchemaVersion,
    ContextualHorizonError, DifferentialReport, EvaluationLimits, EvaluationReport, ExternalStatus,
    ExternalVerdict, HorizonReport, MappingError, MappingManifest, MappingSourceIdentity,
    MappingSourceState, ToolIdentity,
};
use tl_syntax::{
    Formula, FormulaDocument, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration,
    PropositionBinding, PropositionId, RequirementContextDocument, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId, SourceSpan, MAX_FORMULA_DOCUMENT_NODES,
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

fn overlay_nodes_with_spans(spans: [Option<SourceSpan>; 4]) -> Vec<Node> {
    overlay_nodes()
        .into_iter()
        .zip(spans)
        .map(|(mut node, span)| {
            node.span = span;
            node
        })
        .collect()
}

fn catalog() -> SignalCatalogDocument {
    catalog_with_response_name("response_within_2_cycles")
}

fn catalog_with_response_name(response_name: &str) -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(
                SignalId(10),
                "overlay_change_accepted".to_owned(),
                SignalDomain::Boolean,
            ),
            OwnedSignalDeclaration::new(
                SignalId(11),
                response_name.to_owned(),
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

fn assert_stable_v1_wire<T>(record: T, v2_schema: &str)
where
    T: serde::Serialize + serde::de::DeserializeOwned + Eq + core::fmt::Debug,
{
    let bytes = serde_json::to_vec(&record).unwrap();
    assert_eq!(serde_json::to_vec(&record).unwrap(), bytes);
    assert_eq!(serde_json::from_slice::<T>(&bytes).unwrap(), record);

    let mut relabeled: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    relabeled["schemaVersion"] = serde_json::json!(v2_schema);
    assert!(serde_json::from_value::<T>(relabeled).is_err());
}

// Trace: TC-029, FR-007-AC-5, NFR-001-AC-1
#[test]
fn legacy_records_round_trip_as_exact_v1_wires_and_refuse_v2_labels() {
    let nodes = overlay_nodes();
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), &nodes).unwrap();
    let trace = vec![vec![PropositionId(7)], vec![PropositionId(8)]];
    let evaluation = evaluate_prefix(
        formula,
        "overlay-response",
        &trace,
        "overlay-trace",
        false,
        EvaluationLimits::default(),
    )
    .unwrap();
    let horizon = analyze_horizon(formula, "overlay-response").unwrap();
    let mapping = map_to_c2po(
        formula,
        "overlay-response",
        b"overlay-response",
        MappingSourceIdentity {
            revision: "fixture".to_owned(),
            state: MappingSourceState::Clean,
        },
        None,
        100,
    )
    .unwrap();
    let external = ExternalVerdict {
        schema_version: "tl-mltl.external-verdict/v1".to_owned(),
        tool: tool(),
        formula_id: evaluation.formula_id.clone(),
        trace_id: evaluation.trace_id.clone(),
        status: ExternalStatus::Conclusive,
        value: Some(true),
        verdict_time: Some(evaluation.verdict_time),
        detail: None,
    };
    let differential = compare_external(&evaluation, external.clone());
    assert_eq!(differential.status, ComparisonStatus::Agreement);

    assert_stable_v1_wire::<EvaluationReport>(evaluation, "tl-mltl.evaluation/v2");
    assert_stable_v1_wire::<HorizonReport>(horizon, "tl-mltl.horizon/v2");
    assert_stable_v1_wire::<MappingManifest>(mapping, "tl-mltl.monitor-mapping/v2");
    assert_stable_v1_wire::<ExternalVerdict>(external, "tl-mltl.external-verdict/v2");
    assert_stable_v1_wire::<DifferentialReport>(differential, "tl-mltl.differential/v2");
}

// Trace: TC-028, FR-007-AC-4, StR-003-VC-1, NFR-001-AC-1, NFR-002-AC-4
#[test]
fn contextual_identities_change_for_independent_operation_inputs() {
    let nodes = overlay_nodes();
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), &nodes).unwrap();
    let trace = vec![vec![PropositionId(7)], vec![PropositionId(8)]];
    let context = context();
    let limits = EvaluationLimits::default();
    let evaluation = evaluate_prefix_with_context(
        formula,
        "overlay-response",
        &trace,
        "overlay-trace",
        false,
        limits,
        &catalog(),
        Some(&context),
    )
    .unwrap();
    let catalog_changed = evaluate_prefix_with_context(
        formula,
        "overlay-response",
        &trace,
        "overlay-trace",
        false,
        limits,
        &catalog_with_response_name("response_in_window"),
        Some(&context),
    )
    .unwrap();
    let trace_changed = evaluate_prefix_with_context(
        formula,
        "overlay-response",
        &[vec![PropositionId(7)]],
        "overlay-trace",
        false,
        limits,
        &catalog(),
        Some(&context),
    )
    .unwrap();
    let limit_changed = evaluate_prefix_with_context(
        formula,
        "overlay-response",
        &trace,
        "overlay-trace",
        false,
        EvaluationLimits {
            max_node_evaluations: 101,
            ..limits
        },
        &catalog(),
        Some(&context),
    )
    .unwrap();
    assert_ne!(evaluation.request_sha256, catalog_changed.request_sha256);
    assert_ne!(evaluation.result_sha256, catalog_changed.result_sha256);
    assert_ne!(evaluation.request_sha256, trace_changed.request_sha256);
    assert_ne!(evaluation.result_sha256, trace_changed.result_sha256);
    assert_ne!(evaluation.request_sha256, limit_changed.request_sha256);
    assert_ne!(evaluation.result_sha256, limit_changed.result_sha256);

    let mapping = map_to_c2po_with_context(
        formula,
        "overlay-response",
        b"overlay-response",
        MappingSourceIdentity {
            revision: "fixture-a".to_owned(),
            state: MappingSourceState::Clean,
        },
        Some(tool()),
        100,
        &catalog(),
        Some(&context),
    )
    .unwrap();
    let mapping_changed = map_to_c2po_with_context(
        formula,
        "overlay-response",
        b"overlay-response",
        MappingSourceIdentity {
            revision: "fixture-b".to_owned(),
            state: MappingSourceState::Clean,
        },
        Some(tool()),
        100,
        &catalog(),
        Some(&context),
    )
    .unwrap();
    assert_ne!(mapping.request_sha256, mapping_changed.request_sha256);
    assert_ne!(mapping.result_sha256, mapping_changed.result_sha256);

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
    let first = compare_external_with_context(&evaluation, external.clone()).unwrap();
    let mut changed_tool = external;
    changed_tool.tool.configuration_sha256 = "c".repeat(64);
    let changed = compare_external_with_context(&evaluation, changed_tool).unwrap();
    assert_ne!(first.comparison_sha256, changed.comparison_sha256);
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

#[derive(Debug, Eq, PartialEq)]
struct ContextualFormulaIdentities {
    closed_request: String,
    closed_result: String,
    prefix_request: String,
    prefix_result: String,
    horizon_request: String,
    horizon_result: String,
    mapping_request: String,
    mapping_result: String,
}

fn contextual_formula_identities(nodes: &[Node]) -> ContextualFormulaIdentities {
    let online = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), nodes).unwrap();
    let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(3), nodes).unwrap();
    let trace = vec![vec![PropositionId(7)], vec![PropositionId(8)], Vec::new()];
    let catalog = catalog();
    let context = context();
    let limits = EvaluationLimits::default();

    let closed = evaluate_closed_with_context(
        closed,
        "overlay-response",
        &trace,
        "overlay-trace",
        limits,
        &catalog,
        Some(&context),
    )
    .unwrap();
    let prefix = evaluate_prefix_with_context(
        online,
        "overlay-response",
        &trace,
        "overlay-trace",
        false,
        limits,
        &catalog,
        Some(&context),
    )
    .unwrap();
    let horizon =
        analyze_horizon_with_context(online, "overlay-response", &catalog, Some(&context)).unwrap();
    let mapping = map_to_c2po_with_context(
        online,
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

    ContextualFormulaIdentities {
        closed_request: closed.request_sha256,
        closed_result: closed.result_sha256,
        prefix_request: prefix.request_sha256,
        prefix_result: prefix.result_sha256,
        horizon_request: horizon.request_sha256,
        horizon_result: horizon.result_sha256,
        mapping_request: mapping.request_sha256,
        mapping_result: mapping.result_sha256,
    }
}

// Trace: TC-034, FR-007-AC-8, StR-003-VC-1, NFR-001-AC-1, NFR-002-AC-4
#[test]
fn diagnostic_formula_spans_do_not_change_contextual_identities() {
    let variants = [
        overlay_nodes(),
        overlay_nodes_with_spans([
            Some(SourceSpan::new(0, 0).unwrap()),
            Some(SourceSpan::new(0, 0).unwrap()),
            Some(SourceSpan::new(0, 0).unwrap()),
            Some(SourceSpan::new(0, 0).unwrap()),
        ]),
        overlay_nodes_with_spans([
            Some(SourceSpan::new(2, 4).unwrap()),
            Some(SourceSpan::new(11, 13).unwrap()),
            Some(SourceSpan::new(10, 17).unwrap()),
            Some(SourceSpan::new(1, 18).unwrap()),
        ]),
    ];
    let documents = variants
        .iter()
        .map(|nodes| {
            FormulaDocument::new(SemanticProfile::OnlinePrefixV1, NodeId(3), nodes.clone()).unwrap()
        })
        .collect::<Vec<_>>();

    assert_ne!(documents[0], documents[1]);
    assert_ne!(documents[1], documents[2]);
    assert_eq!(documents[0].semantic_view(), documents[1].semantic_view());
    assert_eq!(documents[1].semantic_view(), documents[2].semantic_view());

    let identities = variants
        .iter()
        .map(|nodes| contextual_formula_identities(nodes))
        .collect::<Vec<_>>();
    assert_eq!(identities[0], identities[1]);
    assert_eq!(identities[1], identities[2]);
}

// Trace: TC-035, FR-007-AC-9, NFR-002-AC-1
#[test]
fn oversized_borrowed_formula_is_a_contextual_identity_error() {
    let node_count = MAX_FORMULA_DOCUMENT_NODES + 1;
    let nodes = vec![Node::new(NodeKind::True); node_count];
    let root = NodeId(u32::try_from(node_count - 1).unwrap());
    let online = Formula::new(SemanticProfile::OnlinePrefixV1, root, &nodes).unwrap();
    let closed = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
    let catalog = catalog();
    let limits = EvaluationLimits {
        max_node_evaluations: 0,
        ..EvaluationLimits::default()
    };
    let is_document_limit = |detail: &str| {
        detail.contains("canonical formula identity failed")
            && detail.contains("exceeds the 100000-node limit")
    };

    assert!(matches!(
        evaluate_closed_with_context(
            closed,
            "oversized",
            &[],
            "empty",
            limits,
            &catalog,
            None
        ),
        Err(ContextualEvaluationError::Identity(detail)) if is_document_limit(&detail)
    ));
    assert!(matches!(
        evaluate_prefix_with_context(
            online,
            "oversized",
            &[],
            "empty",
            false,
            limits,
            &catalog,
            None
        ),
        Err(ContextualEvaluationError::Identity(detail)) if is_document_limit(&detail)
    ));
    assert!(matches!(
        analyze_horizon_with_context(online, "oversized", &catalog, None),
        Err(ContextualHorizonError::Identity(detail)) if is_document_limit(&detail)
    ));
    assert!(matches!(
        map_to_c2po_with_context(
            online,
            "oversized",
            b"same-bytes",
            MappingSourceIdentity {
                revision: "fixture".to_owned(),
                state: MappingSourceState::Clean,
            },
            None,
            0,
            &catalog,
            None,
        ),
        Err(MappingError::Identity(detail)) if is_document_limit(&detail)
    ));
}
