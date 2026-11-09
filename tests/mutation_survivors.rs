//! Tests that kill the evaluator-core mutants the first cargo-mutants run left
//! alive (TL-231). Each test names the mutant it targets.

use tl_mltl::{
    evaluate_closed, evaluate_closed_at_with_stats, ContextualEvaluationError,
    ContextualHorizonError, EvaluationError, EvaluationLimits, HorizonError,
};
use tl_syntax::{Formula, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn p0_and_p0() -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::And {
            left: NodeId(0),
            right: NodeId(0),
        }),
    ]
}

fn limits(max_node_evaluations: u64) -> EvaluationLimits {
    EvaluationLimits {
        max_node_evaluations,
        ..EvaluationLimits::default()
    }
}

/// Kills `consume`: replace `>` with `>=` / `==` in the work-limit comparison.
#[test]
fn work_limit_is_inclusive_and_stops_at_the_first_excess_evaluation() {
    let nodes = p0_and_p0();
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(1), &nodes).unwrap();
    let trace = vec![vec![PropositionId(0)]];

    // `p0 & p0` needs exactly three node evaluations: And, left, right.
    let (exact, stats) = evaluate_closed_at_with_stats(formula, "f", &trace, "t", 0, limits(3));
    assert!(
        exact.is_ok(),
        "a budget equal to the work needed must succeed"
    );
    assert_eq!(stats.node_evaluations, 3);

    let (short, stats) = evaluate_closed_at_with_stats(formula, "f", &trace, "t", 0, limits(2));
    assert_eq!(short, Err(EvaluationError::WorkLimitExceeded { limit: 2 }));
    assert_eq!(
        stats.node_evaluations, 3,
        "evaluation stops at the first evaluation past the budget, not at the budget"
    );
}

/// Kills `validate_trace -> Ok(())`.
#[test]
fn unordered_or_duplicate_propositions_in_an_instant_are_refused() {
    let nodes = vec![Node::new(NodeKind::True)];
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(0), &nodes).unwrap();
    for (previous, current) in [(1, 0), (0, 0)] {
        let trace = vec![vec![PropositionId(previous), PropositionId(current)]];
        assert_eq!(
            evaluate_closed(formula, "f", &trace, "t", EvaluationLimits::default()),
            Err(EvaluationError::TraceNotStrictlyOrdered {
                instant: 0,
                previous: PropositionId(previous),
                current: PropositionId(current),
            })
        );
    }
}

/// Kills `<impl Display for EvaluationError>::fmt -> Ok(Default::default())` and
/// the matching `HorizonError`, `ContextualEvaluationError` and
/// `ContextualHorizonError` mutants.
#[test]
fn error_displays_name_the_failure() {
    let node = NodeId(7);
    let horizon = HorizonError::ArithmeticOverflow { node };
    assert_eq!(horizon.to_string(), "horizon arithmetic overflow at node 7");
    assert_eq!(
        HorizonError::UnsupportedProfile { actual: "x" }.to_string(),
        "future-horizon analysis does not support semantic profile x"
    );
    assert_eq!(
        EvaluationError::TimeOverflow.to_string(),
        "discrete-time arithmetic overflow"
    );
    assert_eq!(
        EvaluationError::WorkLimitExceeded { limit: 5 }.to_string(),
        "evaluation exceeded work limit 5"
    );
    assert_eq!(
        EvaluationError::TemporalSpanExceeded {
            requested: 9,
            limit: 4
        }
        .to_string(),
        "temporal span 9 exceeds configured limit 4"
    );
    assert_eq!(
        EvaluationError::TraceNotStrictlyOrdered {
            instant: 2,
            previous: PropositionId(5),
            current: PropositionId(3),
        }
        .to_string(),
        "trace instant 2 proposition 3 does not follow 5"
    );
    assert_eq!(
        ContextualEvaluationError::Identity("boom".into()).to_string(),
        "contextual identity failed: boom"
    );
    assert_eq!(
        ContextualEvaluationError::Evaluation(EvaluationError::TimeOverflow).to_string(),
        EvaluationError::TimeOverflow.to_string()
    );
    assert_eq!(
        ContextualHorizonError::Identity("boom".into()).to_string(),
        "contextual identity failed: boom"
    );
    assert_eq!(
        ContextualHorizonError::Horizon(horizon).to_string(),
        horizon.to_string()
    );
}
