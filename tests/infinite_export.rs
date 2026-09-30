#![cfg(feature = "infinite-trace")]

use tl_mltl::{
    infinite::{
        export_safety_monitor, replay_target_step, EvaluationLimit, PrefixRequest,
        SafetyExportError, SafetyReplayDisposition, TargetStepObservation,
    },
    PastMappingError, TargetOriginContract,
};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as K, Interval, NodeId, OwnedSignalDeclaration, PartialValuation,
    PartialValue, PastOperatorKind, PropositionBinding, PropositionId, SemanticProfile,
    SignalCatalogDocument, SignalDomain, SignalId, TemporalInterval, TraceObservation,
    UnboundedInterval, ValuationEntry,
};

fn graph(nodes: Vec<InfiniteNode>) -> InfiniteFormulaDocument {
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
}

fn node(kind: K) -> InfiniteNode {
    InfiniteNode::new(kind)
}
fn open() -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(0))
}
fn closed() -> TemporalInterval {
    TemporalInterval::Closed(Interval::new(0, 1).unwrap())
}

fn catalog() -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(1),
            "p".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(0), SignalId(1))],
    )
    .unwrap()
}

fn contract() -> TargetOriginContract {
    TargetOriginContract::reviewed_r2u2_4_2()
}

fn rows(value: PartialValue) -> Vec<TraceObservation> {
    vec![TraceObservation {
        position: 0,
        valuation: PartialValuation::new(
            "map".to_owned(),
            &[PropositionId(0)],
            vec![ValuationEntry {
                proposition: PropositionId(0),
                value,
            }],
        )
        .unwrap(),
    }]
}

fn export(
    graph: &InfiniteFormulaDocument,
    observations: &[TraceObservation],
) -> Result<tl_mltl::infinite::SafetyMappingManifest, SafetyExportError> {
    export_safety_monitor(
        &PrefixRequest {
            formula: graph,
            graph_id: &graph.content_identity().unwrap(),
            proposition_map_id: "map",
            propositions: &[PropositionId(0)],
            observations,
            limit: EvaluationLimit::default(),
        },
        None,
        &catalog(),
        &contract(),
        100,
    )
}

// Trace: TC-168, TC-169, TC-171; FR-040-AC-1 and FR-040-AC-3
#[test]
fn exact_outer_safety_guard_exports_only_its_finite_horizon_body() {
    let past = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let evidence = export(&past, &rows(PartialValue::True)).unwrap();
    assert_eq!(evidence.section, "PTSPEC");
    assert_eq!(evidence.expression, "O[0,1](p)");
    assert_eq!(evidence.decision_horizon, 0);
    assert!(evidence.refutation_only);
    assert_eq!(evidence.profile, "mltl.infinite-trace/v1");
    let future = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let evidence = export(&future, &rows(PartialValue::True)).unwrap();
    assert_eq!(evidence.section, "FTSPEC");
    assert_eq!(evidence.expression, "F[0,1](p)");
    assert_eq!(evidence.decision_horizon, 1);
}

// Trace: TC-172, TC-173; FR-041-AC-1 and FR-041-AC-2
#[test]
fn unsupported_shape_partial_observation_and_mixed_target_context_refuse() {
    let unbounded = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: open(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&unbounded, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedLiveness)
    );
    let past = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&past, &rows(PartialValue::Missing)),
        Err(SafetyExportError::PartialValuation)
    );
    let mixed = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(1),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(2),
        }),
    ]);
    assert_eq!(
        export(&mixed, &rows(PartialValue::True)),
        Err(SafetyExportError::MixedTargetContext)
    );
}

