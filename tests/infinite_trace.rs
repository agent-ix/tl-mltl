#![cfg(feature = "infinite-trace")]

use tl_mltl::infinite::{
    evaluate_lasso, evaluate_model, evaluate_prefix_safety, Disposition, EvaluationLimit,
    EvidenceBasis, EvidenceClosure, InfiniteError, InfiniteProvider, LassoRequest,
    ObservationValue, PrefixRequest, ProviderRegistry, ProviderRequest, RegistrationError,
    ResultReason, SettlementEvidence, SubjectKind, UncertaintyStatus, FEATURE, PROFILE,
};
use tl_mltl::{evaluate_closed_at, EvaluationLimits, TL_MLTL_SOURCE_REVISION};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as K, Interval, LassoTraceDocument, LivenessDisposition, LivenessSubject,
    LivenessSubjectKind, Node, NodeId, NodeKind, PartialValuation, PropositionId, SemanticProfile,
    TemporalInterval, TraceObservation, UnboundedInterval, ValuationEntry,
};

fn node(kind: K) -> InfiniteNode {
    InfiniteNode::new(kind)
}

fn formula(root: u32, nodes: Vec<InfiniteNode>) -> InfiniteFormulaDocument {
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(root),
        nodes,
    )
    .unwrap()
}

fn trace(prefix: &[ObservationValue], tail: &[ObservationValue]) -> LassoTraceDocument {
    let propositions = [PropositionId(7)];
    let observation = |position: usize, value| TraceObservation {
        position: u32::try_from(position).unwrap(),
        valuation: PartialValuation::new(
            "map".to_owned(),
            &propositions,
            vec![ValuationEntry {
                proposition: propositions[0],
                value,
            }],
        )
        .unwrap(),
    };
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".to_owned(),
        propositions.to_vec(),
        prefix
            .iter()
            .enumerate()
            .map(|(at, value)| observation(at, *value))
            .collect(),
        tail.iter()
            .enumerate()
            .map(|(at, value)| observation(prefix.len() + at, *value))
            .collect(),
    )
    .unwrap()
}

fn run(
    graph: &InfiniteFormulaDocument,
    lasso: &LassoTraceDocument,
    selected_position: u64,
    fairness: Option<&FairnessPremisesDocument>,
) -> tl_mltl::infinite::InfiniteResult {
    run_with_closure(
        graph,
        lasso,
        selected_position,
        fairness,
        EvidenceClosure::Closed,
    )
}

fn run_with_closure(
    graph: &InfiniteFormulaDocument,
    lasso: &LassoTraceDocument,
    selected_position: u64,
    fairness: Option<&FairnessPremisesDocument>,
    evidence_closure: EvidenceClosure,
) -> tl_mltl::infinite::InfiniteResult {
    let graph_id = graph.content_identity().unwrap();
    let trace_id = lasso.content_identity().unwrap();
    evaluate_lasso(&LassoRequest {
        formula: graph,
        trace: lasso,
        fairness,
        evidence_closure,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position,
        limit: EvaluationLimit::default(),
    })
    .unwrap()
}

// Trace: TC-143; FR-030-AC-2
#[test]
fn closure_declaration_distinguishes_pending_from_indeterminate() {
    let graph = formula(
        0,
        vec![node(K::Proposition {
            proposition: PropositionId(7),
        })],
    );
    for value in [ObservationValue::Missing, ObservationValue::Conflicting] {
        let lasso = trace(&[], &[value]);
        let closed = run_with_closure(&graph, &lasso, 0, None, EvidenceClosure::Closed);
        let progressing =
            run_with_closure(&graph, &lasso, 0, None, EvidenceClosure::ProgressDeclared);
        let expected_reason = if value == ObservationValue::Missing {
            ResultReason::MissingObservation
        } else {
            ResultReason::ConflictingObservation
        };
        assert_eq!(closed.disposition, Disposition::Inconclusive);
        assert_eq!(progressing.disposition, Disposition::Inconclusive);
        assert_eq!(closed.reason, Some(expected_reason));
        assert_eq!(progressing.reason, Some(expected_reason));
        assert_eq!(closed.basis, EvidenceBasis::Indeterminate);
        assert_eq!(progressing.basis, EvidenceBasis::Pending);
        assert_eq!(closed.uncertainty, Some(UncertaintyStatus::Indeterminate));
        assert_eq!(progressing.uncertainty, Some(UncertaintyStatus::Pending));
        assert_eq!(
            closed.identity.evidence_closure,
            Some(EvidenceClosure::Closed)
        );
        assert_eq!(
            progressing.identity.evidence_closure,
            Some(EvidenceClosure::ProgressDeclared)
        );
    }
    let temporal = formula(
        1,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Future {
                interval: closed(0, 1),
                operand: NodeId(0),
            }),
        ],
    );
    let both = trace(
        &[ObservationValue::Missing],
        &[ObservationValue::Conflicting],
    );
    let mixed = run(&temporal, &both, 0, None);
    assert_eq!(mixed.disposition, Disposition::Inconclusive);
    assert_eq!(
        mixed.reason,
        Some(ResultReason::MissingAndConflictingObservations)
    );
}

