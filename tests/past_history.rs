use proptest::prelude::*;
use serde_json::json;
use tl_mltl::{
    analyze_horizon, analyze_required_history, evaluate_closed, evaluate_past,
    fixed_sample_instant, ClockBinding, ClockError, ClockSample, EvaluationError, EvaluationLimits,
    ExactNumber, ExactNumberError, HistoryError, HistoryRequirementError, HorizonError,
    OwnerHistoryState, PastEvaluationError, PastEvaluationLimits, PastEvaluationRelationInput,
    PastEvaluationReport, PastResultRelationKind, PastResultValidationError,
    PositionHistoryDocument, PositionHistorySource, PositionObservation, UnsupportedClockKind,
};
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile, SourceSpan,
};

fn formula(nodes: &[Node]) -> Formula<'_> {
    Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
}

fn event_history(values: &[(bool, bool)], revision: u64) -> PositionHistoryDocument {
    let observations = values
        .iter()
        .enumerate()
        .map(|(position, (p, q))| {
            let mut propositions = Vec::new();
            if *p {
                propositions.push(PropositionId(0));
            }
            if *q {
                propositions.push(PropositionId(1));
            }
            PositionObservation::new(u64::try_from(position).unwrap(), propositions, None)
        })
        .collect();
    PositionHistoryDocument::new(
        "history-a",
        revision,
        0,
        u64::try_from(values.len() - 1).unwrap(),
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap()
}

fn fixed_history(values: &[(bool, bool)]) -> PositionHistoryDocument {
    let epoch = ExactNumber::new(-1, 2).unwrap();
    let period = ExactNumber::new(3, 2).unwrap();
    let observations = values
        .iter()
        .enumerate()
        .map(|(position, (p, q))| {
            let mut propositions = Vec::new();
            if *p {
                propositions.push(PropositionId(0));
            }
            if *q {
                propositions.push(PropositionId(1));
            }
            let position = u64::try_from(position).unwrap();
            PositionObservation::new(
                position,
                propositions,
                Some(ClockSample {
                    instant: fixed_sample_instant(epoch, period, position).unwrap(),
                    unit: "ticks".to_owned(),
                }),
            )
        })
        .collect();
    PositionHistoryDocument::new(
        "history-fixed",
        1,
        0,
        u64::try_from(values.len() - 1).unwrap(),
        Some(ClockBinding::FixedSample {
            epoch,
            period,
            unit: "ticks".to_owned(),
        }),
        observations,
    )
    .unwrap()
}

fn evaluate(nodes: &[Node], history: &PositionHistoryDocument, anchor: u64) -> bool {
    evaluate_past(
        formula(nodes),
        "formula-a",
        history,
        anchor,
        "map-a",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap()
    .verdict
}

/// Test-only semantic implementation. It consumes only the node graph and raw
/// Boolean rows; it does not call history admission, history analysis, or the
/// production evaluator.
fn oracle(nodes: &[Node], values: &[(bool, bool)], node: NodeId, position: i128) -> bool {
    let at = |child, at| oracle(nodes, values, child, at);
    match nodes[usize::try_from(node.0).unwrap()].kind {
        NodeKind::False => false,
        NodeKind::True => true,
        NodeKind::Proposition { proposition } => usize::try_from(position)
            .ok()
            .and_then(|index| values.get(index))
            .is_some_and(|row| match proposition.0 {
                0 => row.0,
                1 => row.1,
                _ => false,
            }),
        NodeKind::Not { operand } => !at(operand, position),
        NodeKind::And { left, right } => at(left, position) && at(right, position),
        NodeKind::Or { left, right } => at(left, position) || at(right, position),
        NodeKind::Implies { left, right } => !at(left, position) || at(right, position),
        NodeKind::Equivalent { left, right } => at(left, position) == at(right, position),
        NodeKind::Once { interval, operand } => (interval.start()..=interval.end())
            .any(|offset| at(operand, position - i128::from(offset))),
        NodeKind::Historically { interval, operand } => (interval.start()..=interval.end())
            .all(|offset| at(operand, position - i128::from(offset))),
        NodeKind::StrongPrevious { operand } => at(operand, position - 1),
        NodeKind::Since {
            interval,
            left,
            right,
        } => (interval.start()..=interval.end()).any(|witness| {
            at(right, position - i128::from(witness))
                && (interval.start()..witness).all(|offset| at(left, position - i128::from(offset)))
        }),
        NodeKind::Triggered {
            interval,
            left,
            right,
        } => !(interval.start()..=interval.end()).any(|witness| {
            !at(right, position - i128::from(witness))
                && (interval.start()..witness)
                    .all(|offset| !at(left, position - i128::from(offset)))
        }),
        NodeKind::Future { .. }
        | NodeKind::Globally { .. }
        | NodeKind::Until { .. }
        | NodeKind::Release { .. } => {
            unreachable!("the independent past oracle accepts no future node")
        }
    }
}

fn unary_nodes(kind: impl FnOnce(NodeId) -> NodeKind) -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
        Node::new(kind(NodeId(1))),
    ]
}

