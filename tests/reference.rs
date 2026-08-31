use tl_mltl::{
    analyze_horizon, evaluate_closed, evaluate_closed_at, evaluate_prefix, evaluate_prefix_at,
    EvaluationError, EvaluationLimits, TruthValue, TL_SYNTAX_CORPUS_REVISION,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn formula<'a>(profile: SemanticProfile, nodes: &'a [Node]) -> Formula<'a> {
    Formula::new(profile, NodeId((nodes.len() - 1) as u32), nodes).unwrap()
}

fn p(id: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(id),
    })
}

fn default_limits() -> EvaluationLimits {
    EvaluationLimits::default()
}

fn proposition_at(trace: &[Vec<PropositionId>], time: u64, proposition: u32) -> bool {
    usize::try_from(time)
        .ok()
        .and_then(|index| trace.get(index))
        .is_some_and(|instant| instant.binary_search(&PropositionId(proposition)).is_ok())
}

fn trace_from_bits(length: usize, bits: usize) -> Vec<Vec<PropositionId>> {
    (0..length)
        .map(|time| {
            (0..=1)
                .filter(|proposition| bits & (1 << (time * 2 + proposition)) != 0)
                .map(|proposition| PropositionId(proposition as u32))
                .collect()
        })
        .collect()
}

// Trace: TC-001, FR-001-AC-1, StR-001-VC-1
#[test]
fn evaluates_boolean_and_unary_temporal_primitives() {
    let interval = Interval::new(1, 2).unwrap();
    let nodes = [
        p(0),
        Node::new(NodeKind::Future {
            interval,
            operand: NodeId(0),
        }),
        Node::new(NodeKind::Not { operand: NodeId(1) }),
    ];
    let false_future = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &nodes),
        "not-future",
        &[vec![], vec![], vec![]],
        "no-p",
        default_limits(),
    )
    .unwrap();
    assert_eq!(false_future.verdict, TruthValue::True);

    let true_future = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &nodes[..2]),
        "future",
        &[vec![], vec![PropositionId(0)]],
        "p-at-one",
        default_limits(),
    )
    .unwrap();
    assert_eq!(true_future.verdict, TruthValue::True);
}

// Trace: TC-002, FR-001-AC-1
#[test]
fn evaluates_until_release_and_closed_boundaries() {
    let interval = Interval::new(1, 2).unwrap();
    let until_nodes = [
        p(0),
        p(1),
        Node::new(NodeKind::Until {
            interval,
            left: NodeId(0),
            right: NodeId(1),
        }),
    ];
    let report = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &until_nodes),
        "until",
        &[vec![PropositionId(0)], vec![PropositionId(1)]],
        "witness",
        default_limits(),
    )
    .unwrap();
    assert_eq!(report.verdict, TruthValue::True);

    let lower_bound_window = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &until_nodes),
        "until-lower-bound",
        &[vec![], vec![PropositionId(1)]],
        "left-before-window-is-irrelevant",
        default_limits(),
    )
    .unwrap();
    assert_eq!(lower_bound_window.verdict, TruthValue::True);

    let release_nodes = [
        Node::new(NodeKind::False),
        Node::new(NodeKind::True),
        Node::new(NodeKind::Release {
            interval,
            left: NodeId(0),
            right: NodeId(1),
        }),
    ];
    let report = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &release_nodes),
        "release",
        &[],
        "empty",
        default_limits(),
    )
    .unwrap();
    assert_eq!(report.verdict, TruthValue::True);
    assert_eq!(report.observed_through, None);

    let release_nodes = [
        p(0),
        p(1),
        Node::new(NodeKind::Release {
            interval,
            left: NodeId(0),
            right: NodeId(1),
        }),
    ];
    let release = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &release_nodes),
        "release-lower-bound",
        &[vec![PropositionId(0)], vec![]],
        "right-false-in-window",
        default_limits(),
    )
    .unwrap();
    assert_eq!(release.verdict, TruthValue::False);

    let nested_nodes = [
        p(0),
        p(1),
        Node::new(NodeKind::Until {
            interval,
            left: NodeId(0),
            right: NodeId(1),
        }),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 0).unwrap(),
            operand: NodeId(2),
        }),
    ];
    let nested = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &nested_nodes),
        "nested-until",
        &[vec![], vec![PropositionId(1)]],
        "nested-window",
        default_limits(),
    )
    .unwrap();
    assert_eq!(nested.verdict, TruthValue::True);
}

