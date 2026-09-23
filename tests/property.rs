use proptest::prelude::*;
use tl_mltl::{evaluate_closed, evaluate_prefix, EvaluationLimits, TruthValue};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn formula<'a>(profile: SemanticProfile, nodes: &'a [Node]) -> Formula<'a> {
    Formula::new(profile, NodeId((nodes.len() - 1) as u32), nodes).unwrap()
}

fn bounded_formula(kind: u8, interval: Interval) -> Vec<Node> {
    let proposition = Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    });
    match kind {
        0 => vec![proposition],
        1 => vec![proposition, Node::new(NodeKind::Not { operand: NodeId(0) })],
        2 => vec![
            proposition,
            Node::new(NodeKind::Future {
                interval,
                operand: NodeId(0),
            }),
        ],
        _ => vec![
            proposition,
            Node::new(NodeKind::Globally {
                interval,
                operand: NodeId(0),
            }),
        ],
    }
}

fn trace(bits: &[bool]) -> Vec<Vec<PropositionId>> {
    bits.iter()
        .copied()
        .map(|present| {
            if present {
                vec![PropositionId(0)]
            } else {
                vec![]
            }
        })
        .collect()
}

/// Independent closed-trace oracle for the deliberately small TC-032 grammar.
/// Keep this separate from the evaluator: the property is useful only when a
/// defect in the production traversal cannot make both sides agree.
fn closed_oracle(kind: u8, start: u32, end: u32, bits: &[bool]) -> TruthValue {
    let at = |time: u32| {
        bits.get(time as usize)
            .copied()
            .map_or(TruthValue::False, |present| {
                if present {
                    TruthValue::True
                } else {
                    TruthValue::False
                }
            })
    };
    match kind {
        0 => at(0),
        1 => match at(0) {
            TruthValue::True => TruthValue::False,
            TruthValue::False => TruthValue::True,
            TruthValue::Pending => unreachable!("closed oracle has no pending values"),
        },
        2 => {
            if (start..=end).any(|offset| at(offset) == TruthValue::True) {
                TruthValue::True
            } else {
                TruthValue::False
            }
        }
        _ => {
            if (start..=end).all(|offset| at(offset) == TruthValue::True) {
                TruthValue::True
            } else {
                TruthValue::False
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 512,
        ..ProptestConfig::default()
    })]

    // Trace: TC-032, FR-003-AC-2
    #[test]
    fn closed_prefix_matches_closed_evaluation_over_bounded_generated_inputs(
        kind in 0_u8..4,
        start in 0_u32..=2,
        end in 0_u32..=2,
        bits in prop::collection::vec(any::<bool>(), 0..=5),
    ) {
        let interval = Interval::new(start.min(end), start.max(end)).unwrap();
        let nodes = bounded_formula(kind, interval);
        let expected = closed_oracle(kind, start.min(end), start.max(end), &bits);
        let trace = trace(&bits);
        let closed = evaluate_closed(
            formula(SemanticProfile::ClosedTraceV1, &nodes),
            "generated",
            &trace,
            "generated-trace",
            EvaluationLimits::default(),
        ).unwrap();
        let prefix = evaluate_prefix(
            formula(SemanticProfile::OnlinePrefixV1, &nodes),
            "generated",
            &trace,
            "generated-trace",
            true,
            EvaluationLimits::default(),
        ).unwrap();
        prop_assert_eq!(closed.verdict, expected);
        prop_assert_eq!(prefix.verdict, expected);
        prop_assert_eq!(closed.verdict, prefix.verdict);
        prop_assert_eq!(closed.horizon, prefix.horizon);
    }
}

#[cfg(feature = "infinite-trace")]
mod v1_campaign {
    use std::cell::Cell;
    use std::collections::BTreeMap;