fn binary_nodes(kind: impl FnOnce(NodeId, NodeId) -> NodeKind) -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(1),
        }),
        Node::new(kind(NodeId(0), NodeId(1))),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 192, ..ProptestConfig::default() })]

    // Trace: TC-048, TC-052, FR-011-AC-1, FR-012-AC-2
    #[test]
    fn once_and_historically_match_independent_reverse_oracle(
        values in prop::collection::vec((any::<bool>(), any::<bool>()), 1..20),
        anchor_seed in 0usize..40,
        left in 0u32..12,
        right in 0u32..12,
    ) {
        let anchor = anchor_seed % values.len();
        let (start, end) = if left <= right { (left, right) } else { (right, left) };
        let interval = Interval::new(start, end).unwrap();
        let event = event_history(&values, 1);
        let fixed = fixed_history(&values);
        for nodes in [
            unary_nodes(|operand| NodeKind::Once { interval, operand }),
            unary_nodes(|operand| NodeKind::Historically { interval, operand }),
        ] {
            let expected = oracle(&nodes, &values, NodeId(2), i128::try_from(anchor).unwrap());
            for history in [&event, &fixed] {
                prop_assert_eq!(evaluate(&nodes, history, u64::try_from(anchor).unwrap()), expected);
            }
        }
    }

    // Trace: TC-049, TC-052, FR-011-AC-2
    #[test]
    fn since_uses_inclusive_witness_and_exact_lower_bounded_left_range(
        values in prop::collection::vec((any::<bool>(), any::<bool>()), 1..20),
        anchor_seed in 0usize..40,
        left in 0u32..10,
        right in 0u32..10,
    ) {
        let anchor = anchor_seed % values.len();
        let (start, end) = if left <= right { (left, right) } else { (right, left) };
        let interval = Interval::new(start, end).unwrap();
        let nodes = binary_nodes(|p, q| NodeKind::Since { interval, left: p, right: q });
        let event = event_history(&values, 1);
        let fixed = fixed_history(&values);
        let expected = oracle(&nodes, &values, NodeId(2), i128::try_from(anchor).unwrap());
        for history in [&event, &fixed] {
            prop_assert_eq!(evaluate(&nodes, history, u64::try_from(anchor).unwrap()), expected);
        }
    }

    // Trace: TC-050, TC-052, FR-011-AC-3, FR-011-AC-5
    #[test]
    fn triggered_dual_and_strong_previous_equivalence_hold_at_every_boundary(
        values in prop::collection::vec((any::<bool>(), any::<bool>()), 1..20),
        anchor_seed in 0usize..40,
        left in 0u32..10,
        right in 0u32..10,
    ) {
        let anchor = anchor_seed % values.len();
        let anchor_u64 = u64::try_from(anchor).unwrap();
        let (start, end) = if left <= right { (left, right) } else { (right, left) };
        let interval = Interval::new(start, end).unwrap();
        let triggered = binary_nodes(|p, q| NodeKind::Triggered { interval, left: p, right: q });
        let dual = vec![
            triggered[0],
            triggered[1],
            Node::new(NodeKind::Not { operand: NodeId(0) }),
            Node::new(NodeKind::Not { operand: NodeId(1) }),
            Node::new(NodeKind::Since { interval, left: NodeId(2), right: NodeId(3) }),
            Node::new(NodeKind::Not { operand: NodeId(4) }),
        ];
        let event = event_history(&values, 1);
        let fixed = fixed_history(&values);

        let previous = unary_nodes(|operand| NodeKind::StrongPrevious { operand });
        let once = unary_nodes(|operand| NodeKind::Once {
            interval: Interval::new(1, 1).unwrap(),
            operand,
        });
        for history in [&event, &fixed] {
            prop_assert_eq!(evaluate(&triggered, history, anchor_u64), evaluate(&dual, history, anchor_u64));
            prop_assert_eq!(
                evaluate(&triggered, history, anchor_u64),
                oracle(&triggered, &values, NodeId(2), i128::try_from(anchor).unwrap())
            );
            prop_assert_eq!(evaluate(&previous, history, anchor_u64), evaluate(&once, history, anchor_u64));
        }
    }
}