// Trace: TC-172, TC-173; FR-041-AC-1 and FR-041-AC-2
#[test]
fn each_unbounded_future_family_has_a_distinct_typed_refusal() {
    let atom = node(K::Proposition {
        proposition: PropositionId(0),
    });
    let future = graph(vec![
        atom,
        node(K::Future {
            interval: open(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&future, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedLiveness)
    );
    for temporal in [
        K::Until {
            interval: open(),
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Release {
            interval: open(),
            left: NodeId(0),
            right: NodeId(1),
        },
    ] {
        let formula = graph(vec![
            atom,
            node(K::True),
            node(temporal),
            node(K::Globally {
                interval: open(),
                operand: NodeId(2),
            }),
        ]);
        assert_eq!(
            export(&formula, &rows(PartialValue::True)),
            Err(SafetyExportError::UnboundedUntilRelease)
        );
    }
    let wrong_outer = graph(vec![
        atom,
        node(K::Globally {
            interval: TemporalInterval::Unbounded(UnboundedInterval::new(1)),
            operand: NodeId(0),
        }),
    ]);
    assert_eq!(
        export(&wrong_outer, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedLiveness)
    );
    let bounded_outer = graph(vec![
        atom,
        node(K::Globally {
            interval: closed(),
            operand: NodeId(0),
        }),
    ]);
    assert_eq!(
        export(&bounded_outer, &rows(PartialValue::True)),
        Err(SafetyExportError::UnsupportedShape)
    );
}

// Trace: TC-172, TC-173; FR-041-AC-1 and FR-041-AC-2
#[test]
fn nonempty_fairness_refuses_before_target_output_and_empty_fairness_is_neutral() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(0),
        }),
    ]);
    let graph_id = safety.content_identity().unwrap();
    let observations = rows(PartialValue::True);
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &observations,
        limit: EvaluationLimit::default(),
    };
    let premises = FairnessPremisesDocument::new(
        &safety,
        graph_id.clone(),
        InfiniteClock::EventPosition,
        vec![NodeId(0)],
    )
    .unwrap();
    assert_eq!(
        export_safety_monitor(&request, Some(&premises), &catalog(), &contract(), 100),
        Err(SafetyExportError::FairnessPremise)
    );
    let empty = FairnessPremisesDocument::new(
        &safety,
        graph_id.clone(),
        InfiniteClock::EventPosition,
        vec![],
    )
    .unwrap();
    assert!(export_safety_monitor(&request, Some(&empty), &catalog(), &contract(), 100).is_ok());
    let foreign_graph = graph(vec![node(K::True)]);
    let foreign = FairnessPremisesDocument::new(
        &foreign_graph,
        foreign_graph.content_identity().unwrap(),
        InfiniteClock::EventPosition,
        vec![],
    )
    .unwrap();
    assert_eq!(
        export_safety_monitor(&request, Some(&foreign), &catalog(), &contract(), 100),
        Err(SafetyExportError::Identity)
    );
}

// Trace: TC-170; FR-040-AC-2
#[test]
fn export_refuses_a_noncanonical_prefix_and_missing_formula_binding() {
    let past = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let mut shifted = rows(PartialValue::True);
    shifted[0].position = 1;
    assert_eq!(export(&past, &shifted), Err(SafetyExportError::Identity));

    let request = PrefixRequest {
        formula: &past,
        graph_id: &past.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: &[],
        observations: &[],
        limit: EvaluationLimit::default(),
    };
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &contract(), 100),
        Err(SafetyExportError::Identity)
    );
}

// Trace: TC-166, TC-173; FR-039-AC-1, FR-041-AC-2
#[test]
fn safety_export_refuses_target_origin_mismatch_before_artifact() {
    let bounds = Interval::new(2, 2).unwrap();
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: TemporalInterval::Closed(bounds),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&safety, &rows(PartialValue::True)),
        Err(SafetyExportError::TargetOriginIntervalMismatch {
            operator: PastOperatorKind::Once,
            interval: bounds,
        })
    );
}