// Trace: TC-156; FR-033-AC-1
#[test]
fn exact_trace_evidence_names_all_fair_completions_and_replayable_examples() {
    let graph = formula(
        0,
        vec![node(K::Proposition {
            proposition: PropositionId(7),
        })],
    );
    let partial = trace(&[], &[ObservationValue::Missing]);
    let mixed = run(&graph, &partial, 0, None);
    let Some(SettlementEvidence::ExhaustiveTrace(evidence)) = mixed.evidence else {
        panic!("partial lasso must carry exhaustive trace evidence");
    };
    assert_eq!(evidence.all_admitted_fair_completions, 2);
    let satisfying = evidence.satisfying.unwrap().assignments;
    let falsifying = evidence.falsifying.unwrap().assignments;
    assert_eq!(satisfying.len(), 1);
    assert_eq!(falsifying.len(), 1);
    assert_eq!(
        (
            satisfying[0].position,
            satisfying[0].proposition,
            satisfying[0].value
        ),
        (0, PropositionId(7), true)
    );
    assert_eq!(
        (
            falsifying[0].position,
            falsifying[0].proposition,
            falsifying[0].value
        ),
        (0, PropositionId(7), false)
    );
    let true_value = run(&graph, &trace(&[], &[ObservationValue::True]), 0, None);
    let false_value = run(&graph, &trace(&[], &[ObservationValue::False]), 0, None);
    assert_eq!(true_value.disposition, Disposition::Proved);
    assert_eq!(false_value.disposition, Disposition::Refuted);
    let Some(SettlementEvidence::ExhaustiveTrace(true_evidence)) = true_value.evidence else {
        panic!("proved lasso must carry exhaustive trace evidence");
    };
    let Some(SettlementEvidence::ExhaustiveTrace(false_evidence)) = false_value.evidence else {
        panic!("refuted lasso must carry exhaustive trace evidence");
    };
    assert_eq!(true_evidence.all_admitted_fair_completions, 1);
    assert_eq!(false_evidence.all_admitted_fair_completions, 1);
    assert!(true_evidence.satisfying.is_some());
    assert!(true_evidence.falsifying.is_none());
    assert!(false_evidence.satisfying.is_none());
    assert!(false_evidence.falsifying.is_some());
}

fn open(start: u32) -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(start))
}

fn closed(start: u32, end: u32) -> TemporalInterval {
    TemporalInterval::Closed(Interval::new(start, end).unwrap())
}

// Trace: TC-145, TC-146, TC-150, TC-151; FR-031-AC-1, FR-032-AC-1
#[test]
fn future_fixed_points_and_lower_bounds_survive_loop_repetition() {
    let lasso = trace(
        &[ObservationValue::False],
        &[ObservationValue::False, ObservationValue::True],
    );
    let atom = node(K::Proposition {
        proposition: PropositionId(7),
    });
    let eventually = formula(
        1,
        vec![
            atom,
            node(K::Future {
                interval: open(2),
                operand: NodeId(0),
            }),
        ],
    );
    assert_eq!(
        run(&eventually, &lasso, 0, None).disposition,
        Disposition::Proved
    );
    assert_eq!(
        run(&eventually, &lasso, 1_000, None).disposition,
        Disposition::Proved
    );
    let globally = formula(
        1,
        vec![
            atom,
            node(K::Globally {
                interval: open(0),
                operand: NodeId(0),
            }),
        ],
    );
    assert_eq!(
        run(&globally, &lasso, 0, None).disposition,
        Disposition::Refuted
    );
    let bounded = formula(
        1,
        vec![
            atom,
            node(K::Future {
                interval: closed(0, 1),
                operand: NodeId(0),
            }),
        ],
    );
    assert_eq!(
        run(&bounded, &lasso, 0, None).disposition,
        Disposition::Refuted
    );
    assert_eq!(
        run(&bounded, &lasso, 1, None).disposition,
        Disposition::Proved
    );
}