    use proptest::prelude::*;
    use proptest::test_runner::{Config, RngAlgorithm, TestCaseError, TestRng, TestRunner};
    use serde_json::{json, Value};
    use tl_mltl::infinite::{
        evaluate_lasso, evaluate_prefix_safety, Disposition, EvaluationLimit, EvidenceClosure,
        LassoRequest, PrefixRequest,
    };
    use tl_mltl::wire::common::{read_expected, OwnerLimits, OwnerUsage};
    use tl_mltl::{
        evaluate_closed, evaluate_past, ClockBinding, EvaluationLimits, PastEvaluationLimits,
        PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TruthValue,
    };
    use tl_syntax::{
        FairnessPremisesDocument, Formula, FormulaDocument, InfiniteClock, InfiniteFormulaDocument,
        InfiniteNode, InfiniteNodeKind as K, Interval, LassoTraceDocument, Node, NodeId, NodeKind,
        PartialValuation, PartialValue, PropositionId, SemanticProfile, SyntaxArtifactLimits,
        TemporalInterval, TraceObservation, UnboundedInterval, ValuationEntry,
    };

    const SEED: [u8; 32] = [0x45; 32];
    const CASES: u32 = 64;
    const MAP: &str = "v3-property-map";
    const P: PropositionId = PropositionId(0);
    const Q: PropositionId = PropositionId(1);
    const V1_REQUIREMENTS: &[&str] = &[
        include_str!("../spec/requirements/FR-027-infinite-trace-crate-boundary.md"),
        include_str!("../spec/requirements/FR-028-liveness-backend-registration-boundary.md"),
        include_str!("../spec/requirements/FR-029-infinite-trace-downstream-evidence.md"),
        include_str!("../spec/requirements/FR-030-infinite-possibility-semantics.md"),
        include_str!("../spec/requirements/FR-031-infinite-temporal-semantics.md"),
        include_str!("../spec/requirements/FR-032-lasso-fairness-admission.md"),
        include_str!("../spec/requirements/FR-033-infinite-settlement.md"),
        include_str!("../spec/requirements/FR-034-infinite-identity-limits.md"),
        include_str!("../spec/requirements/FR-038-past-c2po-export.md"),
        include_str!("../spec/requirements/FR-039-past-origin-contract.md"),
        include_str!("../spec/requirements/FR-040-infinite-safety-export.md"),
        include_str!("../spec/requirements/FR-041-c2po-refusal-partition.md"),
        include_str!("../spec/requirements/FR-042-r2u2-evidence-provenance.md"),
        include_str!("../spec/requirements/FR-043-independent-tl-oracle.md"),
        include_str!("../spec/requirements/FR-044-exhaustive-small-scope.md"),
        include_str!("../spec/requirements/FR-045-property-laws-and-roundtrips.md"),
        include_str!("../spec/requirements/FR-046-per-crate-fuzzing.md"),
        include_str!("../spec/requirements/FR-047-mutation-effectiveness.md"),
        include_str!("../spec/requirements/FR-048-bounded-kani-proofs.md"),
        include_str!("../spec/requirements/FR-049-embedded-and-miri-robustness.md"),
        include_str!("../spec/requirements/FR-050-llvm-coverage.md"),
        include_str!("../spec/requirements/FR-051-performance-regression.md"),
        include_str!("../spec/requirements/FR-052-live-r2u2-differential.md"),
        include_str!("../spec/requirements/FR-053-infinite-verification.md"),
        include_str!("../spec/requirements/FR-054-reproducible-v1-report.md"),
        include_str!("../spec/requirements/FR-055-verification-claim-boundary.md"),
    ];

