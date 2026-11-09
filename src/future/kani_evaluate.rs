//! Bounded Kani proofs for the closed/prefix evaluator and horizon analysis.
//!
//! Compiled only under `cargo kani` (`cfg(kani)`); invisible to ordinary and
//! MSRV builds. Every proof states the bound it covers and what it does not
//! claim. Run them with `make kani-check`.
//!
//! Scope note: a direct proof through the recursive `Evaluator::at` exhausts
//! more than 8 GB of CBMC memory because its statically inlined body is large,
//! so the evaluator is proved compositionally: one harness stubs `at` and proves
//! every operator body, another stubs every operator and proves `at` itself,
//! and the closing-prefix lemmas run the operators over a verdict-table oracle.
//! The argument that the halves compose is stated in the harness docs.

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
        havoc_stats(self);
        any_outcome()
    }
}

/// Proves (compositional, no-panic and typed-error only, with `Evaluator::at`
/// stubbed to return an arbitrary verdict or a typed error): every operator
/// combinator (`not` is inline in `at`; `and`, `or`, `implies`, `equivalent`, `future`, `globally`, `until`
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

/// Arbitrary verdict-or-typed-error, the over-approximation of any operator.
fn any_outcome() -> Result<TruthValue, EvaluationError> {
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

/// Real operators advance the work counters through `at`; the stubs replace
/// them with arbitrary counters, a superset of every real update.
fn havoc_stats(evaluator: &mut Evaluator<'_, '_>) {
    evaluator.stats = EvaluationStats {
        node_evaluations: kani::any(),
        max_recursion_depth: kani::any(),
    };
}

impl<'a, 'b> Evaluator<'a, 'b> {
    fn stub_binary(
        &mut self,
        _left: NodeId,
        _right: NodeId,
        _time: u64,
        _child_depth: u32,
    ) -> Result<TruthValue, EvaluationError> {
        havoc_stats(self);
        any_outcome()
    }

    fn stub_windowed(
        &mut self,
        _interval: Interval,
        _operand: NodeId,
        _time: u64,
        _child_depth: u32,
    ) -> Result<TruthValue, EvaluationError> {
        havoc_stats(self);
        any_outcome()
    }

    #[allow(clippy::too_many_arguments)]
    fn stub_until(
        &mut self,
        _time: u64,
        _start: u32,
        _end: u32,
        _left: NodeId,
        _right: NodeId,
        _negate_operands: bool,
        _child_depth: u32,
    ) -> Result<TruthValue, EvaluationError> {
        havoc_stats(self);
        any_outcome()
    }
}

/// Proves (compositional half 2 of 2, no-panic and typed-error only): the
/// dispatcher `Evaluator::at` and the proposition lookup never panic, overflow
/// or index out of bounds when every operator method it calls (`and`, `or`,
/// `implies`, `equivalent`, `future`, `globally`, `until`) is replaced by an
/// arbitrary verdict-or-typed-error that also leaves the work counters
/// arbitrary. The formula is a topologically ordered table of 3 nodes: a
/// symbolic leaf (`False`, `True`, `Proposition` 0 or 1), a symbolic node over
/// it and a symbolic root over both, each drawn from every future-profile
/// kind with symbolic intervals, so the real `Not` recursion runs to depth 3.
/// The trace has 0..=4 instants over propositions {0, 1}; time, depth and the
/// work counters are symbolic over their full ranges, and `closed` is symbolic.
///
/// Together with `evaluator_combinators_with_stubbed_at_never_panic` (half 1,
/// every operator body with `at` stubbed) this is an assume-guarantee argument:
/// every operator is panic-free whenever `at` is, and `at` is panic-free
/// whenever every operator is, so by induction on the recursion depth (which
/// `max_recursion_depth` and the checked depth increment bound) the real
/// recursive `at` is panic-free for every well-formed formula the profile
/// admits. The induction is the argument, not a Kani result.
///
/// Does not claim: any verdict is correct (the stubs return arbitrary
/// verdicts), that the two halves model the same `at` beyond the shared source
/// (the stubs are hand-written over-approximations, not machine-generated
/// contracts), that unbounded time loops terminate quickly (the windows are in
/// half 1 with a span limit), or anything about traces longer than 4 instants
/// or propositions beyond {0, 1}.
#[kani::proof]
#[kani::stub(Evaluator::and, Evaluator::stub_binary)]
#[kani::stub(Evaluator::or, Evaluator::stub_binary)]
#[kani::stub(Evaluator::implies, Evaluator::stub_binary)]
#[kani::stub(Evaluator::equivalent, Evaluator::stub_binary)]
#[kani::stub(Evaluator::future, Evaluator::stub_windowed)]
#[kani::stub(Evaluator::globally, Evaluator::stub_windowed)]
#[kani::stub(Evaluator::until, Evaluator::stub_until)]
#[kani::unwind(6)]
fn evaluator_dispatch_with_stubbed_operators_never_panics() {
    let leaf = match kani::any::<u8>() % 4 {
        0 => NodeKind::False,
        1 => NodeKind::True,
        2 => NodeKind::Proposition {
            proposition: PropositionId(0),
        },
        _ => NodeKind::Proposition {
            proposition: PropositionId(1),
        },
    };
    let nodes = [
        Node::new(leaf),
        Node::new(any_future_kind(NodeId(0), NodeId(0))),
        Node::new(any_future_kind(NodeId(0), NodeId(1))),
    ];
    let root = NodeId(kani::any::<u32>() % 3);
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, root, &nodes).unwrap();
    let storage = [any_instant(), any_instant(), any_instant(), any_instant()];
    let len: usize = kani::any();
    kani::assume(len <= 4);
    let mut evaluator = Evaluator {
        formula,
        trace: &storage[..len],
        closed: kani::any(),
        limits: EvaluationLimits {
            max_node_evaluations: kani::any(),
            max_temporal_span: kani::any(),
            max_recursion_depth: kani::any(),
        },
        stats: EvaluationStats {
            node_evaluations: kani::any(),
            max_recursion_depth: kani::any(),
        },
    };
    let time: u64 = kani::any();
    let depth: u32 = kani::any();
    let outcome = evaluator.at(root, time, depth);
    match outcome {
        Ok(TruthValue::Pending) => {
            kani::cover!(true, "pending verdict reachable");
        }
        Ok(_) => {
            kani::cover!(true, "decided verdict reachable");
        }
        Err(EvaluationError::RecursionDepthExceeded { .. }) => {
            kani::cover!(true, "depth refusal reachable");
        }
        Err(_) => {
            kani::cover!(true, "other typed refusal reachable");
        }
    }
    kani::cover!(
        matches!(nodes[root.0 as usize].kind, NodeKind::Not { .. }) && outcome.is_ok(),
        "not recursion succeeded"
    );
    kani::cover!(
        matches!(nodes[root.0 as usize].kind, NodeKind::Proposition { .. }) && outcome.is_ok(),
        "proposition lookup succeeded"
    );
}

