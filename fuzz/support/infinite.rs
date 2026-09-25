use tl_mltl::infinite::{
    evaluate_lasso, Disposition, EvaluationLimit, EvidenceClosure, InfiniteResult, LassoRequest,
};
use tl_syntax::{
    FairnessPremisesDocument, InfiniteClock, InfiniteFormulaDocument, InfiniteNode,
    InfiniteNodeKind as K, LassoTraceDocument, NodeId, PartialValuation, PartialValue,
    PropositionId, SemanticProfile, TemporalInterval, TraceObservation, UnboundedInterval,
    ValuationEntry,
};

const P: PropositionId = PropositionId(0);
const Q: PropositionId = PropositionId(1);
const MAP: &str = "tl223-map";

pub struct Seed<'a> {
    bytes: &'a [u8],
    next: usize,
}

impl<'a> Seed<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, next: 0 }
    }

    pub fn byte(&mut self) -> u8 {
        let result = self.bytes.get(self.next).copied().unwrap_or(0);
        self.next += 1;
        result
    }
}

fn node(kind: K) -> InfiniteNode {
    InfiniteNode::new(kind)
}

fn open(start: u8) -> TemporalInterval {
    TemporalInterval::Unbounded(UnboundedInterval::new(u32::from(start % 3)))
}

pub fn formula(seed: &mut Seed<'_>) -> InfiniteFormulaDocument {
    let selector = seed.byte() % 15;
    let interval = open(seed.byte());
    let mut nodes = vec![node(K::Proposition { proposition: P })];
    let kind = match selector {
        0 => K::Future {
            interval,
            operand: NodeId(0),
        },
        1 => K::Globally {
            interval,
            operand: NodeId(0),
        },
        2 => K::Once {
            interval,
            operand: NodeId(0),
        },
        3 => K::Historically {
            interval,
            operand: NodeId(0),
        },
        4 => K::StrongPrevious { operand: NodeId(0) },
        5 => K::Not { operand: NodeId(0) },
        6 => {
            nodes.push(node(K::Historically {
                interval: open(0),
                operand: NodeId(0),
            }));
            K::Future {
                interval,
                operand: NodeId(1),
            }
        }
        7 => {
            nodes.push(node(K::Once {
                interval: open(0),
                operand: NodeId(0),
            }));
            K::Globally {
                interval,
                operand: NodeId(1),
            }
        }
        8..=13 => {
            nodes.push(node(K::Proposition { proposition: Q }));
            match selector {
                8 => K::Until {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
                9 => K::Release {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
                10 => K::Since {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
                11 => K::Triggered {
                    interval,
                    left: NodeId(0),
                    right: NodeId(1),
                },
                12 => K::And {
                    left: NodeId(0),
                    right: NodeId(1),
                },
                _ => K::Or {
                    left: NodeId(0),
                    right: NodeId(1),
                },
            }
        }
        _ => K::Equivalent {
            left: NodeId(0),
            right: NodeId(0),
        },
    };
    nodes.push(node(kind));
    let root = NodeId(u32::try_from(nodes.len() - 1).unwrap());
    InfiniteFormulaDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        root,
        nodes,
    )
    .unwrap()
}

pub fn values(seed: &mut Seed<'_>, require_unknown: bool) -> (usize, Vec<[PartialValue; 2]>) {
    let prefix_len = usize::from(seed.byte() % 3);
    let loop_len = usize::from(seed.byte() % 3) + 1;
    let mut unknowns = 0;
    let mut rows = (0..prefix_len + loop_len)
        .map(|_| {
            [P, Q].map(|_| match seed.byte() % 4 {
                0 => PartialValue::False,
                1 => PartialValue::True,
                2 if unknowns < 2 => {
                    unknowns += 1;
                    PartialValue::Missing
                }
                3 if unknowns < 2 => {
                    unknowns += 1;
                    PartialValue::Conflicting
                }
                _ => PartialValue::False,
            })
        })
        .collect::<Vec<_>>();
    if require_unknown && unknowns == 0 {
        rows[0][0] = PartialValue::Missing;
    }
    (prefix_len, rows)
}

pub fn trace(prefix_len: usize, rows: &[[PartialValue; 2]]) -> LassoTraceDocument {
    let observation = |position: usize, values: [PartialValue; 2]| TraceObservation {
        position: u32::try_from(position).unwrap(),
        valuation: PartialValuation::new(
            MAP.to_owned(),
            &[P, Q],
            [P, Q]
                .into_iter()
                .zip(values)
                .map(|(proposition, value)| ValuationEntry { proposition, value })
                .collect(),
        )
        .unwrap(),
    };
    LassoTraceDocument::new(
        SemanticProfile::InfiniteTraceV1,
        InfiniteClock::EventPosition,
        MAP.to_owned(),
        vec![P, Q],
        rows[..prefix_len]
            .iter()
            .copied()
            .enumerate()
            .map(|(at, value)| observation(at, value))
            .collect(),
        rows[prefix_len..]
            .iter()
            .copied()
            .enumerate()
            .map(|(at, value)| observation(prefix_len + at, value))
            .collect(),
    )
    .unwrap()
}

pub fn evaluate(
    formula: &InfiniteFormulaDocument,
    trace: &LassoTraceDocument,
    fairness: Option<&FairnessPremisesDocument>,
    position: usize,
) -> InfiniteResult {
    let graph_id = formula.content_identity().unwrap();
    let trace_id = trace.content_identity().unwrap();
    let result = evaluate_lasso(&LassoRequest {
        formula,
        trace,
        fairness,
        evidence_closure: EvidenceClosure::Closed,
        graph_id: &graph_id,
        trace_id: &trace_id,
        selected_position: u64::try_from(position).unwrap(),
        limit: EvaluationLimit::default(),
    })
    .unwrap();
    assert_ne!(result.disposition, Disposition::Failed);
    result
}