    fn classification(id: &str) -> Option<(&'static str, &'static str)> {
        Some(match id {
            "FR-027-AC-1" | "FR-027-AC-2" | "FR-027-AC-3" => (
                "excluded",
                "feature-off and dependency-tree boundary: TC-138",
            ),
            "FR-028-AC-1" | "FR-028-AC-2" => (
                "example",
                "deployment_registry_routes_one_provider_and_refuses_duplicate",
            ),
            "FR-028-AC-3" => ("excluded", "bounded wire feature-set comparison: TC-138"),
            "FR-029-AC-1" | "FR-029-AC-3" => (
                "excluded",
                "dependency route and acceptance inspection: TC-088",
            ),
            "FR-029-AC-2" => ("example", "model_and_identity_refusals_keep_their_scope"),
            "FR-030-AC-1" => ("example", "cross_compare_small_lassos"),
            "FR-030-AC-2" => (
                "example",
                "closure_declaration_distinguishes_pending_from_indeterminate",
            ),
            "FR-030-AC-3" => (
                "example",
                "one_unknown_cell_uses_one_shared_completion_across_references_and_loops",
            ),
            "FR-031-AC-1" | "FR-031-AC-2" | "FR-031-AC-3" => ("property", "duality"),
            "FR-032-AC-1" => ("property", "lasso_unrolling"),
            "FR-032-AC-2" => ("property", "fairness_weakening"),
            "FR-032-AC-3" => (
                "example",
                "fairness_filters_completions_without_vacuous_proof",
            ),
            "FR-033-AC-1" => (
                "example",
                "exact_trace_evidence_names_all_fair_completions_and_replayable_examples",
            ),
            "FR-033-AC-2" => ("property", "finite_prefix_refutation"),
            "FR-033-AC-3" => ("example", "model_and_identity_refusals_keep_their_scope"),
            "FR-034-AC-1" => ("example", "model_and_identity_refusals_keep_their_scope"),
            "FR-034-AC-2" | "FR-034-AC-3" => (
                "example",
                "exact_work_limit_succeeds_and_one_less_is_resource_incomplete",
            ),
            "FR-045-AC-1" => ("property", "duality"),
            "FR-045-AC-2" => ("property", "strict_round_trips"),
            _ if id.starts_with("FR-038-")
                || id.starts_with("FR-039-")
                || id.starts_with("FR-040-")
                || id.starts_with("FR-041-")
                || id.starts_with("FR-042-") =>
            {
                (
                    "excluded",
                    "R2U2 and C2PO example/interop lane: TC-165 through TC-174",
                )
            }
            _ if id.starts_with("FR-043-") => (
                "excluded",
                "independent oracle qualification is V1: TC-175/176",
            ),
            _ if id.starts_with("FR-044-") => {
                ("excluded", "exhaustive population is V2: TC-177/178")
            }
            _ if id.starts_with("FR-046-") => {
                ("excluded", "per-crate fuzz population is V4: TC-181/182")
            }
            _ if id.starts_with("FR-047-") => ("excluded", "mutation population is V5: TC-183/184"),
            _ if id.starts_with("FR-048-") => {
                ("excluded", "bounded proof population is V6: TC-185/186")
            }
            _ if id.starts_with("FR-049-") => (
                "excluded",
                "embedded, Miri and limit checks are V7: TC-187/188",
            ),
            _ if id.starts_with("FR-050-") => ("excluded", "coverage measurement is V8: TC-189"),
            _ if id.starts_with("FR-051-") => ("excluded", "paired performance is V9: TC-190"),
            _ if id.starts_with("FR-052-") => {
                ("excluded", "live R2U2 differential is V10: TC-191/192")
            }
            _ if id.starts_with("FR-053-") => (
                "excluded",
                "generated infinite population is V11: TC-193/194",
            ),
            _ if id.starts_with("FR-054-") || id.starts_with("FR-055-") => (
                "excluded",
                "campaign report and claim controls: TC-195 through TC-199",
            ),
            _ => return None,
        })
    }