/// Every future-profile node kind over operands `a` and `b`, symbolic interval.
fn any_future_kind(a: NodeId, b: NodeId) -> NodeKind {
    let interval = any_interval();
    match kani::any::<u8>() % 11 {
        0 => NodeKind::Not { operand: a },
        1 => NodeKind::And { left: a, right: b },
        2 => NodeKind::Or { left: a, right: b },
        3 => NodeKind::Implies { left: a, right: b },
        4 => NodeKind::Equivalent { left: a, right: b },
        5 => NodeKind::Future {
            interval,
            operand: b,
        },
        6 => NodeKind::Globally {
            interval,
            operand: b,
        },
        7 => NodeKind::Until {
            interval,
            left: a,
            right: b,
        },
        8 => NodeKind::Release {
            interval,
            left: a,
            right: b,
        },
        9 => NodeKind::Not { operand: b },
        _ => NodeKind::False,
    }
}

/// Verdict oracle shared by the two evaluations of the closing-prefix lemma:
/// `ORACLE[side][node][offset]` with side 0 the prefix and side 1 the prefix
/// extended by one instant, node 0 or 1, offset the verdict time minus
/// `ORACLE_BASE`. Encoded 0 `True`, 1 `False`, 2 `Pending`.
static mut ORACLE: [[[u8; 4]; 2]; 2] = [[[2; 4]; 2]; 2];
static mut ORACLE_BASE: u64 = 0;
static mut PREFIX_LEN: usize = 0;

fn decode(code: u8) -> TruthValue {
    match code {
        0 => TruthValue::True,
        1 => TruthValue::False,
        _ => TruthValue::Pending,
    }
}

