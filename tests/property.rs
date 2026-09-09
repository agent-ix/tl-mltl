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