// Trace: TC-049, FR-011-AC-2
#[test]
fn since_ignores_offsets_before_nonzero_lower_bound_and_accepts_both_endpoints() {
    let interval = Interval::new(2, 4).unwrap();
    let nodes = binary_nodes(|left, right| NodeKind::Since {
        interval,
        left,
        right,
    });
    // At anchor 4, q witnesses at offset 4. p is required only at offsets 2,3;
    // p at offsets 0,1 is deliberately false.
    let values = [
        (false, true),
        (true, false),
        (true, false),
        (false, false),
        (false, false),
    ];
    let history = event_history(&values, 1);
    assert!(evaluate(&nodes, &history, 4));

    // Lower endpoint witness requires no left position at all.
    let values = [(false, true), (false, false), (false, false)];
    assert!(evaluate(&nodes, &event_history(&values, 1), 2));
}

// Trace: TC-048, TC-050, FR-011-AC-1, FR-011-AC-5
#[test]
fn constants_and_boolean_nests_recurse_across_pre_origin_positions() {
    let interval = Interval::new(1, 3).unwrap();
    let nodes = vec![
        Node::new(NodeKind::True),
        Node::new(NodeKind::False),
        Node::new(NodeKind::Not { operand: NodeId(1) }),
        Node::new(NodeKind::And {
            left: NodeId(0),
            right: NodeId(2),
        }),
        Node::new(NodeKind::Historically {
            interval,
            operand: NodeId(3),
        }),
    ];
    assert!(evaluate(&nodes, &event_history(&[(false, false)], 1), 0));

    let history = event_history(&[(true, false), (false, false)], 1);
    for operand_nodes in [
        vec![Node::new(NodeKind::True)],
        vec![Node::new(NodeKind::False)],
        vec![Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        })],
        vec![
            Node::new(NodeKind::Proposition {
                proposition: PropositionId(0),
            }),
            Node::new(NodeKind::Not { operand: NodeId(0) }),
        ],
    ] {
        let operand = NodeId(u32::try_from(operand_nodes.len() - 1).unwrap());
        let mut previous = operand_nodes.clone();
        previous.push(Node::new(NodeKind::StrongPrevious { operand }));
        let mut once = operand_nodes;
        once.push(Node::new(NodeKind::Once {
            interval: Interval::new(1, 1).unwrap(),
            operand,
        }));
        for anchor in 0..=1 {
            assert_eq!(
                evaluate(&previous, &history, anchor),
                evaluate(&once, &history, anchor)
            );
        }
    }
}

