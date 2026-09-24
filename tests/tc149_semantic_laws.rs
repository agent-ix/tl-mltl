//! Generated infinite temporal laws over complete two-proposition lassos.

#![cfg(feature = "infinite-trace")]

use tl_mltl::infinite::{
    evaluate_lasso, Disposition, EvaluationLimit, EvidenceClosure, LassoRequest,
};
use tl_oracle::{evaluate_documents, Limits, Verdict};
use tl_syntax::{
    lower_infinite_future, FairnessPremisesDocument, FutureKind, InfiniteClock,
    InfiniteFormulaDocument, InfiniteNode, InfiniteNodeKind as K, Interval, LassoTraceDocument,
    NodeId, PartialValuation, PartialValue, PropositionId, SemanticProfile, TemporalInterval,
    TraceObservation, UnboundedInterval, ValuationEntry,
};

#[derive(Clone, Copy, Debug)]
enum Op {
    Future,
    Globally,
    Until,
    Release,
    Once,
    Historically,
    Since,
    Triggered,
}

impl Op {
    fn node(self, interval: TemporalInterval, left: NodeId, right: NodeId) -> K {
        match self {
            Self::Future => K::Future {
                interval,
                operand: left,
            },
            Self::Globally => K::Globally {
                interval,
                operand: left,
            },
            Self::Until => K::Until {
                interval,
                left,
                right,
            },
            Self::Release => K::Release {
                interval,
                left,
                right,
            },
            Self::Once => K::Once {
                interval,
                operand: left,
            },
            Self::Historically => K::Historically {
                interval,
                operand: left,
            },
            Self::Since => K::Since {
                interval,
                left,
                right,
            },
            Self::Triggered => K::Triggered {
                interval,
                left,
                right,
            },
        }
    }
}

fn open(start: u32) -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(start))
}

fn closed(start: u32, end: u32) -> TemporalInterval {
    TemporalInterval::Closed(Interval::new(start, end).unwrap())
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

fn atoms() -> Vec<K> {
    vec![
        K::Proposition {
            proposition: PropositionId(0),
        },
        K::Proposition {
            proposition: PropositionId(1),
        },
        K::Not { operand: NodeId(0) },
        K::Not { operand: NodeId(1) },
    ]
}

fn temporal(
    op: Op,
    interval: TemporalInterval,
    negate_inputs: bool,
    negate_result: bool,
) -> InfiniteFormulaDocument {
    let mut kinds = atoms();
    let (left, right) = if negate_inputs {
        (NodeId(2), NodeId(3))
    } else {
        (NodeId(0), NodeId(1))
    };
    kinds.push(op.node(interval, left, right));
    if negate_result {
        kinds.push(K::Not { operand: NodeId(4) });
    }
    graph(kinds)
}

fn boolean_dual(left_wrong: bool) -> InfiniteFormulaDocument {
    let mut kinds = atoms();
    if left_wrong {
        kinds.push(K::And {
            left: NodeId(2),
            right: NodeId(3),
        });
    } else {
        kinds.push(K::Or {
            left: NodeId(2),
            right: NodeId(3),
        });
    }
    graph(kinds)
}

fn not_and() -> InfiniteFormulaDocument {
    let mut kinds = atoms();
    kinds.push(K::And {
        left: NodeId(0),
        right: NodeId(1),
    });
    kinds.push(K::Not { operand: NodeId(4) });
    graph(kinds)
}

fn lowered(kind: FutureKind, interval: TemporalInterval) -> InfiniteFormulaDocument {
    let generated =
        lower_infinite_future(kind, NodeId(0), NodeId(1), interval, NodeId(2), None).unwrap();
    let mut nodes = atoms()
        .into_iter()
        .take(2)
        .map(InfiniteNode::new)
        .collect::<Vec<_>>();
    nodes.extend(generated);
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        NodeId(4),
        nodes,
    )
    .unwrap()
}

struct CompleteWord {
    prefix: Vec<[bool; 2]>,
    cycle: Vec<[bool; 2]>,
    trace: LassoTraceDocument,
}

impl CompleteWord {
    fn new(prefix: Vec<[bool; 2]>, cycle: Vec<[bool; 2]>) -> Self {
        let props = vec![PropositionId(0), PropositionId(1)];
        let observation = |position: usize, values: [bool; 2]| TraceObservation {
            position: u32::try_from(position).unwrap(),
            valuation: PartialValuation::new(
                "map".into(),
                &props,
                props
                    .iter()
                    .zip(values)
                    .map(|(proposition, value)| ValuationEntry {
                        proposition: *proposition,
                        value: if value {
                            PartialValue::True
                        } else {
                            PartialValue::False
                        },
                    })
                    .collect(),
            )
            .unwrap(),
        };
        let trace = LassoTraceDocument::new(
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
            cycle
                .iter()
                .copied()
                .enumerate()
                .map(|(i, v)| observation(prefix.len() + i, v))
                .collect(),
        )
        .unwrap();
        Self {
            prefix,
            cycle,
            trace,
        }
    }