// Trace: TC-166, TC-173; FR-039-AC-1, FR-041-AC-2
#[test]
fn safety_export_refuses_exhausted_work_and_unreviewed_past_operator() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let observations = rows(PartialValue::True);
    let graph_id = safety.content_identity().unwrap();
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &observations,
        limit: EvaluationLimit::default(),
    };
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &contract(), 0),
        Err(SafetyExportError::ResourceIncomplete)
    );
    let mut unreviewed = contract();
    unreviewed.admitted_operators.clear();
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &unreviewed, 100),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::TargetOriginUnverified(PastOperatorKind::Once)
        ))
    );
    let mut missing_evidence = contract();
    missing_evidence.evidence_sha256.clear();
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &missing_evidence, 100),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::MissingOriginEvidence
        ))
    );
    let mut foreign_target = contract();
    foreign_target.target.version = "C2PO v4.2.0".to_owned();
    assert_eq!(
        export_safety_monitor(&request, None, &catalog(), &foreign_target, 100),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::TargetOriginMismatch
        ))
    );
}

// Trace: TC-172, TC-173; FR-041-AC-1, FR-041-AC-2
#[test]
fn safety_export_refuses_unbounded_past_inside_the_finite_body() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: open(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    assert_eq!(
        export(&safety, &rows(PartialValue::True)),
        Err(SafetyExportError::UnboundedPast)
    );
}

// Trace: TC-169, TC-170; FR-040-AC-1 and FR-040-AC-2
#[test]
fn target_violation_replays_at_its_exact_position_and_pass_remains_inconclusive() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(0),
        }),
    ]);
    let false_rows = rows(PartialValue::False);
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &safety.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &false_rows,
        limit: EvaluationLimit::default(),
    };
    let manifest = export_safety_monitor(&request, None, &catalog(), &contract(), 100).unwrap();
    let step = |position, verdict| TargetStepObservation {
        target: &manifest.target,
        expression_sha256: &manifest.output_sha256,
        position,
        verdict,
    };
    assert_eq!(
        replay_target_step(&manifest, &request, step(0, false)).unwrap(),
        SafetyReplayDisposition::Refuted
    );
    assert_eq!(
        replay_target_step(&manifest, &request, step(0, true)).unwrap(),
        SafetyReplayDisposition::Mismatch
    );
    assert_eq!(
        replay_target_step(&manifest, &request, step(1, false)).unwrap(),
        SafetyReplayDisposition::Mismatch
    );
    let mut stale = manifest.clone();
    stale.refutation_only = false;
    assert_eq!(
        replay_target_step(&stale, &request, step(0, false)),
        Err(SafetyExportError::TargetMismatch)
    );
    let wrong_digest = TargetStepObservation {
        target: &manifest.target,
        expression_sha256: "0",
        position: 0,
        verdict: false,
    };
    assert_eq!(
        replay_target_step(&manifest, &request, wrong_digest),
        Err(SafetyExportError::TargetMismatch)
    );

    let true_rows = rows(PartialValue::True);
    let passing_request = PrefixRequest {
        formula: &safety,
        graph_id: &safety.content_identity().unwrap(),
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &true_rows,
        limit: EvaluationLimit::default(),
    };
    let passing =
        export_safety_monitor(&passing_request, None, &catalog(), &contract(), 100).unwrap();
    let passing_step = TargetStepObservation {
        target: &passing.target,
        expression_sha256: &passing.output_sha256,
        position: 0,
        verdict: true,
    };
    assert_eq!(
        replay_target_step(&passing, &passing_request, passing_step).unwrap(),
        SafetyReplayDisposition::Inconclusive
    );
}