// Trace: TC-002, FR-001-AC-1
#[test]
fn temporal_lower_bound_windows_match_an_independent_exhaustive_oracle() {
    for start in 0..=3 {
        for end in start..=3 {
            let interval = Interval::new(start, end).unwrap();
            let globally_nodes = [
                p(0),
                Node::new(NodeKind::Globally {
                    interval,
                    operand: NodeId(0),
                }),
            ];
            let until_nodes = [
                p(0),
                p(1),
                Node::new(NodeKind::Until {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ];
            let release_nodes = [
                p(0),
                p(1),
                Node::new(NodeKind::Release {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ];
            for trace_length in 0..=5 {
                for bits in 0..(1_usize << (trace_length * 2)) {
                    let trace = trace_from_bits(trace_length, bits);
                    for verdict_time in 0..=2 {
                        let globally_expected = (start..=end).all(|offset| {
                            proposition_at(&trace, verdict_time + u64::from(offset), 0)
                        });
                        let until_expected = (start..=end).any(|witness| {
                            proposition_at(&trace, verdict_time + u64::from(witness), 1)
                                && (start..witness).all(|offset| {
                                    proposition_at(&trace, verdict_time + u64::from(offset), 0)
                                })
                        });
                        let release_expected = (start..=end).all(|witness| {
                            proposition_at(&trace, verdict_time + u64::from(witness), 1)
                                || (start..witness).any(|offset| {
                                    proposition_at(&trace, verdict_time + u64::from(offset), 0)
                                })
                        });
                        for (nodes, expected) in [
                            (globally_nodes.as_slice(), globally_expected),
                            (until_nodes.as_slice(), until_expected),
                            (release_nodes.as_slice(), release_expected),
                        ] {
                            let report = evaluate_closed_at(
                                formula(SemanticProfile::ClosedTraceV1, nodes),
                                "exhaustive-window",
                                &trace,
                                "exhaustive-trace",
                                verdict_time,
                                default_limits(),
                            )
                            .unwrap();
                            assert_eq!(
                                report.verdict,
                                if expected {
                                    TruthValue::True
                                } else {
                                    TruthValue::False
                                },
                                "interval=[{start},{end}] length={trace_length} bits={bits} time={verdict_time}"
                            );
                        }
                    }
                }
            }
        }
    }
}

// Trace: TC-003, TC-009, FR-001-AC-2, FR-003-AC-1, FR-005-AC-2
#[test]
fn evaluates_and_reports_requested_verdict_time() {
    let nodes = [p(0)];
    let report = evaluate_closed_at(
        formula(SemanticProfile::ClosedTraceV1, &nodes),
        "time-indexed",
        &[vec![], vec![PropositionId(0)]],
        "two-instants",
        1,
        default_limits(),
    )
    .unwrap();
    assert_eq!(report.verdict, TruthValue::True);
    assert_eq!(report.verdict_time, 1);
    assert_eq!(report.observed_through, Some(1));

    let prefix = evaluate_prefix_at(
        formula(SemanticProfile::OnlinePrefixV1, &nodes),
        "time-indexed-prefix",
        &[vec![], vec![PropositionId(0)]],
        "two-instants",
        false,
        1,
        default_limits(),
    )
    .unwrap();
    assert_eq!(prefix.verdict, TruthValue::True);
    assert_eq!(prefix.verdict_time, 1);
}

// Trace: TC-003, FR-001-AC-2, NFR-001-AC-1
#[test]
fn preserves_identities_and_deterministic_outcomes() {
    let nodes = [
        p(9),
        p(2),
        Node::new(NodeKind::Or {
            left: NodeId(0),
            right: NodeId(1),
        }),
    ];
    let run = || {
        evaluate_closed(
            formula(SemanticProfile::ClosedTraceV1, &nodes),
            "formula-identity",
            &[vec![PropositionId(2)]],
            "trace-identity",
            default_limits(),
        )
        .unwrap()
    };
    let first = run();
    let second = run();
    assert_eq!(first, second);
    assert_eq!(first.formula_id, "formula-identity");
    assert_eq!(first.trace_id, "trace-identity");
    assert_eq!(first.proposition_ids, [2, 9]);
}

// Trace: TC-004, FR-001-AC-3, NFR-001-AC-2
#[test]
fn rejects_profile_mismatch_and_work_limits() {
    let nodes = [
        p(0),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 10).unwrap(),
            operand: NodeId(0),
        }),
    ];
    let online = formula(SemanticProfile::OnlinePrefixV1, &nodes);
    assert!(matches!(
        evaluate_closed(online, "f", &[], "t", default_limits()),
        Err(EvaluationError::UnsupportedProfile { .. })
    ));

    let limits = EvaluationLimits {
        max_temporal_span: 5,
        ..default_limits()
    };
    assert_eq!(
        evaluate_closed(
            formula(SemanticProfile::ClosedTraceV1, &nodes),
            "f",
            &[],
            "t",
            limits,
        ),
        Err(EvaluationError::TemporalSpanExceeded {
            requested: 11,
            limit: 5,
        })
    );

    let mut deeply_nested = vec![p(0)];
    for index in 0..=512 {
        deeply_nested.push(Node::new(NodeKind::Not {
            operand: NodeId(index),
        }));
    }
    assert!(matches!(
        evaluate_closed(
            formula(SemanticProfile::ClosedTraceV1, &deeply_nested),
            "deep",
            &[],
            "empty",
            default_limits(),
        ),
        Err(EvaluationError::RecursionDepthExceeded { limit: 512 })
    ));
}

// Trace: TC-006, FR-002-AC-2, NFR-001-AC-2, StR-002-VC-1
#[test]
fn nested_large_bounds_do_not_wrap() {
    let maximum = Interval::new(u32::MAX, u32::MAX).unwrap();
    let nodes = [
        p(0),
        Node::new(NodeKind::Future {
            interval: maximum,
            operand: NodeId(0),
        }),
        Node::new(NodeKind::Globally {
            interval: maximum,
            operand: NodeId(1),
        }),
    ];
    let report = analyze_horizon(
        formula(SemanticProfile::ClosedTraceV1, &nodes),
        "large-nested",
    )
    .unwrap();
    assert_eq!(report.lookahead, 2 * u64::from(u32::MAX));
    assert_eq!(report.required_buffer, report.lookahead + 1);
}

// Trace: TC-007, FR-002-AC-3
#[test]
fn horizon_report_retains_identities_and_units() {
    let nodes = [
        p(4),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 2).unwrap(),
            operand: NodeId(0),
        }),
    ];
    let report = analyze_horizon(
        formula(SemanticProfile::OnlinePrefixV1, &nodes),
        "future-two",
    )
    .unwrap();
    assert_eq!(report.formula_id, "future-two");
    assert_eq!(report.formula_root, 1);
    assert_eq!(report.corpus_revision, TL_SYNTAX_CORPUS_REVISION);
    assert_eq!(report.unit, "discrete-instants");
    assert_eq!((report.lookahead, report.required_buffer), (2, 3));
}

