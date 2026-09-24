//! Structure-aware bounded finite words compared with an independent oracle.

use std::collections::BTreeMap;

use arbitrary::Unstructured;
use tl_mltl::{
    evaluate_closed_at, evaluate_past, ClockBinding, EvaluationLimits, PastEvaluationLimits,
    PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TruthValue,
};
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as OracleFormula,
    Interval as OracleInterval, Limits,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const MAX_DEPTH: u8 = 3;
const MAX_TRACE: usize = 8;
const PROPOSITIONS: u32 = 3;

#[derive(Debug, Eq, PartialEq)]
pub struct Difference {
    pub position: usize,
    pub expected: bool,
    pub observed: bool,
    pub root: NodeKind,
}

struct Seed<'a>(Unstructured<'a>);

impl Seed<'_> {
    fn byte(&mut self) -> u8 {
        self.0.arbitrary::<u8>().unwrap_or(0)
    }

    fn interval(&mut self) -> (Interval, OracleInterval) {
        let start = u32::from(self.byte() % 4);
        let end = start + u32::from(self.byte() % 4);
        (
            Interval::new(start, end).expect("generated ordered interval"),
            OracleInterval::Closed {
                start: usize::try_from(start).expect("small offset"),
                end: usize::try_from(end).expect("small offset"),
            },
        )
    }
}

fn push(nodes: &mut Vec<Node>, kind: NodeKind) -> NodeId {
    let id = NodeId(u32::try_from(nodes.len()).expect("depth-three graph is bounded"));
    nodes.push(Node::new(kind));
    id
}

fn build(
    seed: &mut Seed<'_>,
    nodes: &mut Vec<Node>,
    profile: SemanticProfile,
    depth: u8,
) -> (NodeId, OracleFormula) {
    use OracleFormula as O;
    if depth == 0 {
        return match seed.byte() % (PROPOSITIONS as u8 + 2) {
            0 => (push(nodes, NodeKind::False), O::False),
            1 => (push(nodes, NodeKind::True), O::True),
            atom => {
                let proposition = PropositionId(u32::from(atom - 2));
                (
                    push(nodes, NodeKind::Proposition { proposition }),
                    O::Atom(proposition),
                )
            }
        };
    }

    let choices = if profile == SemanticProfile::ClosedTraceV1 {
        9
    } else {
        10
    };
    let op = seed.byte() % choices;
    let (left, oracle_left) = build(seed, nodes, profile, depth - 1);
    if matches!(op, 0 | 5 | 6) || (profile == SemanticProfile::OriginCompleteHistoryV1 && op == 9) {
        let (kind, oracle) = match (profile, op) {
            (_, 0) => (
                NodeKind::Not { operand: left },
                O::Not(Box::new(oracle_left)),
            ),
            (SemanticProfile::ClosedTraceV1, 5) => {
                let (range, oracle_range) = seed.interval();
                (
                    NodeKind::Future {
                        interval: range,
                        operand: left,
                    },
                    O::Future(oracle_range, Box::new(oracle_left)),
                )
            }
            (SemanticProfile::ClosedTraceV1, 6) => {
                let (range, oracle_range) = seed.interval();
                (
                    NodeKind::Globally {
                        interval: range,
                        operand: left,
                    },
                    O::Globally(oracle_range, Box::new(oracle_left)),
                )
            }
            (SemanticProfile::OriginCompleteHistoryV1, 5) => {
                let (range, oracle_range) = seed.interval();
                (
                    NodeKind::Once {
                        interval: range,
                        operand: left,
                    },
                    O::Once(oracle_range, Box::new(oracle_left)),
                )
            }
            (SemanticProfile::OriginCompleteHistoryV1, 6) => {
                let (range, oracle_range) = seed.interval();
                (
                    NodeKind::Historically {
                        interval: range,
                        operand: left,
                    },
                    O::Historically(oracle_range, Box::new(oracle_left)),
                )
            }
            (SemanticProfile::OriginCompleteHistoryV1, 9) => (
                NodeKind::StrongPrevious { operand: left },
                O::StrongPrevious(Box::new(oracle_left)),
            ),
            _ => unreachable!("bounded operator choice"),
        };
        return (push(nodes, kind), oracle);
    }

    let (right, oracle_right) = build(seed, nodes, profile, depth - 1);
    let (kind, oracle) = match (profile, op) {
        (_, 1) => (
            NodeKind::And { left, right },
            O::And(Box::new(oracle_left), Box::new(oracle_right)),
        ),
        (_, 2) => (
            NodeKind::Or { left, right },
            O::Or(Box::new(oracle_left), Box::new(oracle_right)),
        ),
        (_, 3) => (
            NodeKind::Implies { left, right },
            O::Implies(Box::new(oracle_left), Box::new(oracle_right)),
        ),
        (_, 4) => (
            NodeKind::Equivalent { left, right },
            O::Equivalent(Box::new(oracle_left), Box::new(oracle_right)),
        ),
        (SemanticProfile::ClosedTraceV1, 7) => {
            let (range, oracle_range) = seed.interval();
            (
                NodeKind::Until {
                    interval: range,
                    left,
                    right,
                },
                O::Until(oracle_range, Box::new(oracle_left), Box::new(oracle_right)),
            )
        }
        (SemanticProfile::ClosedTraceV1, 8) => {
            let (range, oracle_range) = seed.interval();
            (
                NodeKind::Release {
                    interval: range,
                    left,
                    right,
                },
                O::Release(oracle_range, Box::new(oracle_left), Box::new(oracle_right)),
            )
        }
        (SemanticProfile::OriginCompleteHistoryV1, 7) => {
            let (range, oracle_range) = seed.interval();
            (
                NodeKind::Since {
                    interval: range,
                    left,
                    right,
                },
                O::Since(oracle_range, Box::new(oracle_left), Box::new(oracle_right)),
            )
        }
        (SemanticProfile::OriginCompleteHistoryV1, 8) => {
            let (range, oracle_range) = seed.interval();
            (
                NodeKind::Triggered {
                    interval: range,
                    left,
                    right,
                },
                O::Triggered(oracle_range, Box::new(oracle_left), Box::new(oracle_right)),
            )
        }
        _ => unreachable!("bounded operator choice"),
    };
    (push(nodes, kind), oracle)
}