// Trace: TC-051, TC-053, FR-012-AC-1
#[test]
fn history_admission_refuses_every_completeness_and_identity_dimension() {
    let clock = Some(ClockBinding::EventPosition);
    let row = |position| PositionObservation::new(position, Vec::new(), None);
    assert_eq!(
        PositionHistoryDocument::new("h", 1, 0, 0, clock.clone(), Vec::new()),
        Err(HistoryError::EmptyHistory)
    );
    assert_eq!(
        PositionHistoryDocument::new("h", 1, 1, 1, clock.clone(), vec![row(1)]),
        Err(HistoryError::OriginNotZero { actual: 1 })
    );
    assert_eq!(
        PositionHistoryDocument::new("h", 1, 0, 1, clock.clone(), vec![row(0), row(2)]),
        Err(HistoryError::Gap {
            expected: 1,
            found: 2
        })
    );
    assert_eq!(
        PositionHistoryDocument::new("h", 1, 0, 0, clock.clone(), vec![row(0), row(0)]),
        Err(HistoryError::DuplicatePosition { position: 0 })
    );
    assert_eq!(
        PositionHistoryDocument::new("h", 1, 0, 0, clock.clone(), vec![row(0), row(1), row(0)]),
        Err(HistoryError::OutOfOrder {
            previous: 1,
            current: 0
        })
    );
    assert_eq!(
        PositionHistoryDocument::new("h", 1, 0, 2, clock.clone(), vec![row(0), row(1)]),
        Err(HistoryError::ThroughPositionMismatch {
            declared: 2,
            observed: 1
        })
    );
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            clock.clone(),
            vec![PositionObservation::new(
                0,
                vec![PropositionId(2), PropositionId(1)],
                None,
            )],
        ),
        Err(HistoryError::PropositionsNotStrictlyOrdered {
            position: 0,
            previous: PropositionId(2),
            current: PropositionId(1),
        })
    );
    let too_many_propositions = (0..=100_000)
        .map(PropositionId)
        .collect::<Vec<PropositionId>>();
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            clock.clone(),
            vec![PositionObservation::new(0, too_many_propositions, None)],
        ),
        Err(HistoryError::PropositionLimitExceeded {
            position: 0,
            actual: 100_001,
            limit: 100_000,
        })
    );
    assert_eq!(
        PositionHistoryDocument::new("h", 0, 0, 0, clock.clone(), vec![row(0)]),
        Err(HistoryError::RevisionZero)
    );
    assert_eq!(
        PositionHistoryDocument::new("", 1, 0, 0, clock.clone(), vec![row(0)]),
        Err(HistoryError::EmptyHistoryIdentity)
    );
    assert_eq!(
        PositionHistoryDocument::new("h", 1, 0, 0, None, vec![row(0)]),
        Err(HistoryError::MissingClock)
    );

    let valid = event_history(&[(false, false)], 1);
    let valid_wire = serde_json::to_value(&valid).unwrap();
    assert_eq!(
        serde_json::from_value::<PositionHistoryDocument>(valid_wire.clone()).unwrap(),
        valid
    );
    let mut unknown_field = valid_wire.clone();
    unknown_field["unexpected"] = json!(true);
    assert!(serde_json::from_value::<PositionHistoryDocument>(unknown_field).is_err());
    let mut unknown_schema = valid_wire;
    unknown_schema["schemaVersion"] = json!("tl-mltl.position-history/v2");
    assert!(serde_json::from_value::<PositionHistoryDocument>(unknown_schema).is_err());
    let mut oversized_identity = serde_json::to_value(&valid).unwrap();
    oversized_identity["historyId"] = json!("x".repeat(257));
    assert!(serde_json::from_value::<PositionHistoryDocument>(oversized_identity).is_err());
    assert!(matches!(
        PositionHistoryDocument::from_declared(
            valid.history_id(),
            valid.revision(),
            "0".repeat(64),
            0,
            valid.through_position(),
            Some(ClockBinding::EventPosition),
            valid.observations().to_vec(),
        ),
        Err(HistoryError::StaleDigest { .. })
    ));
    assert_eq!(
        valid.corrected(1, 0, valid.observations().to_vec()),
        Err(HistoryError::RevisionNotAdvanced {
            previous: 1,
            proposed: 1
        })
    );
}