// Trace: TC-172; FR-041-AC-1
#[test]
fn every_syntax_node_and_interval_kind_has_an_explicit_export_class() {
    let atom = node(K::Proposition {
        proposition: PropositionId(0),
    });
    let booleans = [
        K::False,
        K::True,
        K::Proposition {
            proposition: PropositionId(0),
        },
        K::Not { operand: NodeId(0) },
        K::And {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Or {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Implies {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Equivalent {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::StrongPrevious { operand: NodeId(0) },
    ];
    for kind in booleans {
        let formula = graph(vec![
            atom,
            node(K::True),
            node(kind),
            node(K::Globally {
                interval: open(),
                operand: NodeId(2),
            }),
        ]);
        let mapped = export(&formula, &rows(PartialValue::True)).unwrap();
        assert!(mapped.refutation_only, "{kind:?}");
    }
    let mut cells = 0;
    for interval in [
        TemporalInterval::Closed(Interval::new(0, 0).unwrap()),
        open(),
    ] {
        let variants = [
            K::Future {
                interval,
                operand: NodeId(0),
            },
            K::Globally {
                interval,
                operand: NodeId(0),
            },
            K::Until {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Release {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Once {
                interval,
                operand: NodeId(0),
            },
            K::Historically {
                interval,
                operand: NodeId(0),
            },
            K::Since {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Triggered {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
        ];
        for (index, kind) in variants.into_iter().enumerate() {
            let formula = graph(vec![
                atom,
                node(K::True),
                node(kind),
                node(K::Globally {
                    interval: open(),
                    operand: NodeId(2),
                }),
            ]);
            let result = export(&formula, &rows(PartialValue::True));
            match (interval, index) {
                (TemporalInterval::Closed(_), _) => assert!(result.is_ok(), "{kind:?}: {result:?}"),
                (TemporalInterval::Unbounded(_), 0 | 1) => assert_eq!(
                    result,
                    Err(SafetyExportError::UnboundedLiveness),
                    "{kind:?}"
                ),
                (TemporalInterval::Unbounded(_), 2 | 3) => assert_eq!(
                    result,
                    Err(SafetyExportError::UnboundedUntilRelease),
                    "{kind:?}"
                ),
                (TemporalInterval::Unbounded(_), 4..=7) => {
                    assert_eq!(result, Err(SafetyExportError::UnboundedPast), "{kind:?}")
                }
                _ => unreachable!(),
            }
            cells += 1;
        }
    }
    assert_eq!(cells, 16);
}

// Trace: TC-168, TC-172; FR-040-AC-1, FR-041-AC-1
#[test]
fn zero_width_temporal_cells_lower_to_boolean_without_mixed_target_syntax() {
    let zero = TemporalInterval::Closed(Interval::new(0, 0).unwrap());
    let formula = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: zero,
            operand: NodeId(0),
        }),
        node(K::Historically {
            interval: zero,
            operand: NodeId(1),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(2),
        }),
    ]);
    let mapped = export(&formula, &rows(PartialValue::True)).unwrap();
    assert_eq!(mapped.expression, "p");
    assert_eq!(mapped.section, "FTSPEC");
    assert_eq!(mapped.decision_horizon, 0);
}

// Trace: TC-173; FR-041-AC-2
#[test]
fn every_typed_refusal_projects_to_unavailable_truth_and_basis() {
    use tl_mltl::infinite::{Disposition, EvidenceBasis, ExecutionDisposition, TruthAvailability};
    let unsupported = SafetyExportError::UnboundedPast.result_axes();
    assert_eq!(
        unsupported,
        (
            Disposition::Unsupported,
            ExecutionDisposition::Unsupported,
            TruthAvailability::Unavailable,
            EvidenceBasis::Unavailable,
        )
    );
    let exhausted = SafetyExportError::ResourceIncomplete.result_axes();
    assert_eq!(
        exhausted,
        (
            Disposition::Failed,
            ExecutionDisposition::ResourceIncomplete,
            TruthAvailability::Unavailable,
            EvidenceBasis::Unavailable,
        )
    );
}

// Trace: TC-173; FR-041-AC-2
#[test]
fn unsupported_signal_and_conflicting_observation_refuse_without_artifacts() {
    let formula = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(0),
        }),
    ]);
    let observations = rows(PartialValue::True);
    let graph_id = formula.content_identity().unwrap();
    let request = PrefixRequest {
        formula: &formula,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &observations,
        limit: EvaluationLimit::default(),
    };
    let bad_catalog = SignalCatalogDocument::new(
        vec![OwnedSignalDeclaration::new(
            SignalId(1),
            "bad-name".to_owned(),
            SignalDomain::Boolean,
        )],
        vec![PropositionBinding::new(PropositionId(0), SignalId(1))],
    )
    .unwrap();
    assert_eq!(
        export_safety_monitor(&request, None, &bad_catalog, &contract(), 100),
        Err(SafetyExportError::Signal),
    );
    assert_eq!(
        export(&formula, &rows(PartialValue::Conflicting)),
        Err(SafetyExportError::PartialValuation),
    );
}