    fn classify_all() -> BTreeMap<String, Value> {
        let mut classified = BTreeMap::new();
        for document in V1_REQUIREMENTS {
            let mut found = 0;
            for line in document.lines() {
                let Some(id) = line
                    .strip_prefix("| FR-")
                    .and_then(|rest| rest.split('|').next())
                else {
                    continue;
                };
                let id = format!("FR-{}", id.trim());
                if !id.contains("-AC-") {
                    continue;
                }
                let (kind, evidence) = classification(&id)
                    .unwrap_or_else(|| panic!("unclassified V1 acceptance criterion: {id}"));
                if kind == "example" {
                    let sources = concat!(
                        include_str!("infinite_trace.rs"),
                        include_str!("infinite_oracle.rs")
                    );
                    assert!(
                        sources.contains(&format!("fn {evidence}()")),
                        "missing example: {evidence}"
                    );
                }
                assert!(!evidence.is_empty());
                assert!(
                    classified
                        .insert(
                            id.clone(),
                            json!({
                                "kind": kind, "evidence": evidence
                            })
                        )
                        .is_none(),
                    "duplicate V1 criterion: {id}"
                );
                found += 1;
            }
            assert!(found > 0, "V1 requirement lacks acceptance criteria");
        }
        assert_eq!(classified.len(), 61, "V1 criterion population changed");
        classified
    }