impl<'a, 'b> Evaluator<'a, 'b> {
    /// `at` replaced by a deterministic table lookup, so repeated calls for the
    /// same node and time agree within and across the two evaluations.
    fn oracle_at(
        &mut self,
        node: NodeId,
        time: u64,
        _depth: u32,
    ) -> Result<TruthValue, EvaluationError> {
        // SAFETY: single-threaded proof harness; the statics are written only
        // by the harness before the call.
        unsafe {
            let offset = time.wrapping_sub(*core::ptr::addr_of!(ORACLE_BASE));
            assert!(
                offset < 4 && node.0 < 2,
                "oracle key out of the modelled range"
            );
            let side = usize::from(self.trace.len() > *core::ptr::addr_of!(PREFIX_LEN));
            let oracle = &*core::ptr::addr_of!(ORACLE);
            Ok(decode(oracle[side][node.0 as usize][offset as usize]))
        }
    }
}

/// The shared body of the three operator-family harnesses.
fn closing_prefix_lemma(
    run: impl Fn(&mut Evaluator<'_, '_>, u64, Interval) -> Result<TruthValue, EvaluationError>,
) {
    let nodes = [Node::new(NodeKind::True)];
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(0), &nodes).unwrap();
    let time: u64 = kani::any();
    let interval = any_interval();
    kani::assume(interval.end() <= 2);
    let trace: [Vec<PropositionId>; 2] = [Vec::new(), Vec::new()];
    // SAFETY: single-threaded harness initialising the oracle statics.
    unsafe {
        *core::ptr::addr_of_mut!(ORACLE_BASE) = time;
        *core::ptr::addr_of_mut!(PREFIX_LEN) = 1;
        let oracle = &mut *core::ptr::addr_of_mut!(ORACLE);
        for node in 0..2 {
            for offset in 0..4 {
                let prefix: u8 = kani::any();
                kani::assume(prefix <= 2);
                let extended: u8 = kani::any();
                kani::assume(extended <= 2);
                kani::assume(prefix == 2 || extended == prefix);
                oracle[0][node][offset] = prefix;
                oracle[1][node][offset] = extended;
            }
        }
    }
    let make = |len: usize| Evaluator {
        formula,
        trace: &trace[..len],
        closed: false,
        limits: limits(),
        stats: EvaluationStats::default(),
    };
    let before = run(&mut make(1), time, interval);
    let after = run(&mut make(2), time, interval);
    match &before {
        Ok(TruthValue::Pending) => {
            kani::cover!(after.is_ok(), "pending refined or kept");
        }
        Ok(decided) => {
            assert_eq!(after, Ok(*decided));
            kani::cover!(*decided == TruthValue::True, "decided True kept");
            kani::cover!(*decided == TruthValue::False, "decided False kept");
        }
        // Not claimed stable; unreachable for the boolean operators.
        Err(_) => {}
    }
    kani::cover!(
        before == Ok(TruthValue::Pending) && after == Ok(TruthValue::True),
        "pending becomes decided on extension"
    );
}

/// Proves (compositional closing-prefix lemma, operator level): let the
/// verdict of each operand at each time be given by a table, once for a prefix
/// and once for the same prefix extended by one instant, constrained so every
/// operand verdict decided on the prefix is unchanged on the extension
/// (`Pending` may become decided). Then for `and`, `or`, `implies` and `equivalent`, over a symbolic verdict time (the full `u64` range, including the overflow
/// edge): a decided `Ok` verdict on the prefix equals the verdict on the extension.
///
/// Together with the atomic proof `closing_prefix_atomic_verdict_is_stable`
/// (leaf case) this is the induction step of "a decided open-prefix verdict is
/// stable under appending an instant" for every operator body that has a
/// subformula.
///
/// Does not claim: the `Not` arm and the dispatch in `at` preserve stability
/// (they are one-line `TruthValue::not` and routing, covered by unit tests, not
/// this lemma); that the real operand verdicts satisfy the table constraint
/// (that is the induction hypothesis, discharged by the atomic proof only at
/// the leaves); that a typed error on the prefix recurs on the extension (it
/// need not: the extension can decide earlier and skip the time-overflow edge
/// the prefix run reached); anything about work-limit or depth errors (the table never
/// returns an error and the work counters are not consulted); intervals ending
/// past 2 (the table is modelled over 4 times); or that a verdict becomes
/// decided once the prefix passes `time + lookahead`.
#[kani::proof]
#[kani::stub(Evaluator::at, Evaluator::oracle_at)]
#[kani::unwind(6)]
fn closing_prefix_boolean_operator_lemma() {
    let (left, right) = (NodeId(0), NodeId(1));
    // One choice shared by both evaluations of the lemma.
    let which: u8 = kani::any();
    closing_prefix_lemma(|evaluator, time, _interval| match which % 4 {
        0 => evaluator.and(left, right, time, 1),
        1 => evaluator.or(left, right, time, 1),
        2 => evaluator.implies(left, right, time, 1),
        _ => evaluator.equivalent(left, right, time, 1),
    });
}