// Trace: TC-169, TC-170; FR-040-AC-1 and FR-040-AC-2
#[test]
fn bounded_future_violation_requires_the_whole_decision_horizon() {
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Future {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let mut false_rows = rows(PartialValue::False);
    let first_only = false_rows.clone();
    let mut second = rows(PartialValue::False).remove(0);
    second.position = 1;
    false_rows.push(second);
    for (observations, expected) in [
        (first_only.as_slice(), SafetyReplayDisposition::Mismatch),
        (false_rows.as_slice(), SafetyReplayDisposition::Refuted),
    ] {
        let graph_id = safety.content_identity().unwrap();
        let request = PrefixRequest {
            formula: &safety,
            graph_id: &graph_id,
            proposition_map_id: "map",
            propositions: &[PropositionId(0)],
            observations,
            limit: EvaluationLimit::default(),
        };
        let manifest = export_safety_monitor(&request, None, &catalog(), &contract(), 100).unwrap();
        assert_eq!(manifest.decision_horizon, 1);
        let observation = TargetStepObservation {
            target: &manifest.target,
            expression_sha256: &manifest.output_sha256,
            position: 0,
            verdict: false,
        };
        assert_eq!(
            replay_target_step(&manifest, &request, observation).unwrap(),
            expected
        );
    }
}

// Trace: TC-172, TC-173; FR-041-AC-1 and FR-041-AC-2
#[test]
fn tl216_origin_shape_partition_refuses_mixed_and_overdeep_past() {
    let mixed = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::StrongPrevious { operand: NodeId(1) }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(2),
        }),
    ]);
    assert_eq!(
        export(&mixed, &rows(PartialValue::True)),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::TargetOriginShapeUnverified(NodeId(2)),
        )),
    );
    let mut nodes = vec![node(K::Proposition {
        proposition: PropositionId(0),
    })];
    for index in 0..4 {
        nodes.push(node(K::Once {
            interval: closed(),
            operand: NodeId(index),
        }));
    }
    nodes.push(node(K::Globally {
        interval: open(),
        operand: NodeId(4),
    }));
    let deep = graph(nodes);
    assert_eq!(
        export(&deep, &rows(PartialValue::True)),
        Err(SafetyExportError::TargetOrigin(
            PastMappingError::TargetOriginShapeUnverified(NodeId(4)),
        )),
    );
}