    fn graph(kinds: Vec<K>) -> InfiniteFormulaDocument {
        let root = NodeId(u32::try_from(kinds.len() - 1).unwrap());
        InfiniteFormulaDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            root,
            kinds.into_iter().map(InfiniteNode::new).collect(),
        )
        .unwrap()
    }

    fn open() -> TemporalInterval {
        TemporalInterval::Unbounded(UnboundedInterval::new(0))
    }

    fn p() -> K {
        K::Proposition { proposition: P }
    }

    fn q() -> K {
        K::Proposition { proposition: Q }
    }

    fn observation(position: usize, value: PartialValue) -> TraceObservation {
        TraceObservation {
            position: u32::try_from(position).unwrap(),
            valuation: PartialValuation::new(
                MAP.to_owned(),
                &[P],
                vec![ValuationEntry {
                    proposition: P,
                    value,
                }],
            )
            .unwrap(),
        }
    }

    fn lasso(prefix: &[PartialValue], loop_values: &[PartialValue]) -> LassoTraceDocument {
        LassoTraceDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            MAP.to_owned(),
            vec![P],
            prefix
                .iter()
                .enumerate()
                .map(|(at, value)| observation(at, *value))
                .collect(),
            loop_values
                .iter()
                .enumerate()
                .map(|(at, value)| observation(prefix.len() + at, *value))
                .collect(),
        )
        .unwrap()
    }

    fn two_atom_lasso(p_values: [bool; 3], q_values: [bool; 3]) -> LassoTraceDocument {
        let rows = (0..3)
            .map(|at| TraceObservation {
                position: u32::try_from(at).unwrap(),
                valuation: PartialValuation::new(
                    MAP.to_owned(),
                    &[P, Q],
                    vec![
                        ValuationEntry {
                            proposition: P,
                            value: bool_value(p_values[at]),
                        },
                        ValuationEntry {
                            proposition: Q,
                            value: bool_value(q_values[at]),
                        },
                    ],
                )
                .unwrap(),
            })
            .collect::<Vec<_>>();
        LassoTraceDocument::new(
            SemanticProfile::InfiniteTraceV1,
            InfiniteClock::EventPosition,
            MAP.to_owned(),
            vec![P, Q],
            rows[..1].to_vec(),
            rows[1..].to_vec(),
        )
        .unwrap()
    }

    fn evaluate(
        formula: &InfiniteFormulaDocument,
        trace: &LassoTraceDocument,
        fairness: Option<&FairnessPremisesDocument>,
        position: u64,
    ) -> tl_mltl::infinite::InfiniteResult {
        let graph_id = formula.content_identity().unwrap();
        let trace_id = trace.content_identity().unwrap();
        evaluate_lasso(&LassoRequest {
            formula,
            trace,
            fairness,
            evidence_closure: EvidenceClosure::Closed,
            graph_id: &graph_id,
            trace_id: &trace_id,
            selected_position: position,
            limit: EvaluationLimit::default(),
        })
        .unwrap()
    }

    fn bool_value(value: bool) -> PartialValue {
        if value {
            PartialValue::True
        } else {
            PartialValue::False
        }
    }

    fn flip(value: Disposition) -> Disposition {
        match value {
            Disposition::Proved => Disposition::Refuted,
            Disposition::Refuted => Disposition::Proved,
            _ => panic!("complete lasso did not settle"),
        }
    }

    // TC-179 / FR-045-AC-1: every comparison calls the production evaluator.
    // The premise is pointwise Boolean duality over the same infinite word.
    fn duality_law(bits: [bool; 3], inject_fault: bool) -> Result<(), TestCaseError> {
        let trace = lasso(
            &[bool_value(bits[0])],
            &[bool_value(bits[1]), bool_value(bits[2])],
        );
        let future = graph(vec![
            p(),
            K::Future {
                interval: open(),
                operand: NodeId(0),
            },
        ]);
        let not_globally_not = graph(vec![
            p(),
            K::Not { operand: NodeId(0) },
            K::Globally {
                interval: open(),
                operand: NodeId(1),
            },
            K::Not { operand: NodeId(2) },
        ]);
        let once = graph(vec![
            p(),
            K::Once {
                interval: open(),
                operand: NodeId(0),
            },
        ]);
        let not_historically_not = graph(vec![
            p(),
            K::Not { operand: NodeId(0) },
            K::Historically {
                interval: open(),
                operand: NodeId(1),
            },
            K::Not { operand: NodeId(2) },
        ]);
        for position in 0..7 {
            let expected = evaluate(&future, &trace, None, position).disposition;
            let actual = evaluate(&not_globally_not, &trace, None, position).disposition;
            let actual = if inject_fault { flip(actual) } else { actual };
            prop_assert_eq!(expected, actual);
            prop_assert_eq!(
                evaluate(&once, &trace, None, position).disposition,
                evaluate(&not_historically_not, &trace, None, position).disposition
            );
        }
        Ok(())
    }

    fn binary_duality_law(p_values: [bool; 3], q_values: [bool; 3]) -> Result<(), TestCaseError> {
        let trace = two_atom_lasso(p_values, q_values);
        let until = graph(vec![
            p(),
            q(),
            K::Until {
                interval: open(),
                left: NodeId(0),
                right: NodeId(1),
            },
        ]);
        let not_release_not = graph(vec![
            p(),
            q(),
            K::Not { operand: NodeId(0) },
            K::Not { operand: NodeId(1) },
            K::Release {
                interval: open(),
                left: NodeId(2),
                right: NodeId(3),
            },
            K::Not { operand: NodeId(4) },
        ]);
        let since = graph(vec![
            p(),
            q(),
            K::Since {
                interval: open(),
                left: NodeId(0),
                right: NodeId(1),
            },
        ]);
        let not_triggered_not = graph(vec![
            p(),
            q(),
            K::Not { operand: NodeId(0) },
            K::Not { operand: NodeId(1) },
            K::Triggered {
                interval: open(),
                left: NodeId(2),
                right: NodeId(3),
            },
            K::Not { operand: NodeId(4) },
        ]);
        for position in 0..7 {
            prop_assert_eq!(
                evaluate(&until, &trace, None, position).disposition,
                evaluate(&not_release_not, &trace, None, position).disposition
            );
            prop_assert_eq!(
                evaluate(&since, &trace, None, position).disposition,
                evaluate(&not_triggered_not, &trace, None, position).disposition
            );
        }
        Ok(())
    }

    fn embedding_law(bits: [bool; 3]) -> Result<(), TestCaseError> {
        let interval = Interval::new(0, 2).unwrap();
        let nodes = [
            Node::new(NodeKind::Proposition { proposition: P }),
            Node::new(NodeKind::Future {
                interval,
                operand: NodeId(0),
            }),
        ];
        let finite = evaluate_closed(
            Formula::new(SemanticProfile::ClosedTraceV1, NodeId(1), &nodes).unwrap(),
            "v3-bounded",
            &bits
                .iter()
                .map(|value| if *value { vec![P] } else { vec![] })
                .collect::<Vec<_>>(),
            "v3-word",
            EvaluationLimits::default(),
        )
        .unwrap();
        let infinite = graph(vec![
            p(),
            K::Future {
                interval: TemporalInterval::Closed(interval),
                operand: NodeId(0),
            },
        ]);
        let trace = lasso(
            &[bool_value(bits[0])],
            &[bool_value(bits[1]), bool_value(bits[2])],
        );
        let expected = if finite.verdict == TruthValue::True {
            Disposition::Proved
        } else {
            Disposition::Refuted
        };
        prop_assert_eq!(evaluate(&infinite, &trace, None, 0).disposition, expected);
        Ok(())
    }

    fn unrolling_law(bits: [bool; 3]) -> Result<(), TestCaseError> {
        let prefix = [bool_value(bits[0])];
        let repeating = [bool_value(bits[1]), bool_value(bits[2])];
        let original = lasso(&prefix, &repeating);
        let unfolded = lasso(&[prefix[0], repeating[0], repeating[1]], &repeating);
        let mixed = graph(vec![
            p(),
            K::Once {
                interval: open(),
                operand: NodeId(0),
            },
            K::Future {
                interval: open(),
                operand: NodeId(1),
            },
        ]);
        for position in 0..9 {
            prop_assert_eq!(
                evaluate(&mixed, &original, None, position).disposition,
                evaluate(&mixed, &unfolded, None, position).disposition
            );
        }
        Ok(())
    }

    fn fairness_law(bits: [bool; 3]) -> Result<(), TestCaseError> {
        let formula = graph(vec![p()]);
        let trace = lasso(
            &[bool_value(bits[0])],
            &[bool_value(bits[1]), bool_value(bits[2])],
        );
        let fairness = FairnessPremisesDocument::new(
            &formula,
            formula.content_identity().unwrap(),
            InfiniteClock::EventPosition,
            vec![NodeId(0)],
        )
        .unwrap();
        let weak = evaluate(&formula, &trace, None, 0);
        let strong = evaluate(&formula, &trace, Some(&fairness), 0);
        prop_assert!(weak.admitted_completions >= strong.admitted_completions);
        if strong.admitted_completions > 0 {
            prop_assert_eq!(weak.disposition, strong.disposition);
        }
        Ok(())
    }

    fn monotonicity_law(bits: [bool; 3]) -> Result<(), TestCaseError> {
        let formula = graph(vec![
            p(),
            K::Future {
                interval: open(),
                operand: NodeId(0),
            },
        ]);
        let partial = lasso(
            &[bool_value(bits[0])],
            &[PartialValue::Missing, bool_value(bits[2])],
        );
        let settled_true = lasso(
            &[bool_value(bits[0])],
            &[PartialValue::True, bool_value(bits[2])],
        );
        let settled_false = lasso(
            &[bool_value(bits[0])],
            &[PartialValue::False, bool_value(bits[2])],
        );
        let prior = evaluate(&formula, &partial, None, 0);
        for refinement in [&settled_true, &settled_false] {
            let after = evaluate(&formula, refinement, None, 0);
            prop_assert!(prior.admitted_completions >= after.admitted_completions);
            if matches!(
                prior.disposition,
                Disposition::Proved | Disposition::Refuted
            ) {
                prop_assert_eq!(prior.disposition, after.disposition);
            }
        }
        Ok(())
    }

    fn prefix_refutation_law(loop_value: bool) -> Result<(), TestCaseError> {
        let formula = graph(vec![
            p(),
            K::Globally {
                interval: open(),
                operand: NodeId(0),
            },
        ]);
        let trace = lasso(&[PartialValue::False], &[bool_value(loop_value)]);
        let graph_id = formula.content_identity().unwrap();
        let prefix = evaluate_prefix_safety(&PrefixRequest {
            formula: &formula,
            graph_id: &graph_id,
            proposition_map_id: MAP,
            propositions: trace.propositions(),
            observations: trace.prefix(),
            limit: EvaluationLimit::default(),
        })
        .unwrap();
        prop_assert_eq!(prefix.disposition, Disposition::Refuted);
        prop_assert_eq!(
            evaluate(&formula, &trace, None, 0).disposition,
            Disposition::Refuted
        );
        Ok(())
    }

    fn reject_unknown_and_ordered<T>(value: &T, reader: impl Fn(&[u8]) -> bool) -> usize
    where
        T: serde::Serialize,
    {
        let bytes = serde_json::to_vec(value).unwrap();
        assert!(reader(&bytes));
        let mut wire: Value = serde_json::from_slice(&bytes).unwrap();
        wire.as_object_mut()
            .unwrap()
            .insert("unknownField".to_owned(), json!(true));
        assert!(!reader(&serde_json::to_vec(&wire).unwrap()));
        let mut fields: Vec<_> = wire
            .as_object()
            .unwrap()
            .iter()
            .map(|(name, value)| (name.clone(), value.clone()))
            .collect();
        fields.retain(|(name, _)| name != "unknownField");
        fields.reverse();
        let reordered = format!(
            "{{{}}}",
            fields
                .iter()
                .map(|(name, value)| format!("{}:{}", json!(name), value))
                .collect::<Vec<_>>()
                .join(",")
        );
        assert!(!reader(reordered.as_bytes()));
        3
    }

    fn wire_round_trips() -> usize {
        let limits = SyntaxArtifactLimits::default();
        let finite = FormulaDocument::new(
            SemanticProfile::ClosedTraceV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition { proposition: P })],
        )
        .unwrap();
        let finite_v2 = FormulaDocument::new_v2(
            SemanticProfile::ClosedTraceV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition { proposition: P })],
        )
        .unwrap();
        let infinite = graph(vec![p()]);
        let trace = lasso(&[PartialValue::Missing], &[PartialValue::True]);
        let fairness = FairnessPremisesDocument::new(
            &infinite,
            infinite.content_identity().unwrap(),
            InfiniteClock::EventPosition,
            vec![NodeId(0)],
        )
        .unwrap();
        let mut checked = 0;
        for document in [&finite, &finite_v2] {
            checked += reject_unknown_and_ordered(document, |bytes| {
                FormulaDocument::from_json_bytes(bytes, limits).is_ok()
            });
        }
        checked += reject_unknown_and_ordered(&infinite, |bytes| {
            InfiniteFormulaDocument::from_json_bytes(bytes, limits).is_ok()
        });
        checked += reject_unknown_and_ordered(&trace, |bytes| {
            LassoTraceDocument::from_json_bytes(bytes, limits).is_ok()
        });
        checked += reject_unknown_and_ordered(&fairness, |bytes| {
            FairnessPremisesDocument::from_json_bytes(bytes, limits, &infinite).is_ok()
        });
        checked += reject_unknown_and_ordered(&trace.prefix()[0].valuation, |bytes| {
            PartialValuation::from_json_bytes(bytes, limits).is_ok()
        });
        let nodes = [Node::new(NodeKind::Proposition { proposition: P })];
        let result = evaluate_closed(
            Formula::new(SemanticProfile::ClosedTraceV1, NodeId(0), &nodes).unwrap(),
            "v3-result",
            &[vec![P]],
            "v3-trace",
            EvaluationLimits::default(),
        )
        .unwrap();
        checked += reject_unknown_and_ordered(&result, |bytes| {
            read_expected(bytes, &result, OwnerLimits::default(), |_, _| {
                Ok(OwnerUsage::default())
            })
            .is_ok()
        });
        let history = PositionHistoryDocument::new(
            "v3-history",
            1,
            0,
            1,
            Some(ClockBinding::EventPosition),
            vec![
                PositionObservation::new(0, vec![], None),
                PositionObservation::new(1, vec![P], None),
            ],
        )
        .unwrap();
        let past_nodes = [
            Node::new(NodeKind::Proposition { proposition: P }),
            Node::new(NodeKind::Once {
                interval: Interval::new(0, 1).unwrap(),
                operand: NodeId(0),
            }),
        ];
        let past = evaluate_past(
            Formula::new(
                SemanticProfile::OriginCompleteHistoryV1,
                NodeId(1),
                &past_nodes,
            )
            .unwrap(),
            "v3-past",
            &history,
            1,
            MAP,
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits::default(),
        )
        .unwrap();
        checked += reject_unknown_and_ordered(&past, |bytes| {
            read_expected(bytes, &past, OwnerLimits::default(), |value, _| {
                value.validate_with_predecessor(None).map_err(|_| {
                    tl_mltl::wire::OwnerReadError::new(
                        tl_mltl::wire::OwnerReadErrorCode::InvalidJson,
                        "pastResult",
                        OwnerUsage::default(),
                    )
                })?;
                Ok(OwnerUsage::default())
            })
            .is_ok()
        });
        checked
    }

    // Trace: TC-179, TC-180, FR-045-AC-1, FR-045-AC-2, NFR-008-AC-1.
    #[test]
    fn native_semantic_laws_and_strict_round_trips() {
        let classified = classify_all();
        let calls = Cell::new(0_u32);
        let mut runner = TestRunner::new_with_rng(
            Config {
                cases: CASES,
                max_global_rejects: 0,
                max_local_rejects: 0,
                failure_persistence: None,
                ..Config::default()
            },
            TestRng::from_seed(RngAlgorithm::ChaCha, &SEED),
        );
        runner
            .run(
                &(
                    any::<bool>(),
                    any::<bool>(),
                    any::<bool>(),
                    any::<bool>(),
                    any::<bool>(),
                    any::<bool>(),
                ),
                |(a, b, c, d, e, f)| {
                    let bits = [a, b, c];
                    duality_law(bits, false)?;
                    binary_duality_law(bits, [d, e, f])?;
                    embedding_law(bits)?;
                    unrolling_law(bits)?;
                    fairness_law(bits)?;
                    monotonicity_law(bits)?;
                    prefix_refutation_law(c)?;
                    calls.set(calls.get() + 1);
                    Ok(())
                },
            )
            .unwrap();
        assert_eq!(
            calls.get(),
            CASES,
            "a rejected or omitted generator case is incomplete"
        );
        let wire_checks = wire_round_trips();
        assert_eq!(wire_checks, 24);
        let marker = json!({
            "schema": "tl-mltl.semantic-properties/v1",
            "scope": "tl_mltl_v1_semantic_laws_and_owner_wires",
            "seed_hex": "4545454545454545454545454545454545454545454545454545454545454545",
            "generated": CASES,
            "accepted": calls.get(),
            "rejected": 0,
            "law_cases": {
                "duality": calls.get(),
                "bounded_embedding": calls.get(),
                "lasso_unrolling": calls.get(),
                "fairness_weakening": calls.get(),
                "partial_information_monotonicity": calls.get(),
                "finite_prefix_refutation": calls.get()
            },
            "wire_checks": wire_checks,
            "classifications": classified,
            "rewrite_equivalence_owner": "tl-rewrite"
        });
        println!("TL_CAMPAIGN_PROPERTIES {marker}");
    }

    // A seeded fault uses the same production-law assertion and shrinking path.
    // Trace: TC-180, FR-045-AC-2.
    #[test]
    fn seeded_law_fault_is_detected() {
        let mut runner = TestRunner::new_with_rng(
            Config {
                cases: 8,
                failure_persistence: None,
                ..Config::default()
            },
            TestRng::from_seed(RngAlgorithm::ChaCha, &[0x46; 32]),
        );
        assert!(runner
            .run(
                &(any::<bool>(), any::<bool>(), any::<bool>()),
                |(a, b, c)| { duality_law([a, b, c], true) }
            )
            .is_err());
        let finite = graph(vec![p()]);
        let bytes = finite.canonical_json_bytes().unwrap();
        let mut changed: Value = serde_json::from_slice(&bytes).unwrap();
        changed["unknownField"] = json!(1);
        assert!(InfiniteFormulaDocument::from_json_bytes(
            &serde_json::to_vec(&changed).unwrap(),
            SyntaxArtifactLimits::default()
        )
        .is_err());
    }
}