// Trace: TC-051, TC-056, FR-012-AC-3, FR-012-AC-4
#[test]
fn results_bind_all_dimensions_and_validate_direct_corrections() {
    let nodes = unary_nodes(|operand| NodeKind::Once {
        interval: Interval::new(0, 1).unwrap(),
        operand,
    });
    let history_v1 = event_history(&[(false, false), (true, false)], 1);
    let original = evaluate_past(
        formula(&nodes),
        "formula-a",
        &history_v1,
        1,
        "map-a",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(original.relation.kind, PastResultRelationKind::Original);
    assert!(original.relation.direct_predecessor.is_none());
    assert!(original.relation.corrected_history.is_none());
    original.validate_with_predecessor(None).unwrap();
    let bytes = serde_json::to_vec(&original).unwrap();
    assert_eq!(
        serde_json::from_slice::<PastEvaluationReport>(&bytes).unwrap(),
        original
    );

    for (pointer, replacement) in [
        ("/formulaId", json!("formula-b")),
        ("/formulaSha256", json!("1".repeat(64))),
        ("/formulaRoot", json!(1)),
        ("/operatorProfile", json!("tl-syntax.past-operators/v2")),
        ("/semanticProfile", json!("mltl.closed-trace/v1")),
        ("/history/historyId", json!("history-b")),
        ("/history/revision", json!(2)),
        ("/history/historySha256", json!("2".repeat(64))),
        ("/anchor", json!(0)),
        ("/clock/kind", json!("unsupported")),
        ("/propositionMapId", json!("map-b")),
        ("/evaluatorRevision", json!("other")),
        ("/syntaxRevision", json!("other")),
        ("/limits/maxSteps", json!(999_999)),
        ("/requiredHistory", json!(999)),
        ("/stats/steps", json!(999)),
        ("/verdict", json!(!original.verdict)),
        ("/relation/kind", json!("superseding")),
    ] {
        let mut value = serde_json::to_value(&original).unwrap();
        *value.pointer_mut(pointer).unwrap() = replacement;
        assert!(
            serde_json::from_value::<PastEvaluationReport>(value).is_err(),
            "mutation {pointer} was admitted"
        );
    }

    let history_v2 = history_v1
        .corrected(
            2,
            1,
            vec![
                PositionObservation::new(0, vec![PropositionId(0)], None),
                PositionObservation::new(1, vec![PropositionId(0)], None),
            ],
        )
        .unwrap();
    let superseding = evaluate_past(
        formula(&nodes),
        "formula-a",
        &history_v2,
        1,
        "map-a",
        2,
        PastEvaluationRelationInput::Superseding(&original),
        PastEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(
        superseding.relation.kind,
        PastResultRelationKind::Superseding
    );
    assert_eq!(
        superseding
            .relation
            .direct_predecessor
            .as_ref()
            .unwrap()
            .result_sha256,
        original.result_sha256
    );
    assert_eq!(
        superseding.relation.corrected_history.as_ref().unwrap(),
        &superseding.history
    );
    assert_ne!(superseding.result_sha256, original.result_sha256);
    superseding
        .validate_with_predecessor(Some(&original))
        .unwrap();
    assert_eq!(
        superseding.validate_with_predecessor(None),
        Err(PastResultValidationError::RelationShape)
    );
    let wrong_predecessor = evaluate_past(
        formula(&nodes),
        "formula-a",
        &history_v1,
        0,
        "map-a",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(
        superseding.validate_with_predecessor(Some(&wrong_predecessor)),
        Err(PastResultValidationError::PredecessorReferenceMismatch)
    );
    original.validate().unwrap();

    let invalidating = evaluate_past(
        formula(&nodes),
        "formula-a",
        &history_v2,
        1,
        "map-a",
        3,
        PastEvaluationRelationInput::Invalidating(&original),
        PastEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(
        invalidating.relation.kind,
        PastResultRelationKind::Invalidating
    );
    invalidating
        .validate_with_predecessor(Some(&original))
        .unwrap();
    assert_eq!(
        evaluate_past(
            formula(&nodes),
            "formula-a",
            &history_v2,
            1,
            "map-a",
            1,
            PastEvaluationRelationInput::Superseding(&original),
            PastEvaluationLimits::default(),
        ),
        Err(PastEvaluationError::ResultRevisionInvalid)
    );
}

// Trace: FR-012-AC-3, FR-012-AC-4, FR-050-AC-1
#[test]
fn persisted_past_result_refuses_typed_identity_work_and_lineage_mutations() {
    let nodes = unary_nodes(|operand| NodeKind::Once {
        interval: Interval::new(0, 1).unwrap(),
        operand,
    });
    let first_history = event_history(&[(false, false), (true, false)], 1);
    let original = evaluate_past(
        formula(&nodes),
        "formula-a",
        &first_history,
        1,
        "map-a",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    let corrected_history = first_history
        .corrected(2, 1, first_history.observations().to_vec())
        .unwrap();
    let successor = evaluate_past(
        formula(&nodes),
        "formula-a",
        &corrected_history,
        1,
        "map-a",
        2,
        PastEvaluationRelationInput::Superseding(&original),
        PastEvaluationLimits::default(),
    )
    .unwrap();
    original.validate().unwrap();
    successor
        .validate_with_predecessor(Some(&original))
        .unwrap();

    macro_rules! refuses {
        ($base:expr, $mutation:expr, $expected:expr) => {{
            let mut report = $base.clone();
            $mutation(&mut report);
            assert_eq!(report.validate(), Err($expected));
        }};
    }

    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.result_revision = 0,
        PastResultValidationError::ResultRevisionZero
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.formula_id.clear(),
        PastResultValidationError::IdentityMismatch { field: "formulaId" }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.history.history_id.clear(),
        PastResultValidationError::IdentityMismatch { field: "historyId" }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.proposition_map_id.clear(),
        PastResultValidationError::IdentityMismatch {
            field: "propositionMapId"
        }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.evaluator_revision.clear(),
        PastResultValidationError::IdentityMismatch {
            field: "evaluatorRevision"
        }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.result_sha256 = "bad".into(),
        PastResultValidationError::MalformedDigest {
            field: "resultSha256"
        }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.formula_sha256 = "bad".into(),
        PastResultValidationError::MalformedDigest {
            field: "formulaSha256"
        }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.history.history_sha256 = "bad".into(),
        PastResultValidationError::MalformedDigest {
            field: "historySha256"
        }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.history.revision = 0,
        PastResultValidationError::IdentityMismatch {
            field: "historyRevision"
        }
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.limits.max_steps = u64::MAX,
        PastResultValidationError::LimitsNotClamped
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.history.through_position = u64::MAX,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.stats.temporal_iterations = u64::MAX,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.stats.steps = 0,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.stats.node_evaluations = 0,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.stats.steps += 1,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.limits.max_steps = r.stats.steps - 1,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.stats.input_positions = 0,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.limits.max_input_positions = r.stats.input_positions - 1,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.limits.max_recursion_depth = 0,
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| {
            r.stats.max_recursion_depth = u32::try_from(r.stats.node_evaluations).unwrap();
        },
        PastResultValidationError::StatisticsOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.history.origin_position = 1,
        PastResultValidationError::AnchorOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.anchor = 2,
        PastResultValidationError::AnchorOutOfRange
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.relation.corrected_history = Some(r.history.clone()),
        PastResultValidationError::RelationShape
    );
    refuses!(
        original,
        |r: &mut PastEvaluationReport| r.verdict = !r.verdict,
        PastResultValidationError::StaleResultDigest
    );

    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r.relation.direct_predecessor = None,
        PastResultValidationError::RelationShape
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r.relation.corrected_history = None,
        PastResultValidationError::RelationShape
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r.relation.corrected_history.as_mut().unwrap().revision = 1,
        PastResultValidationError::CorrectedHistoryMismatch
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .result_revision = 2,
        PastResultValidationError::PredecessorNotEarlier
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| {
            r.relation
                .direct_predecessor
                .as_mut()
                .unwrap()
                .result_revision = 0;
        },
        PastResultValidationError::PredecessorNotEarlier
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| {
            r.relation
                .direct_predecessor
                .as_mut()
                .unwrap()
                .history_revision = 0;
        },
        PastResultValidationError::PredecessorNotEarlier
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| {
            r.relation
                .direct_predecessor
                .as_mut()
                .unwrap()
                .history_revision = r.history.revision;
        },
        PastResultValidationError::PredecessorNotEarlier
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r.relation.direct_predecessor.as_mut().unwrap().history_id =
            "elsewhere".into(),
        PastResultValidationError::PredecessorContextMismatch
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| {
            r.relation.direct_predecessor.as_mut().unwrap().anchor = 0;
        },
        PastResultValidationError::PredecessorContextMismatch
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .result_sha256 = "bad".into(),
        PastResultValidationError::MalformedDigest {
            field: "predecessorResultSha256"
        }
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .history_sha256 = "bad".into(),
        PastResultValidationError::MalformedDigest {
            field: "predecessorHistorySha256"
        }
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .result_sha256 = r.result_sha256.clone(),
        PastResultValidationError::SelfPredecessor
    );
    refuses!(
        successor,
        |r: &mut PastEvaluationReport| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .history_sha256 = r.history.history_sha256.clone(),
        PastResultValidationError::PredecessorNotEarlier
    );
}

// Trace: TC-052, FR-012-AC-2
#[test]
fn required_history_uses_checked_recursive_equations_and_ignores_spans() {
    let i24 = Interval::new(2, 4).unwrap();
    let i37 = Interval::new(3, 7).unwrap();
    let i12 = Interval::new(1, 2).unwrap();
    let nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(1),
        }),
        Node::new(NodeKind::Once {
            interval: i24,
            operand: NodeId(0),
        }),
        Node::new(NodeKind::StrongPrevious { operand: NodeId(2) }),
        Node::new(NodeKind::Since {
            interval: i37,
            left: NodeId(2),
            right: NodeId(3),
        }),
        Node::new(NodeKind::Historically {
            interval: i12,
            operand: NodeId(4),
        }),
    ];
    let report = analyze_required_history(formula(&nodes), "formula-a").unwrap();
    assert_eq!(report.required_positions, 14);
    assert_eq!(report.unit, "positions");
    report.validate().unwrap();
    let wire = serde_json::to_value(&report).unwrap();
    assert_eq!(
        serde_json::from_value::<tl_mltl::HistoryRequirementReport>(wire.clone()).unwrap(),
        report
    );
    for (field, replacement) in [
        ("formulaSha256", json!("bad")),
        ("operatorProfile", json!("tl-syntax.past-operators/v2")),
        ("semanticProfile", json!("mltl.online-prefix/v1")),
        ("unit", json!("seconds")),
    ] {
        let mut mutated = wire.clone();
        mutated[field] = replacement;
        assert!(serde_json::from_value::<tl_mltl::HistoryRequirementReport>(mutated).is_err());
    }

    let spanned: Vec<Node> = nodes
        .iter()
        .enumerate()
        .map(|(index, node)| {
            let start = u32::try_from(index * 2).unwrap();
            Node::with_span(node.kind, SourceSpan::new(start, start + 1).unwrap())
        })
        .collect();
    let spanned_report = analyze_required_history(formula(&spanned), "formula-a").unwrap();
    assert_eq!(spanned_report.required_positions, report.required_positions);
    assert_eq!(spanned_report.formula_sha256, report.formula_sha256);

    let future_profile_nodes = [Node::new(NodeKind::True)];
    let future = Formula::new(
        SemanticProfile::ClosedTraceV1,
        NodeId(0),
        &future_profile_nodes,
    )
    .unwrap();
    assert!(matches!(
        analyze_required_history(future, "formula-a"),
        Err(HistoryRequirementError::UnsupportedProfile { .. })
    ));
}