#[cfg(test)]
pub fn root_kind(data: &[u8]) -> NodeKind {
    let mut seed = Seed(Unstructured::new(data));
    let profile = if seed.byte() & 1 == 0 {
        SemanticProfile::ClosedTraceV1
    } else {
        SemanticProfile::OriginCompleteHistoryV1
    };
    let depth = seed.byte() % (MAX_DEPTH + 1);
    let mut nodes = Vec::new();
    let (root, _) = build(&mut seed, &mut nodes, profile, depth);
    nodes[usize::try_from(root.0).expect("small root")].kind
}

/// One input always yields a well-formed formula and a complete nonempty word.
pub fn check(data: &[u8], inject_until_fault: bool) -> Result<(), Difference> {
    let mut seed = Seed(Unstructured::new(data));
    let profile = if seed.byte() & 1 == 0 {
        SemanticProfile::ClosedTraceV1
    } else {
        SemanticProfile::OriginCompleteHistoryV1
    };
    let depth = seed.byte() % (MAX_DEPTH + 1);
    let mut nodes = Vec::new();
    let (root, oracle) = build(&mut seed, &mut nodes, profile, depth);
    let root_kind = nodes[usize::try_from(root.0).expect("small root")].kind;
    let formula = Formula::new(profile, root, &nodes).expect("generated valid formula");
    let length = 1 + usize::from(seed.byte()) % MAX_TRACE;
    let rows: Vec<Vec<PropositionId>> = (0..length)
        .map(|_| {
            let mask = seed.byte();
            (0..PROPOSITIONS)
                .filter(|id| mask & (1_u8 << id) != 0)
                .map(PropositionId)
                .collect()
        })
        .collect();
    let oracle_rows: Vec<BTreeMap<PropositionId, bool>> = rows
        .iter()
        .map(|row| {
            (0..PROPOSITIONS)
                .map(|id| (PropositionId(id), row.contains(&PropositionId(id))))
                .collect()
        })
        .collect();
    let history = (profile == SemanticProfile::OriginCompleteHistoryV1).then(|| {
        PositionHistoryDocument::new(
            "finite-oracle-fuzz",
            1,
            0,
            u64::try_from(length - 1).expect("bounded length"),
            Some(ClockBinding::EventPosition),
            rows.iter()
                .enumerate()
                .map(|(position, row)| {
                    PositionObservation::new(
                        u64::try_from(position).expect("bounded position"),
                        row.clone(),
                        None,
                    )
                })
                .collect(),
        )
        .expect("generated complete history")
    });
    for position in 0..length {
        let expected = if profile == SemanticProfile::ClosedTraceV1 {
            evaluate_closed_trace_v1(&oracle, &oracle_rows, position, Limits::default())
        } else {
            evaluate_origin_complete(&oracle, &oracle_rows, position, Limits::default())
        }
        .expect("bounded generated oracle evaluation");
        let observed = if let Some(history) = &history {
            evaluate_past(
                formula,
                "finite-oracle-fuzz",
                history,
                u64::try_from(position).expect("bounded position"),
                "fuzz-map",
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits::default(),
            )
            .expect("bounded generated past evaluation")
            .verdict
        } else {
            match evaluate_closed_at(
                formula,
                "finite-oracle-fuzz",
                &rows,
                "word",
                u64::try_from(position).expect("bounded position"),
                EvaluationLimits::default(),
            )
            .expect("bounded generated closed evaluation")
            .verdict
            {
                TruthValue::True => true,
                TruthValue::False => false,
                TruthValue::Pending => panic!("complete generated word was pending"),
            }
        };
        let observed =
            if inject_until_fault && position == 0 && matches!(root_kind, NodeKind::Until { .. }) {
                !observed
            } else {
                observed
            };
        if observed != expected {
            return Err(Difference {
                position,
                expected,
                observed,
                root: root_kind,
            });
        }
    }
    Ok(())
}