    fn at(&self, time: usize, proposition: usize) -> bool {
        if time < self.prefix.len() {
            self.prefix[time][proposition]
        } else {
            self.cycle[(time - self.prefix.len()) % self.cycle.len()][proposition]
        }
    }

    fn denotation(&self, op: Op, start: usize, upper: Option<usize>, anchor: usize) -> bool {
        // QSL [a,*] is the set of natural offsets d >= a. A complete lasso
        // needs one full cycle after both the lower bound and finite prefix.
        let future_end = upper.map_or_else(
            || self.prefix.len().max(anchor + start) + self.cycle.len() - anchor,
            |end| end + 1,
        );
        let past_end = upper.unwrap_or(anchor).min(anchor);
        let future = |p: usize, offset: usize| self.at(anchor + offset, p);
        let past = |p: usize, offset: usize| self.at(anchor - offset, p);
        match op {
            Op::Future => (start..future_end).any(|d| future(0, d)),
            Op::Globally => (start..future_end).all(|d| future(0, d)),
            Op::Until => (start..future_end)
                .any(|d| future(1, d) && (0..d).all(|earlier| future(0, earlier))),
            Op::Release => !(start..future_end)
                .any(|d| !future(1, d) && (0..d).all(|earlier| !future(0, earlier))),
            Op::Once => (start..=past_end).any(|d| past(0, d)),
            Op::Historically => (start..=past_end).all(|d| past(0, d)),
            Op::Since => {
                (start..=past_end).any(|d| past(1, d) && (0..d).all(|earlier| past(0, earlier)))
            }
            Op::Triggered => {
                !(start..=past_end).any(|d| !past(1, d) && (0..d).all(|earlier| !past(0, earlier)))
            }
        }
    }
}

fn evaluate(formula: &InfiniteFormulaDocument, word: &CompleteWord, anchor: usize) -> bool {
    let graph_id = formula.content_identity().unwrap();
    let trace_id = word.trace.content_identity().unwrap();
    let actual = evaluate_lasso(&LassoRequest {
        formula,
        trace: &word.trace,
        fairness: None::<&FairnessPremisesDocument>,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: u64::try_from(anchor).unwrap(),
        limit: EvaluationLimit::default(),
    })
    .unwrap();
    let oracle = evaluate_documents(
        formula,
        &word.trace,
        formula.formula().root(),
        &[],
        anchor,
        Limits::default(),
    )
    .unwrap();
    let expected = match oracle.verdict {
        Verdict::Proved => true,
        Verdict::Refuted => false,
        Verdict::Inconclusive => panic!("complete word gave inconclusive oracle verdict"),
    };
    assert_eq!(actual.admitted_completions, 1);
    let actual = match actual.disposition {
        Disposition::Proved => true,
        Disposition::Refuted => false,
        Disposition::Inconclusive => panic!("complete word gave inconclusive provider verdict"),
        Disposition::Unsupported => panic!("complete word gave unsupported provider verdict"),
        Disposition::Failed => panic!("complete word gave failed provider verdict"),
    };
    assert_eq!(
        actual, expected,
        "provider disagreed with independent oracle"
    );
    actual
}

fn words() -> Vec<CompleteWord> {
    let mut words = Vec::new();
    for prefix_len in 0..=1_usize {
        for cycle_len in 1..=2_usize {
            let positions = prefix_len + cycle_len;
            for bits in 0..4_usize.pow(u32::try_from(positions).unwrap()) {
                let cells = (0..positions)
                    .map(|position| {
                        let cell = (bits >> (position * 2)) & 3;
                        [cell & 1 != 0, cell & 2 != 0]
                    })
                    .collect::<Vec<_>>();
                words.push(CompleteWord::new(
                    cells[..prefix_len].to_vec(),
                    cells[prefix_len..].to_vec(),
                ));
            }
        }
    }
    assert_eq!(words.len(), 100);
    words
}

