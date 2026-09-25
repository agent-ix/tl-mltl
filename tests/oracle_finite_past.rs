use std::collections::BTreeMap;

use tl_mltl::{
    evaluate_closed_at, evaluate_past, ClockBinding, EvaluationLimits, PastEvaluationLimits,
    PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TruthValue,
};
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as OracleFormula,
    Interval as OracleInterval, Limits,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn p() -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    })
}

fn q() -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(1),
    })
}

fn atom(id: u32) -> OracleFormula {
    OracleFormula::Atom(PropositionId(id))
}

fn range(start: u32, end: u32) -> Interval {
    Interval::new(start, end).unwrap()
}

fn oracle_range(start: usize, end: usize) -> OracleInterval {
    OracleInterval::Closed { start, end }
}

fn cases() -> Vec<(&'static str, SemanticProfile, NodeKind, OracleFormula)> {
    use OracleFormula as O;
    vec![
        (
            "not",
            SemanticProfile::ClosedTraceV1,
            NodeKind::Not { operand: NodeId(0) },
            O::Not(Box::new(atom(0))),
        ),
        (
            "and",
            SemanticProfile::ClosedTraceV1,
            NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            },
            O::And(Box::new(atom(0)), Box::new(atom(1))),
        ),
        (
            "future",
            SemanticProfile::ClosedTraceV1,
            NodeKind::Future {
                interval: range(0, 2),
                operand: NodeId(0),
            },
            O::Future(oracle_range(0, 2), Box::new(atom(0))),
        ),
        (
            "globally",
            SemanticProfile::ClosedTraceV1,
            NodeKind::Globally {
                interval: range(1, 2),
                operand: NodeId(0),
            },
            O::Globally(oracle_range(1, 2), Box::new(atom(0))),
        ),
        (
            "until",
            SemanticProfile::ClosedTraceV1,
            NodeKind::Until {
                interval: range(1, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
            O::Until(oracle_range(1, 2), Box::new(atom(0)), Box::new(atom(1))),
        ),
        (
            "release",
            SemanticProfile::ClosedTraceV1,
            NodeKind::Release {
                interval: range(0, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
            O::Release(oracle_range(0, 2), Box::new(atom(0)), Box::new(atom(1))),
        ),
        (
            "once",
            SemanticProfile::OriginCompleteHistoryV1,
            NodeKind::Once {
                interval: range(0, 2),
                operand: NodeId(0),
            },
            O::Once(oracle_range(0, 2), Box::new(atom(0))),
        ),
        (
            "historically",
            SemanticProfile::OriginCompleteHistoryV1,
            NodeKind::Historically {
                interval: range(1, 2),
                operand: NodeId(0),
            },
            O::Historically(oracle_range(1, 2), Box::new(atom(0))),
        ),
        (
            "since",
            SemanticProfile::OriginCompleteHistoryV1,
            NodeKind::Since {
                interval: range(0, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
            O::Since(oracle_range(0, 2), Box::new(atom(0)), Box::new(atom(1))),
        ),
        (
            "triggered",
            SemanticProfile::OriginCompleteHistoryV1,
            NodeKind::Triggered {
                interval: range(1, 2),
                left: NodeId(0),
                right: NodeId(1),
            },
            O::Triggered(oracle_range(1, 2), Box::new(atom(0)), Box::new(atom(1))),
        ),
        (
            "previous",
            SemanticProfile::OriginCompleteHistoryV1,
            NodeKind::StrongPrevious { operand: NodeId(0) },
            O::StrongPrevious(Box::new(atom(0))),
        ),
    ]
}

// Trace: TC-175; FR-043-AC-1, NFR-007-AC-1
#[test]
fn independent_oracle_matches_small_finite_and_origin_complete_words() {
    let formulas = cases();
    assert_eq!(formulas.len(), 11);
    let mut positions = 0;
    let mut comparisons = 0;
    for length in 1..=4_usize {
        let word_count = 4_usize.pow(u32::try_from(length).unwrap());
        for bits in 0..word_count {
            let rows: Vec<Vec<PropositionId>> = (0..length)
                .map(|position| {
                    (0..2_u32)
                        .filter(|id| bits & (1 << (position * 2 + *id as usize)) != 0)
                        .map(PropositionId)
                        .collect()
                })
                .collect();
            let oracle_rows: Vec<BTreeMap<PropositionId, bool>> = rows
                .iter()
                .map(|row| {
                    (0..2_u32)
                        .map(|id| (PropositionId(id), row.contains(&PropositionId(id))))
                        .collect()
                })
                .collect();
            let history = PositionHistoryDocument::new(
                "oracle-finite",
                1,
                0,
                u64::try_from(length - 1).unwrap(),
                Some(ClockBinding::EventPosition),
                rows.iter()
                    .enumerate()
                    .map(|(position, row)| {
                        PositionObservation::new(
                            u64::try_from(position).unwrap(),
                            row.clone(),
                            None,
                        )
                    })
                    .collect(),
            )
            .unwrap();
            for position in 0..length {
                positions += 1;
                for (name, profile, kind, oracle) in &formulas {
                    let nodes = [p(), q(), Node::new(*kind)];
                    let syntax = Formula::new(*profile, NodeId(2), &nodes).unwrap();
                    let expected = if *profile == SemanticProfile::ClosedTraceV1 {
                        evaluate_closed_trace_v1(oracle, &oracle_rows, position, Limits::default())
                    } else {
                        evaluate_origin_complete(oracle, &oracle_rows, position, Limits::default())
                    }
                    .unwrap();
                    let actual = if *profile == SemanticProfile::ClosedTraceV1 {
                        evaluate_closed_at(
                            syntax,
                            *name,
                            &rows,
                            "word",
                            u64::try_from(position).unwrap(),
                            EvaluationLimits::default(),
                        )
                        .unwrap()
                        .verdict
                            == TruthValue::True
                    } else {
                        evaluate_past(
                            syntax,
                            *name,
                            &history,
                            u64::try_from(position).unwrap(),
                            "map",
                            1,
                            PastEvaluationRelationInput::Original,
                            PastEvaluationLimits::default(),
                        )
                        .unwrap()
                        .verdict
                    };
                    assert_eq!(
                        actual, expected,
                        "{name}, length={length}, bits={bits}, position={position}"
                    );
                    comparisons += 1;
                }
            }
        }
    }
    assert_eq!(positions, 4 + 2 * 16 + 3 * 64 + 4 * 256);
    assert_eq!(comparisons, 11 * positions);
}
