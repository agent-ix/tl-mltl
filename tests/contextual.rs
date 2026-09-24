use tl_mltl::wire::{
    self, CommandDocument, CommandSchemaVersion, Operation, OwnerLimits, OwnerReadErrorCode,
    TraceDocument, TraceSchemaVersion, ValidatedCommand, ValidatedTrace,
};
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

fn stable_future_mapping_payload<T: serde::Serialize>(manifest: T) -> serde_json::Value {
    let mut value = serde_json::to_value(manifest).unwrap();
    let fields = value.as_object_mut().unwrap();
    for field in [
        "adapterVersion",
        "syntaxRevision",
        "requestSha256",
        "resultSha256",
    ] {
        fields.remove(field);
    }
    value
}

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

// Trace: TC-029, FR-007-AC-5, NFR-001-AC-1
#[test]
fn owner_command_round_trip_and_expected_identity_are_strict() {
    let formula = FormulaDocument::new(
        SemanticProfile::ClosedTraceV1,
        NodeId(0),
        vec![Node::new(NodeKind::Proposition {
            proposition: PropositionId(7),
        })],
    )
    .unwrap();
    let trace = TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: "ordered".to_owned(),
        closed: true,
        instants: vec![vec![PropositionId(7), PropositionId(8)]],
    };
    let document = CommandDocument {
        schema_version: CommandSchemaVersion::V1,
        operation: Operation::Evaluate,
        formula_id: "p".to_owned(),
        formula,
        trace: Some(trace),
    };
    let limits = OwnerLimits::owner_max();
    let owner = wire::command::derive(&document, limits).unwrap();
    let admitted = wire::command::read(owner.bytes(), &document, limits).unwrap();
    assert_eq!(admitted.document(), &document);
    assert_eq!(admitted.bytes(), owner.bytes());
    assert_eq!(admitted.usage().formula_nodes, 1);
    assert_eq!(admitted.usage().positions, 1);

    let mut other = document.clone();
    other.formula_id = "other".to_owned();
    assert_eq!(
        wire::command::read(owner.bytes(), &other, limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );
    assert_eq!(
        ValidatedCommand::from_json_bytes(&[&b" "[..], owner.bytes()].concat(), limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::NonCanonical
    );
    other.formula_id.clear();
    assert_eq!(
        wire::command::derive(&other, limits).unwrap_err().code(),
        OwnerReadErrorCode::InvalidCombination
    );
    other.formula_id = "x".repeat(257);
    assert_eq!(
        wire::command::derive(&other, limits).unwrap_err().field(),
        "formulaId"
    );
    let mut horizon_only = document.clone();
    horizon_only.operation = Operation::Analyze;
    horizon_only.trace = None;
    assert_eq!(
        wire::command::derive(&horizon_only, limits)
            .unwrap()
            .usage()
            .positions,
        0
    );
    assert_eq!(
        wire::command::derive(
            &document,
            OwnerLimits {
                max_output_bytes: 1,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );
}

// Trace: TC-029, FR-007-AC-5, NFR-001-AC-1
#[test]
fn owner_trace_refuses_unsorted_and_over_budget_observations() {
    let limits = OwnerLimits::owner_max();
    let trace = TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: "trace".to_owned(),
        closed: false,
        instants: vec![vec![PropositionId(7), PropositionId(8)]],
    };
    let owner = wire::trace::derive(&trace, limits).unwrap();
    let admitted = ValidatedTrace::from_json_bytes(owner.bytes(), limits).unwrap();
    assert_eq!(admitted.document(), &trace);
    assert_eq!(admitted.canonical_json_bytes(), owner.bytes());
    assert_eq!(
        wire::trace::read(owner.bytes(), &trace, limits).unwrap(),
        admitted
    );

    let mut unsorted = trace.clone();
    unsorted.instants[0].reverse();
    assert_eq!(
        wire::trace::derive(&unsorted, limits).unwrap_err().code(),
        OwnerReadErrorCode::InvalidCombination
    );
    assert_eq!(
        wire::trace::derive(
            &trace,
            OwnerLimits {
                max_positions: 0,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );
    assert_eq!(
        ValidatedTrace::from_json_bytes(&[0xff], limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::InvalidUtf8
    );
}

// Trace: FR-050-AC-1, NFR-001-AC-1
#[test]
fn owner_semantic_limits_admit_the_boundary_and_type_one_over() {
    use tl_mltl::wire::common::{produce, OwnerUsage};

    macro_rules! boundary {
        ($limit:ident, $usage:ident, $field:literal) => {{
            let limits = OwnerLimits {
                $limit: 1,
                ..OwnerLimits::owner_max()
            };
            let usage = OwnerUsage {
                $usage: 1,
                ..OwnerUsage::default()
            };
            assert!(produce((), usage, limits).is_ok(), "{} at limit", $field);
            let over = OwnerUsage { $usage: 2, ..usage };
            let error = produce((), over, limits).unwrap_err();
            assert_eq!(error.code(), OwnerReadErrorCode::ResourceIncomplete);
            assert_eq!(error.field(), $field);
        }};
    }

    boundary!(max_formula_nodes, formula_nodes, "formulaNodes");
    boundary!(max_formula_depth, formula_depth, "formulaDepth");
    boundary!(max_positions, positions, "positions");
    boundary!(max_propositions, propositions, "propositions");
    boundary!(max_support, support, "support");
    boundary!(max_history_span, history_span, "historySpan");
    boundary!(max_evaluation_steps, evaluation_steps, "evaluationSteps");
    boundary!(max_recursion_depth, recursion_depth, "recursionDepth");
}

// Trace: FR-050-AC-1, NFR-001-AC-1
#[test]
fn owner_trace_limits_bind_wire_shape_and_expected_identity() {
    let trace = TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: "edge".to_owned(),
        closed: true,
        instants: vec![vec![PropositionId(7), PropositionId(8)]],
    };
    let max = OwnerLimits::owner_max();
    let bytes = wire::trace::derive(&trace, max).unwrap().bytes().to_vec();
    let usage = ValidatedTrace::from_json_bytes(&bytes, max)
        .unwrap()
        .usage();
    let bounded = OwnerLimits {
        max_input_bytes: bytes.len(),
        max_depth: usage.depth,
        max_string_bytes: usage.string_bytes,
        max_visited_fields: usage.visited_fields,
        ..max
    };
    assert_eq!(
        wire::trace::read(&bytes, &trace, bounded)
            .unwrap()
            .document(),
        &trace
    );
    for (limits, field) in [
        (
            OwnerLimits {
                max_input_bytes: bytes.len() - 1,
                ..bounded
            },
            "inputBytes",
        ),
        (
            OwnerLimits {
                max_depth: usage.depth - 1,
                ..bounded
            },
            "depth",
        ),
        (
            OwnerLimits {
                max_string_bytes: usage.string_bytes - 1,
                ..bounded
            },
            "stringBytes",
        ),
        (
            OwnerLimits {
                max_visited_fields: usage.visited_fields - 1,
                ..bounded
            },
            "visitedFields",
        ),
    ] {
        let error = ValidatedTrace::from_json_bytes(&bytes, limits).unwrap_err();
        assert_eq!(error.code(), OwnerReadErrorCode::ResourceIncomplete);
        assert_eq!(error.field(), field);
    }

    let mut other = trace.clone();
    other.trace_id = "other".to_owned();
    let error = wire::trace::read(&bytes, &other, bounded).unwrap_err();
    assert_eq!(error.code(), OwnerReadErrorCode::ExpectedMismatch);
    assert_eq!(error.field(), "trace");

    let mut nameless = trace.clone();
    nameless.trace_id.clear();
    let error = wire::trace::derive(&nameless, max).unwrap_err();
    assert_eq!(error.code(), OwnerReadErrorCode::InvalidCombination);
    assert_eq!(error.field(), "traceId");
}

// Trace: TC-029, FR-007-AC-5, NFR-001-AC-1
#[test]
fn owner_trace_binds_expected_identity_and_distinct_resource_ceilings() {
    let limits = OwnerLimits::owner_max();
    let trace = TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: "bound-trace".to_owned(),
        closed: false,
        instants: vec![vec![PropositionId(7), PropositionId(8)]],
    };
    let canonical = wire::trace::derive(&trace, limits).unwrap();

    let mut foreign = trace.clone();
    foreign.trace_id = "foreign-trace".to_owned();
    let mismatch = wire::trace::read(canonical.bytes(), &foreign, limits).unwrap_err();
    assert_eq!(mismatch.code(), OwnerReadErrorCode::ExpectedMismatch);
    assert_eq!(mismatch.field(), "trace");

    let mut invalid = trace.clone();
    invalid.trace_id = "x".repeat(257);
    let refusal = wire::trace::derive(&invalid, limits).unwrap_err();
    assert_eq!(refusal.code(), OwnerReadErrorCode::InvalidCombination);
    assert_eq!(refusal.field(), "traceId");

    let proposition_limit = OwnerLimits {
        max_propositions: 1,
        ..limits
    };
    let refusal = wire::trace::derive(&trace, proposition_limit).unwrap_err();
    assert_eq!(refusal.code(), OwnerReadErrorCode::ResourceIncomplete);
    assert_eq!(refusal.field(), "propositions");

    let input_limit = OwnerLimits {
        max_input_bytes: canonical.bytes().len() - 1,
        ..limits
    };
    let refusal = ValidatedTrace::from_json_bytes(canonical.bytes(), input_limit).unwrap_err();
    assert_eq!(refusal.code(), OwnerReadErrorCode::ResourceIncomplete);
    assert_eq!(refusal.field(), "inputBytes");
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

// Trace: TC-029, FR-007-AC-5, FR-050-AC-1
#[test]
fn evaluation_schema_refusal_agrees_across_json_deserializer_forms() {
    let nodes = overlay_nodes();
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), &nodes).unwrap();
    let report = evaluate_prefix(
        formula,
        "schema-check",
        &[vec![PropositionId(7)]],
        "schema-trace",
        false,
        EvaluationLimits::default(),
    )
    .unwrap();
    let valid = serde_json::to_value(&report).unwrap();
    assert_eq!(
        serde_json::from_value::<EvaluationReport>(valid.clone()).unwrap(),
        report
    );
    assert_eq!(
        serde_json::from_slice::<EvaluationReport>(&serde_json::to_vec(&valid).unwrap()).unwrap(),
        report
    );

    let mut invalid = valid;
    invalid["schemaVersion"] = serde_json::json!("tl-mltl.evaluation/v2");
    assert!(serde_json::from_value::<EvaluationReport>(invalid.clone()).is_err());
    assert!(
        serde_json::from_slice::<EvaluationReport>(&serde_json::to_vec(&invalid).unwrap()).is_err()
    );
}

// Trace: TC-163, FR-038-AC-2
#[test]
fn past_mapping_addition_preserved_bounded_future_behavioral_payload() {
    let nodes = overlay_nodes();
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), &nodes).unwrap();
    let source = MappingSourceIdentity {
        revision: "fixture-source".to_owned(),
        state: MappingSourceState::Clean,
    };
    let v1 = map_to_c2po(
        formula,
        "overlay-response",
        b"overlay-response",
        source.clone(),
        None,
        100,
    )
    .unwrap();
    let v2 = map_to_c2po_with_context(
        formula,
        "overlay-response",
        b"overlay-response",
        source,
        None,
        100,
        &catalog(),
        Some(&context()),
    )
    .unwrap();
    let actual = serde_json::json!({
        "v1": stable_future_mapping_payload(v1),
        "v2": stable_future_mapping_payload(v2),
    });
    let golden: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/legacy-future-mapping-stable.json")).unwrap();
    assert_eq!(actual, golden);

    // The commit that added past mapping changed only these two functions'
    // visibility in the legacy implementation. The fixture above keeps the
    // rendered payload checked at the current release graph as well.
    let root = env!("CARGO_MANIFEST_DIR");
    let source_at = |revision: &str| {
        let output = std::process::Command::new("git")
            .current_dir(root)
            .args(["show", &format!("{revision}:src/mapping/legacy.rs")])
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    };
    let before = source_at("9c3d99907c64ccc5ccf59f79bbc56bc9cf4233b1");
    let after = source_at("878e4f8c64b0f19fbd03978cc3cee814fc3367e8")
        .replace("pub(crate) fn is_c2po_identifier", "fn is_c2po_identifier")
        .replace("pub(crate) fn sha256_hex", "fn sha256_hex");
    assert_eq!(before, after, "past mapping changed legacy future logic");
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
    let mut semantic_nodes = overlay_nodes();
    semantic_nodes[3] = Node::new(NodeKind::And {
        left: NodeId(0),
        right: NodeId(2),
    });
    let semantic_formula =
        Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), &semantic_nodes).unwrap();
    let formula_changed = evaluate_prefix_with_context(
        semantic_formula,
        "overlay-response",
        &trace,
        "overlay-trace",
        false,
        limits,
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
    assert_ne!(evaluation.request_sha256, formula_changed.request_sha256);
    assert_ne!(evaluation.result_sha256, formula_changed.result_sha256);

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

    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(3), &variants[0]).unwrap();
    let map = |formula_bytes: &[u8]| {
        map_to_c2po_with_context(
            formula,
            "overlay-response",
            formula_bytes,
            MappingSourceIdentity {
                revision: "fixture".to_owned(),
                state: MappingSourceState::Clean,
            },
            None,
            100,
            &catalog(),
            Some(&context()),
        )
        .unwrap()
    };
    let original_bytes = map(b"overlay-response");
    let changed_bytes = map(b"overlay-response-with-distinct-bytes");
    assert_ne!(original_bytes.request_sha256, changed_bytes.request_sha256);
    assert_ne!(original_bytes.result_sha256, changed_bytes.result_sha256);
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