// Trace: TC-149, FR-031-AC-3
#[test]
fn generated_duals_lowering_and_qsl_unbounded_denotations_hold() {
    let words = words();
    let demorgan_left = not_and();
    let demorgan_right = boolean_dual(false);
    let wrong_demorgan = boolean_dual(true);
    let intervals = [open(0), open(2), closed(1, 2)];
    let mut checked = 0;
    let mut wrong_boolean = 0;
    let mut wrong_until_dual = 0;
    let mut wrong_since_dual = 0;
    let mut wrong_weak = 0;
    let mut wrong_mighty = 0;
    let mut wrong_qsl_lower = 0;
    for word in &words {
        for anchor in 0..=word.prefix.len() + 2 * word.cycle.len() {
            let left = evaluate(&demorgan_left, word, anchor);
            assert_eq!(left, evaluate(&demorgan_right, word, anchor));
            wrong_boolean += usize::from(left != evaluate(&wrong_demorgan, word, anchor));
            for interval in intervals {
                let until = temporal(Op::Until, interval, false, true);
                let release = temporal(Op::Release, interval, true, false);
                let since = temporal(Op::Since, interval, false, true);
                let triggered = temporal(Op::Triggered, interval, true, false);
                assert_eq!(
                    evaluate(&until, word, anchor),
                    evaluate(&release, word, anchor)
                );
                assert_eq!(
                    evaluate(&since, word, anchor),
                    evaluate(&triggered, word, anchor)
                );
                for (positive, negative) in
                    [(Op::Future, Op::Globally), (Op::Once, Op::Historically)]
                {
                    assert_eq!(
                        evaluate(&temporal(positive, interval, false, true), word, anchor),
                        evaluate(&temporal(negative, interval, true, false), word, anchor),
                        "unary temporal dual: {positive:?}/{negative:?}"
                    );
                }
                wrong_until_dual += usize::from(
                    evaluate(&until, word, anchor)
                        != evaluate(&temporal(Op::Release, interval, false, false), word, anchor),
                );
                wrong_since_dual += usize::from(
                    evaluate(&since, word, anchor)
                        != evaluate(
                            &temporal(Op::Triggered, interval, false, false),
                            word,
                            anchor,
                        ),
                );

                let weak = lowered(FutureKind::WeakUntil, interval);
                let mighty = lowered(FutureKind::StrongRelease, interval);
                let direct_weak =
                    evaluate(&temporal(Op::Until, interval, false, false), word, anchor)
                        || evaluate(
                            &temporal(Op::Globally, interval, false, false),
                            word,
                            anchor,
                        );
                let direct_mighty =
                    evaluate(&temporal(Op::Release, interval, false, false), word, anchor)
                        && evaluate(&temporal(Op::Future, interval, false, false), word, anchor);
                assert_eq!(evaluate(&weak, word, anchor), direct_weak);
                assert_eq!(evaluate(&mighty, word, anchor), direct_mighty);
                wrong_weak += usize::from(
                    direct_weak
                        != evaluate(&temporal(Op::Until, interval, false, false), word, anchor),
                );
                wrong_mighty += usize::from(
                    direct_mighty
                        != evaluate(&temporal(Op::Release, interval, false, false), word, anchor),
                );
                checked += 1;
            }
            for start in 0..=2_u32 {
                for op in [
                    Op::Future,
                    Op::Globally,
                    Op::Until,
                    Op::Release,
                    Op::Once,
                    Op::Historically,
                    Op::Since,
                    Op::Triggered,
                ] {
                    let tl = evaluate(&temporal(op, open(start), false, false), word, anchor);
                    let qsl = word.denotation(op, usize::try_from(start).unwrap(), None, anchor);
                    assert_eq!(
                        tl, qsl,
                        "QSL [a,*] denotation: {op:?}, start={start}, anchor={anchor}"
                    );
                    if start == 1 {
                        wrong_qsl_lower += usize::from(tl != word.denotation(op, 0, None, anchor));
                    }
                }
            }
            for op in [
                Op::Future,
                Op::Globally,
                Op::Until,
                Op::Release,
                Op::Once,
                Op::Historically,
                Op::Since,
                Op::Triggered,
            ] {
                assert_eq!(
                    evaluate(&temporal(op, closed(1, 2), false, false), word, anchor),
                    word.denotation(op, 1, Some(2), anchor),
                    "closed interval embedded in infinite word: {op:?}, anchor={anchor}"
                );
            }
        }
    }
    assert_eq!(checked, 540 * intervals.len());
    for (name, count) in [
        ("De Morgan and/or swap", wrong_boolean),
        ("U/R missing operand negation", wrong_until_dual),
        ("S/T missing operand negation", wrong_since_dual),
        ("W missing G branch", wrong_weak),
        ("M missing F guard", wrong_mighty),
        ("QSL lower bound shifted to zero", wrong_qsl_lower),
    ] {
        assert!(count > 0, "generated population did not detect {name}");
    }
}