// Trace: TC-170; FR-040-AC-2
#[test]
fn retained_c2po_steps_agree_with_the_exact_new_safety_body() {
    let retained: serde_json::Value =
        serde_json::from_slice(include_bytes!("../corpus/past-c2po-v1/manifest.json")).unwrap();
    let origin = contract();
    let source = include_bytes!("../corpus/past-c2po-v1/target-4.2/past.c2po");
    let output = include_bytes!("../corpus/past-c2po-v1/target-4.2/r2u2.stdout");
    let recorded_body = std::str::from_utf8(source)
        .unwrap()
        .split("PTSPEC")
        .nth(1)
        .unwrap()
        .split(';')
        .next()
        .unwrap()
        .trim();
    let safety = graph(vec![
        node(K::Proposition {
            proposition: PropositionId(0),
        }),
        node(K::Once {
            interval: closed(),
            operand: NodeId(0),
        }),
        node(K::Globally {
            interval: open(),
            operand: NodeId(1),
        }),
    ]);
    let observations: Vec<_> = retained["trace"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| {
            let mut observation = rows(if row["p"] == true {
                PartialValue::True
            } else {
                PartialValue::False
            })
            .remove(0);
            observation.position = u32::try_from(row["position"].as_u64().unwrap()).unwrap();
            observation
        })
        .collect();
    let graph_id = safety.content_identity().unwrap();
    let request = PrefixRequest {
        formula: &safety,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: &[PropositionId(0)],
        observations: &observations,
        limit: EvaluationLimit::default(),
    };
    let manifest = export_safety_monitor(&request, None, &catalog(), &origin, 100).unwrap();
    assert_eq!(manifest.expression, recorded_body);
    assert_eq!(manifest.section, "PTSPEC");
    let mut checked = 0;
    for line in std::str::from_utf8(output).unwrap().lines() {
        let Some((identity, verdict)) = line.split_once(',') else {
            panic!("invalid retained target line: {line}");
        };
        let Some(position) = identity.strip_prefix("0:") else {
            continue;
        };
        let position: u64 = position.parse().unwrap();
        if position >= u64::try_from(observations.len()).unwrap() {
            continue; // The retained monitor also emits its terminal flush step.
        }
        let verdict = match verdict {
            "T" => true,
            "F" => false,
            _ => panic!("invalid retained verdict: {verdict}"),
        };
        let step = TargetStepObservation {
            target: &manifest.target,
            expression_sha256: &manifest.output_sha256,
            position,
            verdict,
        };
        let expected = if position == 0 {
            SafetyReplayDisposition::Refuted
        } else {
            SafetyReplayDisposition::Inconclusive
        };
        assert_eq!(
            replay_target_step(&manifest, &request, step).unwrap(),
            expected
        );
        checked += 1;
    }
    assert_eq!(checked, 3);
}