// Trace: TC-053, FR-011-AC-4, FR-013-AC-1
#[test]
fn future_analysis_and_evaluation_refuse_the_closed_past_profile() {
    let nodes = [Node::new(NodeKind::True)];
    let past = formula(&nodes);
    assert!(matches!(
        analyze_horizon(past, "formula-a"),
        Err(HorizonError::UnsupportedProfile { .. })
    ));
    assert!(matches!(
        evaluate_closed(
            past,
            "formula-a",
            &[],
            "trace-a",
            EvaluationLimits::default(),
        ),
        Err(EvaluationError::UnsupportedProfile { .. })
    ));
}

// Trace: TC-052, TC-053, FR-012-AC-2, FR-012-AC-5
#[test]
fn evaluation_enforces_each_resource_limit_without_a_fallback_verdict() {
    let interval = Interval::new(0, 5).unwrap();
    let nodes = unary_nodes(|operand| NodeKind::Once { interval, operand });
    let history = event_history(&[(false, false); 8], 1);
    let call = |limits| {
        evaluate_past(
            formula(&nodes),
            "formula-a",
            &history,
            7,
            "map-a",
            1,
            PastEvaluationRelationInput::Original,
            limits,
        )
    };
    assert_eq!(
        call(PastEvaluationLimits {
            max_temporal_span: 5,
            ..PastEvaluationLimits::default()
        }),
        Err(PastEvaluationError::TemporalSpanExceeded {
            requested: 6,
            limit: 5
        })
    );
    assert_eq!(
        call(PastEvaluationLimits {
            max_steps: 1,
            ..PastEvaluationLimits::default()
        }),
        Err(PastEvaluationError::StepLimitExceeded { limit: 1 })
    );
    assert_eq!(
        call(PastEvaluationLimits {
            max_recursion_depth: 0,
            ..PastEvaluationLimits::default()
        }),
        Err(PastEvaluationError::RecursionDepthExceeded { limit: 0 })
    );
    assert_eq!(
        call(PastEvaluationLimits {
            max_input_positions: 7,
            ..PastEvaluationLimits::default()
        }),
        Err(PastEvaluationError::InputPositionLimitExceeded {
            requested: 8,
            limit: 7
        })
    );

    let previous = vec![
        Node::new(NodeKind::True),
        Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
    ];
    assert_eq!(
        evaluate_past(
            formula(&previous),
            "formula-a",
            &history,
            0,
            "map-a",
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits {
                max_temporal_span: 0,
                ..PastEvaluationLimits::default()
            },
        ),
        Err(PastEvaluationError::TemporalSpanExceeded {
            requested: 1,
            limit: 0
        })
    );
}