/// Proves (compositional closing-prefix lemma, operator level): let the
/// verdict of each operand at each time be given by a table, once for a prefix
/// and once for the same prefix extended by one instant, constrained so every
/// operand verdict decided on the prefix is unchanged on the extension
/// (`Pending` may become decided). Then for `future` and `globally`, over a symbolic verdict time (the full `u64` range, including the overflow
/// edge) and every interval with end at most 2: a decided `Ok` verdict on the prefix equals the verdict on the extension.
///
/// Together with the atomic proof `closing_prefix_atomic_verdict_is_stable`
/// (leaf case) this is the induction step of "a decided open-prefix verdict is
/// stable under appending an instant" for every operator body that has a
/// subformula.
///
/// Does not claim: the `Not` arm and the dispatch in `at` preserve stability
/// (they are one-line `TruthValue::not` and routing, covered by unit tests, not
/// this lemma); that the real operand verdicts satisfy the table constraint
/// (that is the induction hypothesis, discharged by the atomic proof only at
/// the leaves); that a typed error on the prefix recurs on the extension (it
/// need not: the extension can decide earlier and skip the time-overflow edge
/// the prefix run reached); anything about work-limit or depth errors (the table never
/// returns an error and the work counters are not consulted); intervals ending
/// past 2 (the table is modelled over 4 times); or that a verdict becomes
/// decided once the prefix passes `time + lookahead`.
#[kani::proof]
#[kani::stub(Evaluator::at, Evaluator::oracle_at)]
#[kani::unwind(6)]
fn closing_prefix_window_operator_lemma() {
    let (left, right) = (NodeId(0), NodeId(1));
    // One choice shared by both evaluations of the lemma.
    let which: u8 = kani::any();
    closing_prefix_lemma(|evaluator, time, interval| {
        if which % 2 == 0 {
            evaluator.future(interval, right, time, 1)
        } else {
            evaluator.globally(interval, right, time, 1)
        }
    });
}

/// Proves (compositional closing-prefix lemma, operator level): let the
/// verdict of each operand at each time be given by a table, once for a prefix
/// and once for the same prefix extended by one instant, constrained so every
/// operand verdict decided on the prefix is unchanged on the extension
/// (`Pending` may become decided). Then for `until` and its `Release` negation, over a symbolic verdict time (the full `u64` range, including the overflow
/// edge) and every interval with end at most 2: a decided `Ok` verdict on the prefix equals the verdict on the extension.
///
/// Together with the atomic proof `closing_prefix_atomic_verdict_is_stable`
/// (leaf case) this is the induction step of "a decided open-prefix verdict is
/// stable under appending an instant" for every operator body that has a
/// subformula.
///
/// Does not claim: the `Not` arm and the dispatch in `at` preserve stability
/// (they are one-line `TruthValue::not` and routing, covered by unit tests, not
/// this lemma); that the real operand verdicts satisfy the table constraint
/// (that is the induction hypothesis, discharged by the atomic proof only at
/// the leaves); that a typed error on the prefix recurs on the extension (it
/// need not: the extension can decide earlier and skip the time-overflow edge
/// the prefix run reached); anything about work-limit or depth errors (the table never
/// returns an error and the work counters are not consulted); intervals ending
/// past 2 (the table is modelled over 4 times); or that a verdict becomes
/// decided once the prefix passes `time + lookahead`.
#[kani::proof]
#[kani::stub(Evaluator::at, Evaluator::oracle_at)]
#[kani::unwind(6)]
fn closing_prefix_until_release_lemma() {
    let (left, right) = (NodeId(0), NodeId(1));
    // One choice shared by both evaluations of the lemma.
    let which: u8 = kani::any();
    closing_prefix_lemma(|evaluator, time, interval| {
        evaluator.until(
            time,
            interval.start(),
            interval.end(),
            left,
            right,
            which % 2 == 0,
            1,
        )
    });
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
