use proptest::prelude::*;
use tl_mltl::{evaluate_closed, evaluate_prefix, EvaluationLimits};
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

fn trace(bits: Vec<bool>) -> Vec<Vec<PropositionId>> {
    bits.into_iter()
        .map(|present| {
            if present {
                vec![PropositionId(0)]
            } else {
                vec![]
            }
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: None,
        ..ProptestConfig::default()
    })]

    // Trace: TC-032, FR-003-AC-2, NFR-001-AC-1
    #[test]
    fn closed_prefix_matches_closed_evaluation_over_bounded_generated_inputs(
        kind in 0_u8..4,
        start in 0_u32..=2,
        end in 0_u32..=2,
        bits in prop::collection::vec(any::<bool>(), 0..=5),
    ) {
        let interval = Interval::new(start.min(end), start.max(end)).unwrap();
        let nodes = bounded_formula(kind, interval);
        let trace = trace(bits);
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
        prop_assert_eq!(closed.verdict, prefix.verdict);
        prop_assert_eq!(closed.horizon, prefix.horizon);
    }
}