// Trace: TC-053, TC-056, FR-012-AC-5
#[test]
fn clock_admission_is_exact_and_owner_nonvalues_are_preserved() {
    let values = [(false, false), (true, false), (false, true)];
    let fixed = fixed_history(&values);
    fixed.validate().unwrap();
    assert_eq!(
        fixed_sample_instant(
            ExactNumber::new(-1, 2).unwrap(),
            ExactNumber::new(3, 2).unwrap(),
            2,
        )
        .unwrap(),
        ExactNumber::new(5, 2).unwrap()
    );
    assert_eq!(
        fixed_sample_instant(
            ExactNumber::new(i64::MAX, 1).unwrap(),
            ExactNumber::new(i64::MAX, 1).unwrap(),
            u64::MAX,
        ),
        Err(ExactNumberError::ArithmeticOverflow)
    );
    assert!(
        serde_json::from_value::<ExactNumber>(json!({"numerator": 2, "denominator": 2})).is_err()
    );

    let epoch = ExactNumber::new(0, 1).unwrap();
    let period = ExactNumber::new(1, 1).unwrap();
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            Some(ClockBinding::FixedSample {
                epoch,
                period,
                unit: "u".to_owned()
            }),
            vec![PositionObservation::new(0, Vec::new(), None)],
        ),
        Err(HistoryError::IncompleteSample { position: 0 })
    );
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            Some(ClockBinding::FixedSample {
                epoch,
                period,
                unit: "u".to_owned(),
            }),
            vec![PositionObservation::new(
                0,
                Vec::new(),
                Some(ClockSample {
                    instant: epoch,
                    unit: "other".to_owned(),
                }),
            )],
        ),
        Err(HistoryError::Clock(ClockError::SampleUnitMismatch {
            position: 0
        }))
    );
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            Some(ClockBinding::FixedSample {
                epoch,
                period,
                unit: "u".to_owned(),
            }),
            vec![PositionObservation::new(
                0,
                Vec::new(),
                Some(ClockSample {
                    instant: ExactNumber::new(1, 1).unwrap(),
                    unit: "u".to_owned(),
                }),
            )],
        ),
        Err(HistoryError::Clock(ClockError::SampleInstantMismatch {
            position: 0,
            expected: epoch,
            actual: ExactNumber::new(1, 1).unwrap(),
        }))
    );
    let maximal = ExactNumber::new(i64::MAX, 1).unwrap();
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            1,
            Some(ClockBinding::FixedSample {
                epoch: maximal,
                period: maximal,
                unit: "u".to_owned(),
            }),
            vec![
                PositionObservation::new(
                    0,
                    Vec::new(),
                    Some(ClockSample {
                        instant: maximal,
                        unit: "u".to_owned(),
                    }),
                ),
                PositionObservation::new(
                    1,
                    Vec::new(),
                    Some(ClockSample {
                        instant: maximal,
                        unit: "u".to_owned(),
                    }),
                ),
            ],
        ),
        Err(HistoryError::Clock(ClockError::ArithmeticOverflow {
            position: 1
        }))
    );
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            Some(ClockBinding::EventPosition),
            vec![PositionObservation::new(
                0,
                Vec::new(),
                Some(ClockSample {
                    instant: epoch,
                    unit: "u".to_owned()
                }),
            )],
        ),
        Err(HistoryError::Clock(ClockError::UnexpectedSample {
            position: 0
        }))
    );
    assert_eq!(
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            Some(ClockBinding::FixedSample {
                epoch,
                period: ExactNumber::new(-1, 1).unwrap(),
                unit: "u".to_owned(),
            }),
            vec![PositionObservation::new(0, Vec::new(), None)],
        ),
        Err(HistoryError::Clock(ClockError::NonPositivePeriod))
    );
    for clock_kind in [
        UnsupportedClockKind::Dense,
        UnsupportedClockKind::Timestamped,
        UnsupportedClockKind::Duration,
        UnsupportedClockKind::Rounded,
        UnsupportedClockKind::Resampled,
        UnsupportedClockKind::WallClock,
    ] {
        assert_eq!(
            PositionHistoryDocument::new(
                "h",
                1,
                0,
                0,
                Some(ClockBinding::Unsupported { clock_kind }),
                vec![PositionObservation::new(0, Vec::new(), None)],
            ),
            Err(HistoryError::Clock(ClockError::Unsupported {
                kind: clock_kind
            }))
        );
    }

    let nodes = [Node::new(NodeKind::True)];
    for state in [
        OwnerHistoryState::Incomplete,
        OwnerHistoryState::Unavailable,
        OwnerHistoryState::Unsupported,
        OwnerHistoryState::Failed,
        OwnerHistoryState::Refused,
        OwnerHistoryState::Conflict,
    ] {
        assert_eq!(
            evaluate_past(
                formula(&nodes),
                "formula-a",
                PositionHistorySource::NonValue(state),
                0,
                "map-a",
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits::default(),
            ),
            Err(PastEvaluationError::OwnerStatePreserved { state })
        );
    }
}

// Trace: TC-051, TC-056, FR-012-AC-4
#[test]
fn anchors_are_explicit_final_originals_and_silence_advances_nothing() {
    let nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
    ];
    let history = event_history(&[(true, false), (false, false)], 1);
    let at_zero = evaluate_past(
        formula(&nodes),
        "formula-a",
        &history,
        0,
        "map-a",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    let at_one = evaluate_past(
        formula(&nodes),
        "formula-a",
        &history,
        1,
        "map-a",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    assert!(!at_zero.verdict);
    assert!(at_one.verdict);
    assert_ne!(at_zero.result_sha256, at_one.result_sha256);
    assert_eq!(at_zero.relation.kind, PastResultRelationKind::Original);
    assert_eq!(at_one.relation.kind, PastResultRelationKind::Original);
    assert_eq!(at_zero.anchor, 0);
    assert_eq!(at_one.anchor, 1);

    assert_eq!(
        evaluate_past(
            formula(&nodes),
            "formula-a",
            &history,
            2,
            "map-a",
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits::default(),
        ),
        Err(PastEvaluationError::AnchorOutOfRange {
            anchor: 2,
            through: 1
        })
    );
}
