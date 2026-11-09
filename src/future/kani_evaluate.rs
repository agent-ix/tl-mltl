//! Bounded Kani proofs for the closed/prefix evaluator and horizon analysis.
//!
//! Compiled only under `cargo kani` (`cfg(kani)`); invisible to ordinary and
//! MSRV builds. Every proof states the bound it covers and what it does not
//! claim. Run them with `make kani-check`.
//!
//! Scope note: a direct proof through the recursive `Evaluator::at` exhausted
//! more than 12 GB of CBMC memory even for depth-1 formulas, so the evaluator
//! is proved compositionally: `at` is stubbed and each combinator, the interval
//! arithmetic and the atomic proposition lookup are proved on their own.

use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

use super::{EvaluationError, EvaluationLimits, EvaluationStats, Evaluator, TruthValue};
use crate::horizon::node_lookahead;
use crate::HorizonError;

fn any_interval() -> Interval {
    let start: u32 = kani::any();
    let end: u32 = kani::any();
    kani::assume(start <= end);
    Interval::new(start, end).unwrap()
}

/// A strictly ordered instant over propositions {0, 1}.
fn any_instant() -> Vec<PropositionId> {
    match kani::any::<u8>() % 4 {
        0 => Vec::new(),
        1 => vec![PropositionId(0)],
        2 => vec![PropositionId(1)],
        _ => vec![PropositionId(0), PropositionId(1)],
    }
}

fn limits() -> EvaluationLimits {
    EvaluationLimits {
        max_node_evaluations: 1_000,
        max_temporal_span: 100,
        max_recursion_depth: 8,
    }
}

/// Proves: for every `Interval` (all `start <= end` over the full `u32`
/// range) the evaluator's `temporal_endpoints` never overflows or panics and
/// returns exactly the widened endpoints or the typed span-limit error.
///
/// Does not claim anything about the window loops that consume the endpoints.
#[kani::proof]
fn evaluator_temporal_endpoints_full_range() {
    let interval = any_interval();
    let nodes = [Node::new(NodeKind::True)];
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(0), &nodes).unwrap();
    let trace: Vec<Vec<PropositionId>> = Vec::new();
    let max_span: u64 = kani::any();
    let evaluator = Evaluator {
        formula,
        trace: &trace,
        closed: true,
        limits: EvaluationLimits {
            max_node_evaluations: 1,
            max_temporal_span: max_span,
            max_recursion_depth: 1,
        },
        stats: EvaluationStats::default(),
    };
    match evaluator.temporal_endpoints(interval.start(), interval.end()) {
        Ok((start, end)) => {
            assert_eq!(start, u64::from(interval.start()));
            assert_eq!(end, u64::from(interval.end()));
            assert!(end - start + 1 <= max_span);
            kani::cover!(end > start, "wide window accepted");
        }
        Err(EvaluationError::TemporalSpanExceeded { requested, limit }) => {
            assert_eq!(limit, max_span);
            assert!(requested > limit);
            kani::cover!(true, "span rejection reachable");
        }
        Err(_) => panic!("unexpected error kind"),
    }
}

/// Proves: for every temporal operator (`Future`, `Globally`, `Until`,
/// `Release`), every interval over the full `u32` space, and every pair of
/// operand lookahead values over the full `u64` space, `node_lookahead`
/// returns exactly `interval.end + max(operand values)` when that sum fits in
/// `u64`, and the typed `ArithmeticOverflow` exactly when it does not. It never
/// wraps and never panics. Because `lookahead` computes each node through this
/// function, one application of the proof covers every nesting level.
///
/// Does not claim: the boolean operators (they only take a maximum, covered by
/// the evaluator no-panic proof), operand indices outside the table (typed
/// `InvalidNodeReference`, covered by the unit tests), or the contextual
/// propagation-delay and buffer outputs.
#[kani::proof]
#[kani::unwind(3)]
fn horizon_temporal_lookahead_is_exact_or_typed_overflow() {
    let values: [u64; 2] = kani::any();
    let interval = any_interval();
    let kind = any_temporal(NodeId(0), NodeId(1), interval);
    let operands_max = match kind {
        NodeKind::Future { .. } | NodeKind::Globally { .. } => values[0],
        _ => values[0].max(values[1]),
    };
    let exact = u128::from(interval.end()) + u128::from(operands_max);
    let expected = if exact <= u128::from(u64::MAX) {
        Ok(u64::try_from(exact).unwrap())
    } else {
        Err(HorizonError::ArithmeticOverflow { node: NodeId(2) })
    };
    let actual = node_lookahead(NodeId(2), kind, &values);
    kani::cover!(
        actual.is_ok() && u64::from(interval.end()) > 0,
        "non-trivial sum reachable"
    );
    kani::cover!(actual.is_err(), "overflow refusal reachable");
    assert_eq!(actual, expected);
}

fn any_temporal(left: NodeId, right: NodeId, interval: Interval) -> NodeKind {
    match kani::any::<u8>() % 4 {
        0 => NodeKind::Future {
            interval,
            operand: left,
        },
        1 => NodeKind::Globally {
            interval,
            operand: left,
        },
        2 => NodeKind::Until {
            interval,
            left,
            right,
        },
        _ => NodeKind::Release {
            interval,
            left,
            right,
        },
    }
}

