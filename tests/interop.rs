use tl_mltl::{
    compare_external, evaluate_prefix, map_to_c2po, ComparisonStatus, EvaluationLimits,
    ExternalStatus, ExternalVerdict, MappingError, ToolIdentity, TruthValue,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn future_nodes() -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(7),
        }),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 2).unwrap(),
            operand: NodeId(0),
        }),
    ]
}

fn future_formula(profile: SemanticProfile, nodes: &[Node]) -> Formula<'_> {
    Formula::new(profile, NodeId(1), nodes).unwrap()
}

fn tool() -> ToolIdentity {
    ToolIdentity {
        name: "r2u2".to_owned(),
        version: "vX.Y.Z-test-fixture".to_owned(),
        executable_sha256: "1".repeat(64),
        configuration_sha256: "2".repeat(64),
    }
}

// Trace: TC-011, FR-004-AC-1, StR-002-VC-2
#[test]
fn supported_mapping_is_stable_and_identity_preserving() {
    let nodes = future_nodes();
    let formula = future_formula(SemanticProfile::OnlinePrefixV1, &nodes);
    let run = || {
        map_to_c2po(
            formula,
            "future-seven",
            br#"{"formula":"fixture"}"#,
            "source-revision",
            "clean",
            Some(tool()),
            100,
        )
        .unwrap()
    };
    let first = run();
    assert_eq!(first, run());
    assert_eq!(first.expression, "F[0,2](p7)");
    assert_eq!(first.proposition_ids, [7]);
    assert_eq!(first.formula_id, "future-seven");
}

// Trace: TC-012, FR-004-AC-2, NFR-002-AC-1
#[test]
fn unsupported_mapping_emits_no_manifest() {
    let fixture_nodes = future_nodes();
    let formula = future_formula(SemanticProfile::ClosedTraceV1, &fixture_nodes);
    assert!(matches!(
        map_to_c2po(formula, "closed", b"formula", "source", "clean", None, 100),
        Err(MappingError::UnsupportedProfile { .. })
    ));

    let mut nodes = vec![Node::new(NodeKind::True)];
    for index in 0..=512 {
        nodes.push(Node::new(NodeKind::Not {
            operand: NodeId(index),
        }));
    }
    let deep = Formula::new(
        SemanticProfile::OnlinePrefixV1,
        NodeId((nodes.len() - 1) as u32),
        &nodes,
    )
    .unwrap();
    assert!(matches!(
        map_to_c2po(deep, "deep", b"formula", "source", "clean", None, 10_000),
        Err(MappingError::RecursionDepthExceeded { limit: 512 })
    ));
}

// Trace: TC-013, FR-004-AC-3, NFR-001-AC-1
#[test]
fn mapping_digests_and_tool_identity_detect_substitution() {
    let nodes = future_nodes();
    let formula = future_formula(SemanticProfile::OnlinePrefixV1, &nodes);
    let original = map_to_c2po(formula, "f", b"one", "source", "clean", Some(tool()), 100).unwrap();
    let changed = map_to_c2po(formula, "f", b"two", "source", "clean", Some(tool()), 100).unwrap();
    assert_ne!(original.input_sha256, changed.input_sha256);
    assert_eq!(original.output_sha256, changed.output_sha256);
    assert_eq!(original.external_tool, Some(tool()));
    assert_eq!(original.source_state, "clean");
    assert_eq!(original.input_sha256.len(), 64);
}

// Trace: TC-015, FR-005-AC-2, StR-001-VC-2
#[test]
fn differential_comparison_separates_agreement_mismatch_and_nonconclusive() {
    let nodes = future_nodes();
    let formula = future_formula(SemanticProfile::OnlinePrefixV1, &nodes);
    let reference = evaluate_prefix(
        formula,
        "formula",
        &[vec![PropositionId(7)]],
        "trace",
        false,
        EvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(reference.verdict, TruthValue::True);

    let mut external = ExternalVerdict {
        schema_version: "tl-mltl.external-verdict/v1".to_owned(),
        tool: tool(),
        formula_id: "formula".to_owned(),
        trace_id: "trace".to_owned(),
        status: ExternalStatus::Conclusive,
        value: Some(true),
        verdict_time: Some(reference.verdict_time),
        detail: None,
    };
    assert_eq!(
        compare_external(&reference, external.clone()).status,
        ComparisonStatus::Agreement
    );
    external.value = Some(false);
    assert_eq!(
        compare_external(&reference, external.clone()).status,
        ComparisonStatus::Mismatch
    );
    external.status = ExternalStatus::Unsupported;
    external.value = None;
    external.verdict_time = None;
    assert_eq!(
        compare_external(&reference, external).status,
        ComparisonStatus::NonConclusive
    );
}