// Trace: TC-008, TC-009, FR-002-AC-1, FR-003-AC-1, FR-003-AC-3, StR-002-VC-1
#[test]
fn prefix_semantics_preserve_pending_and_decide_early() {
    let nodes = [
        p(0),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 2).unwrap(),
            operand: NodeId(0),
        }),
    ];
    let online = formula(SemanticProfile::OnlinePrefixV1, &nodes);
    let pending = evaluate_prefix(
        online,
        "future",
        &[vec![]],
        "prefix-1",
        false,
        default_limits(),
    )
    .unwrap();
    assert_eq!(pending.verdict, TruthValue::Pending);

    let early_true = evaluate_prefix(
        online,
        "future",
        &[vec![], vec![PropositionId(0)]],
        "prefix-2",
        false,
        default_limits(),
    )
    .unwrap();
    assert_eq!(early_true.verdict, TruthValue::True);

    let deadline_false = evaluate_prefix(
        online,
        "future",
        &[vec![], vec![], vec![]],
        "prefix-3",
        false,
        default_limits(),
    )
    .unwrap();
    assert_eq!(deadline_false.verdict, TruthValue::False);
    assert_eq!(deadline_false.horizon, 2);
}

// Trace: TC-010, FR-003-AC-2
#[test]
fn closing_prefix_matches_closed_trace_evaluation() {
    let nodes = [
        p(0),
        Node::new(NodeKind::Globally {
            interval: Interval::new(0, 2).unwrap(),
            operand: NodeId(0),
        }),
    ];
    let trace = [vec![PropositionId(0)], vec![PropositionId(0)]];
    let closed = evaluate_closed(
        formula(SemanticProfile::ClosedTraceV1, &nodes),
        "globally",
        &trace,
        "trace",
        default_limits(),
    )
    .unwrap();
    let prefix = evaluate_prefix(
        formula(SemanticProfile::OnlinePrefixV1, &nodes),
        "globally",
        &trace,
        "trace",
        true,
        default_limits(),
    )
    .unwrap();
    assert_eq!(closed.verdict, prefix.verdict);
}