// Trace: TC-172; FR-041-AC-1
#[test]
fn node_interval_and_placement_census_has_exact_mapped_or_refused_cells() {
    #[derive(Clone, Copy)]
    enum Placement {
        Inner,
        BooleanNested,
        Outer,
    }
    let placements = [Placement::Inner, Placement::BooleanNested, Placement::Outer];
    let atom = node(K::Proposition {
        proposition: PropositionId(0),
    });
    let bool_kinds = [
        K::False,
        K::True,
        K::Proposition {
            proposition: PropositionId(0),
        },
        K::Not { operand: NodeId(0) },
        K::And {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Or {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Implies {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::Equivalent {
            left: NodeId(0),
            right: NodeId(1),
        },
        K::StrongPrevious { operand: NodeId(0) },
    ];
    let build = |kind: K, placement: Placement| {
        let mut nodes = vec![atom, node(K::True), node(kind)];
        let inner = if matches!(placement, Placement::BooleanNested) {
            nodes.push(node(K::Not { operand: NodeId(2) }));
            NodeId(3)
        } else {
            NodeId(2)
        };
        if !matches!(placement, Placement::Outer) {
            nodes.push(node(K::Globally {
                interval: open(),
                operand: inner,
            }));
        }
        graph(nodes)
    };
    let mut checked = 0;
    for kind in bool_kinds {
        for placement in placements {
            let formula = build(kind, placement);
            let result = export(&formula, &rows(PartialValue::True));
            if matches!(placement, Placement::Outer) {
                assert_eq!(result, Err(SafetyExportError::UnsupportedShape), "{kind:?}");
            } else {
                assert!(result.is_ok(), "{kind:?}: {result:?}");
            }
            checked += 1;
        }
    }
    let intervals = [
        TemporalInterval::Closed(Interval::new(0, 0).unwrap()),
        TemporalInterval::Closed(Interval::new(0, 1).unwrap()),
        TemporalInterval::Closed(Interval::new(1, 1).unwrap()),
        TemporalInterval::Closed(Interval::new(1, 2).unwrap()),
        open(),
        TemporalInterval::Unbounded(UnboundedInterval::new(1)),
    ];
    for interval in intervals {
        let kinds = [
            K::Future {
                interval,
                operand: NodeId(0),
            },
            K::Globally {
                interval,
                operand: NodeId(0),
            },
            K::Until {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Release {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Once {
                interval,
                operand: NodeId(0),
            },
            K::Historically {
                interval,
                operand: NodeId(0),
            },
            K::Since {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Triggered {
                interval,
                left: NodeId(0),
                right: NodeId(1),
            },
        ];
        for (index, kind) in kinds.into_iter().enumerate() {
            for placement in placements {
                let formula = build(kind, placement);
                let result = export(&formula, &rows(PartialValue::True));
                let expected = match (interval, index, placement) {
                    (TemporalInterval::Closed(_), _, Placement::Outer) => {
                        Some(SafetyExportError::UnsupportedShape)
                    }
                    (TemporalInterval::Closed(bounds), 4..=7, _)
                        if bounds.start() != 0
                            || (bounds.end() != 0 && index != 4)
                            || bounds.end() > 1 =>
                    {
                        let operator = [
                            PastOperatorKind::Once,
                            PastOperatorKind::Historically,
                            PastOperatorKind::Since,
                            PastOperatorKind::Triggered,
                        ][index - 4];
                        Some(SafetyExportError::TargetOriginIntervalMismatch {
                            operator,
                            interval: bounds,
                        })
                    }
                    (TemporalInterval::Closed(_), _, _) => None,
                    (TemporalInterval::Unbounded(bounds), 1, Placement::Outer)
                        if bounds.start() == 0 =>
                    {
                        None
                    }
                    (TemporalInterval::Unbounded(_), 0 | 1, _) => {
                        Some(SafetyExportError::UnboundedLiveness)
                    }
                    (TemporalInterval::Unbounded(_), 2 | 3, _) => {
                        Some(SafetyExportError::UnboundedUntilRelease)
                    }
                    (TemporalInterval::Unbounded(_), 4..=7, _) => {
                        Some(SafetyExportError::UnboundedPast)
                    }
                    _ => unreachable!(),
                };
                match expected {
                    Some(error) => assert_eq!(result, Err(error), "{kind:?}"),
                    None => assert!(result.is_ok(), "{kind:?}: {result:?}"),
                }
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 171); // Nine noninterval nodes and eight interval nodes across placements.
}

// Trace: TC-172; FR-041-AC-1
#[test]
fn temporal_nesting_cross_product_classifies_target_context() {
    let atom = node(K::Proposition {
        proposition: PropositionId(0),
    });
    let intervals = [
        TemporalInterval::Closed(Interval::new(0, 0).unwrap()),
        TemporalInterval::Closed(Interval::new(0, 1).unwrap()),
        TemporalInterval::Closed(Interval::new(1, 1).unwrap()),
        TemporalInterval::Closed(Interval::new(1, 2).unwrap()),
        open(),
        TemporalInterval::Unbounded(UnboundedInterval::new(1)),
    ];
    let mut checked = 0;
    for future_context in [true, false] {
        for interval in intervals {
            let kinds = [
                K::Future {
                    interval,
                    operand: NodeId(0),
                },
                K::Globally {
                    interval,
                    operand: NodeId(0),
                },
                K::Until {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
                K::Release {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
                K::Once {
                    interval,
                    operand: NodeId(0),
                },
                K::Historically {
                    interval,
                    operand: NodeId(0),
                },
                K::Since {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
                K::Triggered {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
            ];
            for (index, kind) in kinds.into_iter().enumerate() {
                let parent = if future_context {
                    K::Future {
                        interval: TemporalInterval::Closed(Interval::new(0, 1).unwrap()),
                        operand: NodeId(2),
                    }
                } else {
                    K::Once {
                        interval: closed(),
                        operand: NodeId(2),
                    }
                };
                let formula = graph(vec![
                    atom,
                    node(K::True),
                    node(kind),
                    node(parent),
                    node(K::Globally {
                        interval: open(),
                        operand: NodeId(3),
                    }),
                ]);
                let result = export(&formula, &rows(PartialValue::True));
                let expected = match interval {
                    TemporalInterval::Unbounded(_) if index <= 1 => {
                        Some(SafetyExportError::UnboundedLiveness)
                    }
                    TemporalInterval::Unbounded(_) if index <= 3 => {
                        Some(SafetyExportError::UnboundedUntilRelease)
                    }
                    TemporalInterval::Unbounded(_) => Some(SafetyExportError::UnboundedPast),
                    TemporalInterval::Closed(bounds) if !future_context && index >= 4 => {
                        if index == 4 && bounds.start() == 0 && bounds.end() == 1 {
                            None
                        } else {
                            Some(SafetyExportError::TargetOrigin(
                                PastMappingError::TargetOriginShapeUnverified(NodeId(3)),
                            ))
                        }
                    }
                    TemporalInterval::Closed(bounds) if !future_context => {
                        (bounds.end() != 0).then_some(SafetyExportError::MixedTargetContext)
                    }
                    TemporalInterval::Closed(_) if index <= 3 => None,
                    TemporalInterval::Closed(bounds)
                        if bounds.start() != 0
                            || (bounds.end() != 0 && index != 4)
                            || bounds.end() > 1 =>
                    {
                        let operator = [
                            PastOperatorKind::Once,
                            PastOperatorKind::Historically,
                            PastOperatorKind::Since,
                            PastOperatorKind::Triggered,
                        ][index - 4];
                        Some(SafetyExportError::TargetOriginIntervalMismatch {
                            operator,
                            interval: bounds,
                        })
                    }
                    TemporalInterval::Closed(bounds) => {
                        (bounds.end() != 0).then_some(SafetyExportError::MixedTargetContext)
                    }
                };
                match expected {
                    Some(error) => assert_eq!(result, Err(error), "{future_context} {kind:?}"),
                    None => assert!(result.is_ok(), "{future_context} {kind:?}: {result:?}"),
                }
                checked += 1;
            }
        }
        for kind in [
            K::False,
            K::True,
            K::Proposition {
                proposition: PropositionId(0),
            },
            K::Not { operand: NodeId(0) },
            K::And {
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Or {
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Implies {
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Equivalent {
                left: NodeId(0),
                right: NodeId(1),
            },
            K::StrongPrevious { operand: NodeId(0) },
        ] {
            let parent = if future_context {
                K::Future {
                    interval: closed(),
                    operand: NodeId(2),
                }
            } else {
                K::Once {
                    interval: closed(),
                    operand: NodeId(2),
                }
            };
            let formula = graph(vec![
                atom,
                node(K::True),
                node(kind),
                node(parent),
                node(K::Globally {
                    interval: open(),
                    operand: NodeId(3),
                }),
            ]);
            let result = export(&formula, &rows(PartialValue::True));
            let expected = if matches!(kind, K::StrongPrevious { .. }) {
                if future_context {
                    Some(SafetyExportError::MixedTargetContext)
                } else {
                    Some(SafetyExportError::TargetOrigin(
                        PastMappingError::TargetOriginShapeUnverified(NodeId(3)),
                    ))
                }
            } else {
                None
            };
            match expected {
                Some(error) => assert_eq!(result, Err(error), "{future_context} {kind:?}"),
                None => assert!(result.is_ok(), "{future_context} {kind:?}: {result:?}"),
            }
            checked += 1;
        }
    }
    assert_eq!(checked, 114);
}