impl<'a, 'b> Evaluator<'a, 'b> {
    fn stub_at(
        &mut self,
        _node: NodeId,
        _time: u64,
        _depth: u32,
    ) -> Result<TruthValue, EvaluationError> {
        if kani::any() {
            Ok(match kani::any::<u8>() % 3 {
                0 => TruthValue::True,
                1 => TruthValue::False,
                _ => TruthValue::Pending,
            })
        } else {
            Err(EvaluationError::WorkLimitExceeded { limit: 0 })
        }
    }
}

/// Proves (compositional, no-panic and typed-error only, with `Evaluator::at`
/// stubbed to return an arbitrary verdict or a typed error): every operator combinator (`not` is inline in
/// `at`; `and`, `or`, `implies`, `equivalent`, `future`, `globally`, `until`
/// and its `Release` negation) returns `Ok` or a typed error and never panics
/// or overflows, for every verdict time over the full `u64` range, every
/// operand pair, and every interval over the full `u32` range. The proof sets
/// the span limit to 3; wider intervals are not excluded, they take the typed
/// span-limit error. Time overflow (`time + offset > u64::MAX`) is reported
/// as `EvaluationError::TimeOverflow`, never wrapped.
///
/// Does not claim: that the recursion in `at` itself terminates or is
/// panic-free for any formula shape (a direct depth-1 proof of `at` exhausted
/// 12 GB of CBMC memory and was abandoned; recursion is bounded at run time by
/// `max_recursion_depth` and `max_node_evaluations`, tested elsewhere), or the
/// correctness of any verdict: the stub returns an arbitrary verdict, so this is
/// a no-panic and typed-error proof only. The cover checks show reachability of
/// the success path and the time-overflow refusal, not correctness.
#[kani::proof]
#[kani::stub(Evaluator::at, Evaluator::stub_at)]
#[kani::unwind(6)]
fn evaluator_combinators_with_stubbed_at_never_panic() {
    let nodes = [Node::new(NodeKind::True)];
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(0), &nodes).unwrap();
    let trace: &[Vec<PropositionId>] = &[];
    let mut evaluator = Evaluator {
        formula,
        trace,
        closed: kani::any(),
        limits: EvaluationLimits {
            max_node_evaluations: 1_000,
            max_temporal_span: 3,
            max_recursion_depth: 8,
        },
        stats: EvaluationStats::default(),
    };
    let time: u64 = kani::any();
    let interval = any_interval();
    let operand = NodeId(0);
    let result = match kani::any::<u8>() % 8 {
        0 => evaluator.and(operand, operand, time, 1),
        1 => evaluator.or(operand, operand, time, 1),
        2 => evaluator.implies(operand, operand, time, 1),
        3 => evaluator.equivalent(operand, operand, time, 1),
        4 => evaluator.future(interval, operand, time, 1),
        5 => evaluator.globally(interval, operand, time, 1),
        6 => evaluator.until(
            time,
            interval.start(),
            interval.end(),
            operand,
            operand,
            false,
            1,
        ),
        _ => evaluator.until(
            time,
            interval.start(),
            interval.end(),
            operand,
            operand,
            true,
            1,
        ),
    };
    match result {
        Ok(_) => {
            kani::cover!(true, "combinator success reachable");
        }
        Err(EvaluationError::TimeOverflow) => {
            kani::cover!(true, "time overflow refusal reachable");
        }
        Err(_) => {
            kani::cover!(true, "other typed refusal reachable");
        }
    }
}

/// Proves (closing prefix, atomic case): for every trace of at most 2 instants
/// over propositions {0, 1}, every proposition, every time, and every extra
/// instant appended to the trace, an open-prefix proposition lookup is decided
/// (`True`/`False`) exactly when `time < len`, is `Pending` exactly when
/// `time >= len`, and a decided answer is unchanged by appending the instant.
///
/// Does not claim: the lifting to compound formulas (that a formula whose
/// lookahead is below the prefix length is decided and stable). A direct
/// proof through `Evaluator::at` exhausted 12 GB of CBMC memory, so the
/// compound statement is evidenced only by the property and oracle tests; the
/// SR-020 closing-prefix harness remains open at the compound level.
#[kani::proof]
#[kani::unwind(6)]
fn closing_prefix_atomic_verdict_is_stable() {
    let nodes = [Node::new(NodeKind::True)];
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(0), &nodes).unwrap();
    let storage = [any_instant(), any_instant(), any_instant()];
    let len: usize = kani::any();
    kani::assume(len <= 2);
    let time: u64 = kani::any();
    let proposition = PropositionId(kani::any::<u32>() % 2);
    let at = |trace: &[Vec<PropositionId>]| {
        Evaluator {
            formula,
            trace,
            closed: false,
            limits: limits(),
            stats: EvaluationStats::default(),
        }
        .proposition(time, proposition)
    };
    let before = at(&storage[..len]).unwrap();
    let after = at(&storage[..len + 1]).unwrap();
    if time < len as u64 {
        assert_ne!(before, TruthValue::Pending);
        kani::cover!(before == TruthValue::True, "decided True reachable");
        kani::cover!(before == TruthValue::False, "decided False reachable");
    } else {
        assert_eq!(before, TruthValue::Pending);
    }
    if before != TruthValue::Pending {
        assert_eq!(before, after);
    }
    kani::cover!(before == TruthValue::Pending, "pending reachable");
}
