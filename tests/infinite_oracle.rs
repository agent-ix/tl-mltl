#![cfg(feature = "infinite-trace")]

use tl_mltl::infinite::{
    evaluate_lasso, Disposition, EvaluationLimit, EvidenceClosure, LassoRequest,
};
use tl_oracle::{evaluate_documents, Limits, Verdict};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as K, Interval, LassoTraceDocument, NodeId, PartialValuation, PartialValue,
    PropositionId, SemanticProfile, TemporalInterval, TraceObservation, UnboundedInterval,
    ValuationEntry,
};

fn open(start: u32) -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(start))
}
fn closed(start: u32, end: u32) -> TemporalInterval {
    TemporalInterval::Closed(Interval::new(start, end).unwrap())
}
fn node(kind: K) -> InfiniteNode {
    InfiniteNode::new(kind)
}

fn graphs() -> Vec<(&'static str, InfiniteFormulaDocument)> {
    let mut cases = Vec::new();
    let p = || {
        node(K::Proposition {
            proposition: PropositionId(0),
        })
    };
    let q = || {
        node(K::Proposition {
            proposition: PropositionId(1),
        })
    };
    let unary = [
        (
            "F0",
            K::Future {
                interval: open(0),
                operand: NodeId(0),
            },
        ),
        (
            "F2",
            K::Future {
                interval: open(2),
                operand: NodeId(0),
            },
        ),
        (
            "G1",
            K::Globally {
                interval: open(1),
                operand: NodeId(0),
            },
        ),
        (
            "O0",
            K::Once {
                interval: open(0),
                operand: NodeId(0),
            },
        ),
        (
            "O2",
            K::Once {
                interval: open(2),
                operand: NodeId(0),
            },
        ),
        (
            "H1",
            K::Historically {
                interval: open(1),
                operand: NodeId(0),
            },
        ),
        (
            "Fb",
            K::Future {
                interval: closed(1, 2),
                operand: NodeId(0),
            },
        ),
        (
            "Gb",
            K::Globally {
                interval: closed(0, 2),
                operand: NodeId(0),
            },
        ),
        (
            "Ob",
            K::Once {
                interval: closed(1, 2),
                operand: NodeId(0),
            },
        ),
        (
            "Hb",
            K::Historically {
                interval: closed(1, 2),
                operand: NodeId(0),
            },
        ),
        ("Y", K::StrongPrevious { operand: NodeId(0) }),
    ];
    for (name, kind) in unary {
        cases.push((
            name,
            InfiniteFormulaDocument::new(
                SemanticProfile::InfiniteTraceV1,
                InfiniteClock::EventPosition,
                NodeId(1),
                vec![p(), node(kind)],
            )
            .unwrap(),
        ));
    }
    let binary = [
        (
            "U0",
            K::Until {
                interval: open(0),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "U2",
            K::Until {
                interval: open(2),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "R1",
            K::Release {
                interval: open(1),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "S0",
            K::Since {
                interval: open(0),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "S2",
            K::Since {
                interval: open(2),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "T1",
            K::Triggered {
                interval: open(1),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "Ub",
            K::Until {
                interval: closed(1, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "Rb",
            K::Release {
                interval: closed(0, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "Sb",
            K::Since {
                interval: closed(1, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
        (
            "Tb",
            K::Triggered {
                interval: closed(0, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
        ),
    ];
    for (name, kind) in binary {
        cases.push((
            name,
            InfiniteFormulaDocument::new(
                SemanticProfile::InfiniteTraceV1,
                InfiniteClock::EventPosition,
                NodeId(2),
                vec![p(), q(), node(kind)],
            )
            .unwrap(),
        ));
    }
    let nested = [
        (
            "F(Hp)",
            K::Historically {
                interval: open(0),
                operand: NodeId(0),
            },
            K::Future {
                interval: open(0),
                operand: NodeId(1),
            },
        ),
        (
            "H(Fp)",
            K::Future {
                interval: open(0),
                operand: NodeId(0),
            },
            K::Historically {
                interval: open(0),
                operand: NodeId(1),
            },
        ),
        (
            "G(Op)",
            K::Once {
                interval: open(0),
                operand: NodeId(0),
            },
            K::Globally {
                interval: open(0),
                operand: NodeId(1),
            },
        ),
        (
            "O(Gp)",
            K::Globally {
                interval: open(0),
                operand: NodeId(0),
            },
            K::Once {
                interval: open(0),
                operand: NodeId(1),
            },
        ),
        (
            "F(S)",
            K::Since {
                interval: open(1),
                left: NodeId(0),
                right: NodeId(1),
            },
            K::Future {
                interval: open(1),
                operand: NodeId(2),
            },
        ),
    ];
    for (name, inner, outer) in nested {
        let mut nodes = vec![p()];
        if name == "F(S)" {
            nodes.push(q());
        }
        nodes.push(node(inner));
        nodes.push(node(outer));
        let root = NodeId(u32::try_from(nodes.len() - 1).unwrap());
        cases.push((
            name,
            InfiniteFormulaDocument::new(
                SemanticProfile::InfiniteTraceV1,
                InfiniteClock::EventPosition,
                root,
                nodes,
            )
            .unwrap(),
        ));
    }
    cases
}

fn trace(prefix: &[[PartialValue; 2]], loop_cells: &[[PartialValue; 2]]) -> LassoTraceDocument {
    let props = vec![PropositionId(0), PropositionId(1)];
    let observation = |index: usize, values: [PartialValue; 2]| TraceObservation {
        position: u32::try_from(index).unwrap(),
        valuation: PartialValuation::new(
            "map".into(),
            &props,
            props
                .iter()
                .zip(values)
                .map(|(proposition, value)| ValuationEntry {
                    proposition: *proposition,
                    value,
                })
                .collect(),
        )
        .unwrap(),
    };
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        "map".into(),
        props.clone(),
        prefix
            .iter()
            .copied()
            .enumerate()
            .map(|(i, v)| observation(i, v))
            .collect(),
        loop_cells
            .iter()
            .copied()
            .enumerate()
            .map(|(i, v)| observation(prefix.len() + i, v))
            .collect(),
    )
    .unwrap()
}

// Trace: TC-141, TC-145, TC-146, TC-147, TC-148, TC-151, TC-175; FR-030-AC-1, FR-031-AC-1, FR-031-AC-2, FR-032-AC-1, FR-043-AC-1
#[test]
fn cross_compare_small_lassos() {
    use PartialValue::{Conflicting as C, False as F, Missing as M, True as T};
    let words = [
        trace(&[], &[[F, F]]),
        trace(&[], &[[T, F]]),
        trace(&[], &[[F, T], [T, F]]),
        trace(&[[T, F]], &[[F, T]]),
        trace(&[[F, F], [T, F]], &[[T, T], [F, F]]),
        trace(&[], &[[M, F]]),
        trace(&[], &[[C, T]]),
        trace(&[[M, T]], &[[T, C], [F, F]]),
    ];
    let formulas = graphs();
    assert_eq!(formulas.len(), 26);
    assert_eq!(words.len(), 8);
    let mut checked = 0;
    for (name, graph) in formulas {
        let graph_id = graph.content_identity().unwrap();
        for (word_index, word) in words.iter().enumerate() {
            let trace_id = word.content_identity().unwrap();
            for position in 0..8 {
                let actual = evaluate_lasso(&LassoRequest {
                    formula: &graph,
                    trace: word,
                    fairness: None,
                    evidence_closure: EvidenceClosure::Closed,
                    graph_id: &graph_id,
                    trace_id: &trace_id,
                    selected_position: position,
                    limit: EvaluationLimit::default(),
                })
                .unwrap();
                let oracle = evaluate_documents(
                    &graph,
                    word,
                    graph.formula().root(),
                    &[],
                    usize::try_from(position).unwrap(),
                    Limits::default(),
                )
                .unwrap();
                let expected = match oracle.verdict {
                    Verdict::Proved => Disposition::Proved,
                    Verdict::Refuted => Disposition::Refuted,
                    Verdict::Inconclusive => Disposition::Inconclusive,
                };
                assert_eq!(
                    actual.disposition, expected,
                    "{name}, word={word_index}, position={position}"
                );
                assert_eq!(
                    actual.admitted_completions, oracle.fair_completions as u64,
                    "{name}, word={word_index}, position={position}"
                );
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 26 * 8 * 8);
}

// Trace: TC-142, TC-153, TC-175, TC-193; FR-030-AC-1, FR-032-AC-2, FR-043-AC-1, FR-053-AC-1
#[test]
fn cross_compare_fairness_and_partial_completions() {
    use PartialValue::{Conflicting as C, False as F, Missing as M, True as T};
    let words = [
        trace(&[], &[[F, F]]),
        trace(&[], &[[M, F]]),
        trace(&[], &[[C, T]]),
        trace(&[], &[[F, T], [M, F]]),
        trace(&[[F, F]], &[[M, T], [F, F]]),
    ];
    let formulas = graphs();
    assert_eq!(formulas.len(), 26);
    assert_eq!(words.len(), 5);
    let mut checked = 0;
    for (name, graph) in formulas {
        let graph_id = graph.content_identity().unwrap();
        let fairness = FairnessPremisesDocument::new(
            &graph,
            graph_id.clone(),
            InfiniteClock::EventPosition,
            vec![NodeId(0)],
        )
        .unwrap();
        for (word_index, word) in words.iter().enumerate() {
            let trace_id = word.content_identity().unwrap();
            for position in 0..5 {
                let actual = evaluate_lasso(&LassoRequest {
                    formula: &graph,
                    trace: word,
                    fairness: Some(&fairness),
                    evidence_closure: EvidenceClosure::Closed,
                    graph_id: &graph_id,
                    trace_id: &trace_id,
                    selected_position: position,
                    limit: EvaluationLimit::default(),
                })
                .unwrap();
                let expected = evaluate_documents(
                    &graph,
                    word,
                    graph.formula().root(),
                    &[NodeId(0)],
                    usize::try_from(position).unwrap(),
                    Limits::default(),
                );
                match expected {
                    Ok(oracle) => {
                        let verdict = match oracle.verdict {
                            Verdict::Proved => Disposition::Proved,
                            Verdict::Refuted => Disposition::Refuted,
                            Verdict::Inconclusive => Disposition::Inconclusive,
                        };
                        assert_eq!(
                            actual.disposition, verdict,
                            "{name}, word={word_index}, position={position}"
                        );
                        assert_eq!(
                            actual.admitted_completions, oracle.fair_completions as u64,
                            "{name}, word={word_index}, position={position}"
                        );
                    }
                    Err(tl_oracle::OracleError::EmptyFairAdmission) => {
                        assert_eq!(
                            actual.disposition,
                            Disposition::Inconclusive,
                            "{name}, word={word_index}, position={position}"
                        );
                        assert_eq!(
                            actual.admitted_completions, 0,
                            "{name}, word={word_index}, position={position}"
                        );
                    }
                    Err(error) => panic!("unexpected oracle error {error:?}"),
                }
                checked += 1;
            }
        }
    }
    assert_eq!(checked, 26 * 5 * 5);
}