// Trace: TC-147, TC-148; FR-031-AC-2
#[test]
fn past_nodes_stabilize_later_than_the_trace_loop_entry() {
    let lasso = trace(&[ObservationValue::True], &[ObservationValue::False]);
    let atom = node(K::Proposition {
        proposition: PropositionId(7),
    });
    let once = formula(
        1,
        vec![
            atom,
            node(K::Once {
                interval: open(0),
                operand: NodeId(0),
            }),
        ],
    );
    assert_eq!(run(&once, &lasso, 0, None).disposition, Disposition::Proved);
    assert_eq!(
        run(&once, &lasso, 100, None).disposition,
        Disposition::Proved
    );
    let previous = formula(
        1,
        vec![atom, node(K::StrongPrevious { operand: NodeId(0) })],
    );
    assert_eq!(
        run(&previous, &lasso, 0, None).disposition,
        Disposition::Refuted
    );
    assert_eq!(
        run(&previous, &lasso, 1, None).disposition,
        Disposition::Proved
    );
    assert_eq!(
        run(&previous, &lasso, 2, None).disposition,
        Disposition::Refuted
    );
}

// Trace: TC-142, TC-143, TC-144; FR-030-AC-1 through FR-030-AC-3
#[test]
fn one_unknown_cell_uses_one_shared_completion_across_references_and_loops() {
    let lasso = trace(&[], &[ObservationValue::Missing]);
    let atom = node(K::Proposition {
        proposition: PropositionId(7),
    });
    let contradiction = formula(
        2,
        vec![
            atom,
            node(K::Not { operand: NodeId(0) }),
            node(K::And {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    );
    let outcome = run(&contradiction, &lasso, 0, None);
    assert_eq!(outcome.disposition, Disposition::Refuted);
    assert_eq!(outcome.admitted_completions, 2);
    let repeated = formula(
        2,
        vec![
            atom,
            node(K::StrongPrevious { operand: NodeId(0) }),
            node(K::Equivalent {
                left: NodeId(0),
                right: NodeId(1),
            }),
        ],
    );
    assert_eq!(
        run(&repeated, &lasso, 1, None).disposition,
        Disposition::Proved
    );
    let conflict = trace(&[], &[ObservationValue::Conflicting]);
    let unresolved = run(&formula(0, vec![atom]), &conflict, 0, None);
    assert_eq!(unresolved.disposition, Disposition::Inconclusive);
    assert_eq!(
        unresolved.reason,
        Some(ResultReason::ConflictingObservation)
    );
}

// Trace: TC-152, TC-153, TC-154; FR-032-AC-2 and FR-032-AC-3
#[test]
fn fairness_filters_completions_without_vacuous_proof() {
    let graph = formula(
        1,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Not { operand: NodeId(0) }),
        ],
    );
    let fairness = FairnessPremisesDocument::new(
        &graph,
        graph.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    let partial = trace(&[], &[ObservationValue::Missing]);
    let result = run(&graph, &partial, 0, Some(&fairness));
    assert_eq!(result.disposition, Disposition::Refuted);
    assert_eq!(result.admitted_completions, 1);
    let Some(SettlementEvidence::ExhaustiveTrace(evidence)) = result.evidence else {
        panic!("fair trace refutation must carry exhaustive evidence");
    };
    assert_eq!(evidence.all_admitted_fair_completions, 1);
    assert!(evidence.satisfying.is_none());
    assert!(evidence.falsifying.unwrap().assignments[0].value);
    let unfair = trace(&[], &[ObservationValue::False]);
    let result = run(&graph, &unfair, 0, Some(&fairness));
    assert_eq!(result.disposition, Disposition::Inconclusive);
    assert_eq!(result.reason, Some(ResultReason::EmptyFairAdmission));
    assert_eq!(result.basis, EvidenceBasis::Pending);
    assert_eq!(result.uncertainty, None);
    assert!(result.evidence.is_none());
}

// Trace: TC-089, TC-139, TC-140, TC-155, TC-158, TC-159; FR-028-AC-2, FR-029-AC-2, FR-033-AC-1, FR-033-AC-3, FR-034-AC-1
#[test]
fn model_and_identity_refusals_keep_their_scope() {
    let graph = formula(0, vec![node(K::True)]);
    let lasso = trace(&[], &[ObservationValue::True]);
    let trace_result = run(&graph, &lasso, 0, None);
    assert_eq!(trace_result.disposition, Disposition::Proved);
    assert_eq!(trace_result.identity.feature, FEATURE);
    assert_eq!(
        trace_result.identity.provider_revision,
        TL_MLTL_SOURCE_REVISION
    );
    assert_eq!(trace_result.identity.profile, PROFILE);
    assert_eq!(
        trace_result.identity.graph_id,
        graph.content_identity().unwrap()
    );
    assert_eq!(
        trace_result.identity.proposition_map_id,
        lasso.proposition_map_identity()
    );
    assert_eq!(trace_result.identity.subject_kind, SubjectKind::Lasso);
    assert_eq!(
        trace_result.identity.subject_id,
        lasso.content_identity().unwrap()
    );
    assert_eq!(
        trace_result.identity.trace_id.as_deref(),
        Some(trace_result.identity.subject_id.as_str())
    );
    assert_eq!(trace_result.identity.clock, "event_position");
    assert_eq!(trace_result.identity.selected_position, 0);
    let model = evaluate_model(&graph, &graph.content_identity().unwrap(), "model", "map").unwrap();
    assert_eq!(model.disposition, Disposition::Unsupported);
    assert_eq!(model.identity.feature, FEATURE);
    assert_eq!(model.identity.provider_revision, TL_MLTL_SOURCE_REVISION);
    assert_eq!(model.identity.profile, PROFILE);
    assert_eq!(model.identity.subject_kind, SubjectKind::Model);
    assert!(model.identity.trace_id.is_none());
    assert!(model.evidence.is_none());
    let bounded_nodes = [Node::new(NodeKind::True)];
    let bounded_formula =
        tl_syntax::Formula::new(SemanticProfile::ClosedTraceV1, NodeId(0), &bounded_nodes).unwrap();
    let bounded = evaluate_closed_at(
        bounded_formula,
        "bounded-formula",
        &[vec![]],
        "bounded-trace",
        0,
        EvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(
        bounded.semantic_profile,
        SemanticProfile::ClosedTraceV1.as_str()
    );
    assert_eq!(bounded.formula_id, "bounded-formula");
    assert_eq!(bounded.trace_id, "bounded-trace");
    let bounded_wire = serde_json::to_value(&bounded).unwrap();
    assert!(bounded_wire.get("providerRevision").is_none());
    assert!(bounded_wire.get("subjectKind").is_none());
    let bad = evaluate_lasso(&LassoRequest {
        formula: &graph,
        trace: &lasso,
        fairness: None,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: "",
        trace_id: "trace",
        selected_position: 0,
        limit: EvaluationLimit::default(),
    });
    assert!(matches!(bad, Err(InfiniteError::IdentityMismatch)));
}

// Trace: TC-139; FR-028-AC-1, FR-028-AC-2
#[test]
fn lasso_refuses_each_identity_axis_independently() {
    let graph = formula(0, vec![node(K::True)]);
    let lasso = trace(&[], &[ObservationValue::True]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = lasso.content_identity().unwrap();
    for (requested_graph, requested_trace) in [
        ("wrong-graph", trace_id.as_str()),
        (graph_id.as_str(), "wrong-trace"),
    ] {
        assert!(matches!(
            evaluate_lasso(&LassoRequest {
                formula: &graph,
                trace: &lasso,
                fairness: None,
                evidence_closure: EvidenceClosure::Closed,
                graph_id: requested_graph,
                trace_id: requested_trace,
                selected_position: 0,
                limit: EvaluationLimit::default(),
            }),
            Err(InfiniteError::IdentityMismatch)
        ));
    }
}

// Trace: TC-152; FR-032-AC-1
#[test]
fn lasso_refuses_fairness_from_a_different_graph() {
    let graph = formula(0, vec![node(K::True)]);
    let other_graph = formula(0, vec![node(K::False)]);
    let fairness = FairnessPremisesDocument::new(
        &other_graph,
        other_graph.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    let lasso = trace(&[], &[ObservationValue::True]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = lasso.content_identity().unwrap();
    assert!(matches!(
        evaluate_lasso(&LassoRequest {
            formula: &graph,
            trace: &lasso,
            fairness: Some(&fairness),
            evidence_closure: EvidenceClosure::Closed,
            graph_id: &graph_id,
            trace_id: &trace_id,
            selected_position: 0,
            limit: EvaluationLimit::default(),
        }),
        Err(InfiniteError::IdentityMismatch)
    ));
}

// Trace: TC-139; FR-028-AC-1 and FR-028-AC-2
#[test]
fn deployment_registry_routes_one_provider_and_refuses_duplicate() {
    let graph = formula(0, vec![node(K::True)]);
    let lasso = trace(&[], &[ObservationValue::True]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = lasso.content_identity().unwrap();
    let provider = InfiniteProvider {
        request: ProviderRequest::Lasso(LassoRequest {
            formula: &graph,
            trace: &lasso,
            fairness: None,
            evidence_closure: EvidenceClosure::Closed,
            graph_id: &graph_id,
            trace_id: &trace_id,
            selected_position: 0,
            limit: EvaluationLimit::default(),
        }),
    };
    let subject = LivenessSubject {
        kind: LivenessSubjectKind::LassoTrace,
        identity: &trace_id,
    };
    let mut registry = ProviderRegistry::default();
    let absent = registry.settle(&graph, subject);
    assert_eq!(absent.disposition, LivenessDisposition::Unsupported);
    assert_eq!(absent.warning, Some("tl-syntax.liveness/v1"));
    registry.register(&provider).unwrap();
    assert_eq!(
        registry.register(&provider),
        Err(RegistrationError::DuplicateProvider)
    );
    assert_eq!(
        registry.settle(&graph, subject).disposition,
        LivenessDisposition::Proved
    );
    let model = LivenessSubject {
        kind: LivenessSubjectKind::Model,
        identity: "model",
    };
    assert_eq!(
        registry.settle(&graph, model).disposition,
        LivenessDisposition::Unsupported
    );
}

// Trace: TC-159; FR-034-AC-2 and FR-034-AC-3
#[test]
fn exact_work_limit_succeeds_and_one_less_is_resource_incomplete() {
    let graph = formula(
        1,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Future {
                interval: open(0),
                operand: NodeId(0),
            }),
        ],
    );
    let lasso = trace(&[], &[ObservationValue::True]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = lasso.content_identity().unwrap();
    let request = |max_steps| LassoRequest {
        formula: &graph,
        trace: &lasso,
        fairness: None,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: 0,
        limit: EvaluationLimit {
            max_steps,
            ..EvaluationLimit::default()
        },
    };
    let baseline = evaluate_lasso(&request(u64::MAX)).unwrap();
    assert_eq!(baseline.disposition, Disposition::Proved);
    assert_eq!(
        evaluate_lasso(&request(baseline.evaluation_steps))
            .unwrap()
            .disposition,
        Disposition::Proved
    );
    let incomplete = evaluate_lasso(&request(baseline.evaluation_steps - 1)).unwrap();
    assert_eq!(incomplete.disposition, Disposition::Failed);
    assert_eq!(incomplete.reason, Some(ResultReason::ResourceIncomplete));
    assert!(incomplete.evidence.is_none());
    assert_eq!(
        incomplete.execution,
        tl_mltl::infinite::ExecutionDisposition::ResourceIncomplete
    );
}

// Trace: TC-188; FR-049-AC-2, NFR-009-AC-1
#[test]
fn every_lasso_resource_dimension_refuses_one_over_without_panic() {
    let graph = formula(
        1,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Future {
                interval: open(0),
                operand: NodeId(0),
            }),
        ],
    );
    let complete = trace(&[], &[ObservationValue::True]);
    let partial = trace(&[], &[ObservationValue::Missing]);
    let graph_id = graph.content_identity().unwrap();
    for (lasso, completion_count) in [(&complete, 1), (&partial, 2)] {
        let trace_id = lasso.content_identity().unwrap();
        let evaluate = |limit| {
            evaluate_lasso(&LassoRequest {
                formula: &graph,
                trace: lasso,
                fairness: None,
                evidence_closure: EvidenceClosure::Closed,
                graph_id: &graph_id,
                trace_id: &trace_id,
                selected_position: 0,
                limit,
            })
            .unwrap()
        };
        let exact = EvaluationLimit {
            max_nodes: 2,
            max_positions: 1,
            max_valuation_cells: 1,
            max_states: 2,
            max_completions: completion_count,
            ..EvaluationLimit::default()
        };
        assert_ne!(evaluate(exact).disposition, Disposition::Failed);
        let one_over = [
            EvaluationLimit {
                max_nodes: 1,
                ..exact
            },
            EvaluationLimit {
                max_positions: 0,
                ..exact
            },
            EvaluationLimit {
                max_valuation_cells: 0,
                ..exact
            },
            EvaluationLimit {
                max_states: 1,
                ..exact
            },
            EvaluationLimit {
                max_completions: exact.max_completions - 1,
                ..exact
            },
        ];
        for limited in one_over {
            let outcome = evaluate(limited);
            assert_eq!(outcome.disposition, Disposition::Failed);
            assert_eq!(outcome.reason, Some(ResultReason::ResourceIncomplete));
        }
    }
}

// Trace: TC-188; FR-049-AC-2, NFR-009-AC-1
#[test]
fn proposition_word_refuses_at_zero_state_budget() {
    let graph = formula(
        0,
        vec![node(K::Proposition {
            proposition: PropositionId(7),
        })],
    );
    let lasso = trace(&[], &[ObservationValue::True]);
    let graph_id = graph.content_identity().unwrap();
    let trace_id = lasso.content_identity().unwrap();
    let result = evaluate_lasso(&LassoRequest {
        formula: &graph,
        trace: &lasso,
        fairness: None,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: 0,
        limit: EvaluationLimit {
            max_states: 0,
            ..EvaluationLimit::default()
        },
    })
    .unwrap();
    assert_eq!(result.disposition, Disposition::Failed);
    assert_eq!(result.reason, Some(ResultReason::ResourceIncomplete));
}

// Trace: TC-188; FR-049-AC-2, NFR-009-AC-1
#[test]
fn each_periodic_operator_refuses_at_its_own_state_budget_boundary() {
    let atom = || K::Proposition {
        proposition: PropositionId(7),
    };
    let cases = vec![
        ("not", vec![atom(), K::Not { operand: NodeId(0) }]),
        (
            "future",
            vec![
                atom(),
                K::Future {
                    interval: open(0),
                    operand: NodeId(0),
                },
            ],
        ),
        (
            "globally",
            vec![
                atom(),
                K::Globally {
                    interval: open(0),
                    operand: NodeId(0),
                },
            ],
        ),
        (
            "once",
            vec![
                atom(),
                K::Once {
                    interval: closed(0, 0),
                    operand: NodeId(0),
                },
            ],
        ),
        (
            "historically",
            vec![
                atom(),
                K::Historically {
                    interval: closed(0, 0),
                    operand: NodeId(0),
                },
            ],
        ),
        (
            "strong_previous",
            vec![atom(), K::StrongPrevious { operand: NodeId(0) }],
        ),
        (
            "and",
            vec![
                atom(),
                K::True,
                K::And {
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
        (
            "or",
            vec![
                atom(),
                K::True,
                K::Or {
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
        (
            "implies",
            vec![
                atom(),
                K::True,
                K::Implies {
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
        (
            "equivalent",
            vec![
                atom(),
                K::True,
                K::Equivalent {
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
        (
            "until",
            vec![
                atom(),
                K::True,
                K::Until {
                    interval: closed(0, 0),
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
        (
            "release",
            vec![
                atom(),
                K::True,
                K::Release {
                    interval: closed(0, 0),
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
        (
            "since",
            vec![
                atom(),
                K::True,
                K::Since {
                    interval: closed(0, 0),
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
        (
            "triggered",
            vec![
                atom(),
                K::True,
                K::Triggered {
                    interval: closed(0, 0),
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ],
        ),
    ];
    let lasso = trace(&[], &[ObservationValue::True]);
    let trace_id = lasso.content_identity().unwrap();
    for (name, kinds) in cases {
        let root = u32::try_from(kinds.len() - 1).unwrap();
        let graph = formula(root, kinds.into_iter().map(node).collect());
        let graph_id = graph.content_identity().unwrap();
        let evaluate = |limit| {
            evaluate_lasso(&LassoRequest {
                formula: &graph,
                trace: &lasso,
                fairness: None,
                evidence_closure: EvidenceClosure::Closed,
                graph_id: &graph_id,
                trace_id: &trace_id,
                selected_position: 0,
                limit,
            })
            .unwrap()
        };
        assert_ne!(
            evaluate(EvaluationLimit::default()).disposition,
            Disposition::Failed,
            "{name}"
        );
        let bounded = evaluate(EvaluationLimit {
            max_states: graph.nodes().len() - 1,
            ..EvaluationLimit::default()
        });
        assert_eq!(bounded.disposition, Disposition::Failed, "{name}");
        assert_eq!(
            bounded.reason,
            Some(ResultReason::ResourceIncomplete),
            "{name}"
        );
        assert!(bounded.evidence.is_none(), "{name}");
    }
}

// Trace: TC-188; FR-049-AC-2, NFR-009-AC-1
#[test]
fn prefix_resource_dimensions_refuse_one_over_without_panic() {
    let graph = formula(
        1,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Globally {
                interval: open(0),
                operand: NodeId(0),
            }),
        ],
    );
    let graph_id = graph.content_identity().unwrap();
    let finite = trace(&[ObservationValue::False], &[ObservationValue::True]);
    let evaluate = |limit| {
        evaluate_prefix_safety(&PrefixRequest {
            formula: &graph,
            graph_id: &graph_id,
            proposition_map_id: "map",
            propositions: finite.propositions(),
            observations: finite.prefix(),
            limit,
        })
        .unwrap()
    };
    let exact = EvaluationLimit {
        max_nodes: 2,
        max_positions: 2,
        max_valuation_cells: 1,
        max_states: 4,
        max_completions: 1,
        ..EvaluationLimit::default()
    };
    let baseline = evaluate(exact);
    assert_eq!(baseline.disposition, Disposition::Refuted);
    assert_eq!(
        evaluate(EvaluationLimit {
            max_steps: baseline.evaluation_steps,
            ..exact
        })
        .disposition,
        Disposition::Refuted
    );
    let one_over = [
        EvaluationLimit {
            max_nodes: 1,
            ..exact
        },
        EvaluationLimit {
            max_positions: 1,
            ..exact
        },
        EvaluationLimit {
            max_valuation_cells: 0,
            ..exact
        },
        EvaluationLimit {
            max_states: 3,
            ..exact
        },
        EvaluationLimit {
            max_completions: 0,
            ..exact
        },
        EvaluationLimit {
            max_steps: baseline.evaluation_steps - 1,
            ..exact
        },
    ];
    for limited in one_over {
        let outcome = evaluate(limited);
        assert_eq!(outcome.disposition, Disposition::Failed);
        assert_eq!(outcome.reason, Some(ResultReason::ResourceIncomplete));
    }
}

// Trace: TC-089, TC-140, TC-156, TC-157, TC-168, TC-169, TC-170; FR-029-AC-2, FR-033-AC-1, FR-033-AC-2, FR-040-AC-1, FR-040-AC-2
#[test]
fn finite_prefix_refutes_only_a_decisive_safety_violation() {
    let graph = formula(
        1,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Globally {
                interval: open(0),
                operand: NodeId(0),
            }),
        ],
    );
    let graph_id = graph.content_identity().unwrap();
    let false_trace = trace(&[ObservationValue::False], &[ObservationValue::True]);
    let request = PrefixRequest {
        formula: &graph,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: false_trace.propositions(),
        observations: false_trace.prefix(),
        limit: EvaluationLimit::default(),
    };
    let refuted = evaluate_prefix_safety(&request).unwrap();
    assert_eq!(refuted.disposition, Disposition::Refuted);
    assert_eq!(refuted.identity.feature, FEATURE);
    assert_eq!(refuted.identity.provider_revision, TL_MLTL_SOURCE_REVISION);
    assert_eq!(refuted.identity.profile, PROFILE);
    assert_eq!(refuted.identity.graph_id, graph_id);
    assert_eq!(refuted.identity.proposition_map_id, "map");
    assert_eq!(refuted.identity.subject_kind, SubjectKind::FinitePrefix);
    assert_eq!(
        refuted.identity.trace_id.as_deref(),
        Some(refuted.identity.subject_id.as_str())
    );
    assert_eq!(refuted.identity.clock, "event_position");
    assert_eq!(refuted.basis, tl_mltl::infinite::EvidenceBasis::BadPrefix);
    let Some(SettlementEvidence::BadPrefix(counterexample)) = refuted.evidence else {
        panic!("finite safety refutation must carry a bad-prefix counterexample");
    };
    assert_eq!(counterexample.violation_position, 0);
    assert_eq!(counterexample.decision_horizon, 0);
    assert_eq!(counterexample.observed_through, 0);
    let true_trace = trace(&[ObservationValue::True], &[ObservationValue::False]);
    let unsettled = evaluate_prefix_safety(&PrefixRequest {
        observations: true_trace.prefix(),
        propositions: true_trace.propositions(),
        ..request
    })
    .unwrap();
    assert_eq!(unsettled.disposition, Disposition::Inconclusive);
    assert_eq!(unsettled.reason, Some(ResultReason::FinitePrefixUnsettled));
    assert!(unsettled.evidence.is_none());
}

// Trace: TC-157, TC-168, TC-172; FR-033-AC-2, FR-040-AC-1, FR-041-AC-1
#[test]
fn finite_horizon_inner_future_requires_enough_prefix_positions() {
    let graph = formula(
        2,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Future {
                interval: closed(0, 1),
                operand: NodeId(0),
            }),
            node(K::Globally {
                interval: open(0),
                operand: NodeId(1),
            }),
        ],
    );
    let graph_id = graph.content_identity().unwrap();
    let short = trace(&[ObservationValue::False], &[ObservationValue::False]);
    let outcome = evaluate_prefix_safety(&PrefixRequest {
        formula: &graph,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: short.propositions(),
        observations: short.prefix(),
        limit: EvaluationLimit::default(),
    })
    .unwrap();
    assert_eq!(outcome.disposition, Disposition::Inconclusive);
    let long = trace(
        &[ObservationValue::False, ObservationValue::False],
        &[ObservationValue::True],
    );
    let outcome = evaluate_prefix_safety(&PrefixRequest {
        formula: &graph,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: long.propositions(),
        observations: long.prefix(),
        limit: EvaluationLimit::default(),
    })
    .unwrap();
    assert_eq!(outcome.disposition, Disposition::Refuted);
    let unbounded = formula(
        2,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Future {
                interval: open(0),
                operand: NodeId(0),
            }),
            node(K::Globally {
                interval: open(0),
                operand: NodeId(1),
            }),
        ],
    );
    let refused = evaluate_prefix_safety(&PrefixRequest {
        formula: &unbounded,
        graph_id: &unbounded.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: long.propositions(),
        observations: long.prefix(),
        limit: EvaluationLimit::default(),
    })
    .unwrap();
    assert_eq!(refused.disposition, Disposition::Unsupported);
    assert_eq!(
        refused.reason,
        Some(ResultReason::SafetyFragmentUnsupported)
    );
}

// Trace: TC-139, TC-157; FR-028-AC-2, FR-033-AC-2
#[test]
fn registered_prefix_route_returns_only_a_bad_prefix_refutation() {
    let graph = formula(
        1,
        vec![
            node(K::Proposition {
                proposition: PropositionId(7),
            }),
            node(K::Globally {
                interval: open(0),
                operand: NodeId(0),
            }),
        ],
    );
    let graph_id = graph.content_identity().unwrap();
    let trace = trace(&[ObservationValue::False], &[ObservationValue::True]);
    let request = PrefixRequest {
        formula: &graph,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: trace.propositions(),
        observations: trace.prefix(),
        limit: EvaluationLimit::default(),
    };
    let prefix_id = request.content_identity().unwrap();
    let provider = InfiniteProvider {
        request: ProviderRequest::FinitePrefix(request),
    };
    let mut registry = ProviderRegistry::default();
    registry.register(&provider).unwrap();
    let subject = LivenessSubject {
        kind: LivenessSubjectKind::FinitePrefix,
        identity: &prefix_id,
    };
    assert_eq!(
        registry.settle(&graph, subject).disposition,
        LivenessDisposition::Refuted
    );
    let wrong = LivenessSubject {
        kind: LivenessSubjectKind::FinitePrefix,
        identity: "other",
    };
    assert_eq!(
        registry.settle(&graph, wrong).disposition,
        LivenessDisposition::Unsupported
    );
}
