//! TL-222: property laws for infinite-trace and partial-valuation semantics.
//!
//! Each law is a proptest property over generated formulas and lassos, run
//! under a fixed ChaCha seed and accepted-case count from `CAMPAIGN` with no
//! discards (strategies construct only admissible inputs), so a run
//! reproduces byte for byte. The provider is the subject. Laws that compare it
//! with an oracle use the successor-fixpoint procedure in `tl-oracle`, which
//! shares no code with the provider.
//!
//! Every law has a matching `control_*` test that feeds the same checker a
//! deliberately wrong claim and requires the checker to reject it, so a
//! checker that cannot fail is caught here. The mutation record for the
//! provider itself is in `spec/reviews/` (TL-223).

#![cfg(feature = "infinite-trace")]

#[path = "support/lasso_kit.rs"]
mod kit;

use kit::*;
use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use tl_mltl::infinite::{
    evaluate_prefix_safety, Disposition, EvaluationLimit, InfiniteResult, PrefixRequest,
    ResultReason, SettlementEvidence, TruthAvailability,
};
use tl_mltl::{evaluate_closed_at, EvaluationLimits, TruthValue};
use tl_syntax::{
    Formula as FiniteFormula, Interval, Node, NodeId as FiniteId, NodeKind, PropositionId,
    SemanticProfile,
};

/// (property, seed byte, accepted cases). The 32-byte ChaCha seed repeats the
/// seed byte.
const CAMPAIGN: &[(&str, u8, u32)] = &[
    ("unbounded_duality", 0x41, 1000),
    ("lasso_unrolling", 0x42, 1000),
    ("bounded_embedding", 0x43, 1000),
    ("prefix_refutation_sound", 0x44, 1000),
    ("no_liveness_from_prefix", 0x45, 1000),
    ("partial_monotone", 0x46, 1000),
    ("missing_not_conflicting", 0x47, 1000),
    ("fairness_weakening", 0x48, 1000),
    ("provider_matches_fixpoint", 0x49, 1000),
];

fn run<S: Strategy>(name: &str, strategy: S, test: impl Fn(S::Value) -> Result<(), TestCaseError>) {
    let &(_, seed, cases) = CAMPAIGN
        .iter()
        .find(|row| row.0 == name)
        .expect("property is recorded in CAMPAIGN");
    let config = Config {
        cases,
        failure_persistence: None,
        max_global_rejects: 0,
        max_local_rejects: 0,
        ..Config::default()
    };
    let rng = TestRng::from_seed(RngAlgorithm::ChaCha, &[seed; 32]);
    let mut runner = TestRunner::new_with_rng(config, rng);
    if let Err(failure) = runner.run(&strategy, test) {
        panic!("{name} (seed byte {seed:#04x}, {cases} cases): {failure}");
    }
}

// --------------------------------------------------------------- strategies

fn closed_iv() -> impl Strategy<Value = Iv> {
    (0u32..3, 0u32..3).prop_map(|(a, d)| Iv::Closed(a, a + d))
}
fn open_iv() -> impl Strategy<Value = Iv> {
    (0u32..4).prop_map(Iv::Open)
}
fn any_iv() -> impl Strategy<Value = Iv> {
    prop_oneof![closed_iv(), open_iv()]
}

fn leaf() -> impl Strategy<Value = T> {
    prop_oneof![
        Just(T::P(0)),
        Just(T::P(1)),
        Just(not(T::P(0))),
        Just(T::True),
        Just(T::False),
    ]
}

/// Formulas over every operator of the infinite profile, depth at most three.
fn tree() -> impl Strategy<Value = T> {
    leaf().prop_recursive(3, 12, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(not),
            inner.clone().prop_map(|x| T::Y(bx(x))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::And(bx(x), bx(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::Or(bx(x), bx(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::Implies(bx(x), bx(y))),
            (any_iv(), inner.clone()).prop_map(|(i, x)| T::F(i, bx(x))),
            (any_iv(), inner.clone()).prop_map(|(i, x)| T::G(i, bx(x))),
            (any_iv(), inner.clone()).prop_map(|(i, x)| T::Once(i, bx(x))),
            (any_iv(), inner.clone()).prop_map(|(i, x)| T::Hist(i, bx(x))),
            (any_iv(), inner.clone(), inner.clone()).prop_map(|(i, x, y)| T::U(i, bx(x), bx(y))),
            (any_iv(), inner.clone(), inner.clone()).prop_map(|(i, x, y)| T::R(i, bx(x), bx(y))),
            (any_iv(), inner.clone(), inner.clone()).prop_map(|(i, x, y)| T::Since(
                i,
                bx(x),
                bx(y)
            )),
            (any_iv(), inner.clone(), inner).prop_map(|(i, x, y)| T::Trig(i, bx(x), bx(y))),
        ]
    })
}

/// Closed intervals starting at zero: the fragment on which the finite and
/// infinite profiles give `U` and `R` the same guard range (see
/// `until_guard_range_differs_between_the_finite_and_infinite_profiles`).
fn zero_iv() -> impl Strategy<Value = Iv> {
    (0u32..4).prop_map(|b| Iv::Closed(0, b))
}

/// Bounded-future formulas only: the fragment the finite evaluator decides.
/// `F` and `G` take any closed interval; `U` and `R` start at zero.
fn bounded_tree() -> impl Strategy<Value = T> {
    leaf().prop_recursive(3, 12, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(not),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::And(bx(x), bx(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::Or(bx(x), bx(y))),
            (closed_iv(), inner.clone()).prop_map(|(i, x)| T::F(i, bx(x))),
            (closed_iv(), inner.clone()).prop_map(|(i, x)| T::G(i, bx(x))),
            (zero_iv(), inner.clone(), inner.clone()).prop_map(|(i, x, y)| T::U(i, bx(x), bx(y))),
            (zero_iv(), inner.clone(), inner).prop_map(|(i, x, y)| T::R(i, bx(x), bx(y))),
        ]
    })
}

fn complete_cell() -> impl Strategy<Value = Cell> {
    (any::<bool>(), any::<bool>())
        .prop_map(|(a, b)| [if a { Tr } else { F }, if b { Tr } else { F }])
}

fn partial_value() -> impl Strategy<Value = tl_syntax::PartialValue> {
    prop_oneof![3 => Just(Tr), 3 => Just(F), 1 => Just(M), 1 => Just(C)]
}

fn partial_cell() -> impl Strategy<Value = Cell> {
    (partial_value(), partial_value()).prop_map(|(a, b)| [a, b])
}

/// A complete lasso: prefix 0..=3 cells, loop 1..=3 cells.
fn complete_lasso() -> impl Strategy<Value = (Vec<Cell>, Vec<Cell>)> {
    (
        prop::collection::vec(complete_cell(), 0..=3),
        prop::collection::vec(complete_cell(), 1..=3),
    )
}

fn partial_lasso() -> impl Strategy<Value = (Vec<Cell>, Vec<Cell>)> {
    (
        prop::collection::vec(partial_cell(), 0..=2),
        prop::collection::vec(partial_cell(), 1..=2),
    )
}

// ------------------------------------------------------------------ helpers

fn disp(result: &InfiniteResult) -> Disposition {
    result.disposition
}

fn at(built: &Built, word: &tl_syntax::LassoTraceDocument, position: u64) -> InfiniteResult {
    provider(built, word, position)
}

fn same_verdict(
    lhs: &T,
    rhs: &T,
    word: &tl_syntax::LassoTraceDocument,
    positions: u64,
) -> Result<(), TestCaseError> {
    let (l, r) = (build(lhs, &[]), build(rhs, &[]));
    for p in 0..positions {
        prop_assert_eq!(
            disp(&at(&l, word, p)),
            disp(&at(&r, word, p)),
            "{:?} vs {:?} at {}",
            lhs,
            rhs,
            p
        );
    }
    Ok(())
}

// -------------------------------------------------------------- law: duality

/// Checks `not(lhs)` against `rhs` on a complete lasso, and the provider
/// against the fixpoint reference on both sides.
fn check_duality(lhs: &T, rhs: &T, prefix: &[Cell], tail: &[Cell]) -> Result<(), TestCaseError> {
    let word = lasso(prefix, tail);
    same_verdict(&not(lhs.clone()), rhs, &word, 6)?;
    for p in 0..6 {
        for side in [lhs, rhs] {
            let built = build(side, &[]);
            let why = disagreement(&at(&built, &word, p), &fixpoint(&built, &word, p));
            prop_assert!(why.is_none(), "{side:?} at {p}: {why:?}");
        }
    }
    Ok(())
}

// Trace: TC-146, TC-149, FR-031-AC-1, FR-031-AC-3
#[test]
fn unbounded_future_operators_are_dual() {
    run(
        "unbounded_duality",
        (tree(), tree(), 0u32..4, complete_lasso()),
        |(x, y, a, (prefix, tail))| {
            let i = Iv::Open(a);
            // not F[a,) x == G[a,) not x
            check_duality(
                &T::F(i, bx(x.clone())),
                &T::G(i, bx(not(x.clone()))),
                &prefix,
                &tail,
            )?;
            // not (x U[a,) y) == (not x) R[a,) (not y)
            check_duality(
                &T::U(i, bx(x.clone()), bx(y.clone())),
                &T::R(i, bx(not(x)), bx(not(y))),
                &prefix,
                &tail,
            )
        },
    );
}

#[test]
fn control_duality_rejects_a_wrong_dual() {
    // not F[0,) p0  is  G[0,) not p0, never  G[0,) p0.
    let wrong = check_duality(
        &T::F(Iv::Open(0), bx(T::P(0))),
        &T::G(Iv::Open(0), bx(T::P(0))),
        &[],
        &[[Tr, F]],
    );
    assert!(wrong.is_err());
}

// ----------------------------------------------------------- law: unrolling

/// Every way of writing the same infinite word as a lasso.
fn rewritings(prefix: &[Cell], tail: &[Cell], rotate: usize) -> Vec<(Vec<Cell>, Vec<Cell>)> {
    let mut out = Vec::new();
    // u . (v) . v^w
    let mut p = prefix.to_vec();
    p.extend_from_slice(tail);
    out.push((p, tail.to_vec()));
    // u . (v v)^w
    let mut vv = tail.to_vec();
    vv.extend_from_slice(tail);
    out.push((prefix.to_vec(), vv));
    // rotate the loop forward by k, absorbing the head into the prefix
    let k = rotate % tail.len();
    let mut p = prefix.to_vec();
    p.extend_from_slice(&tail[..k]);
    let mut v = tail[k..].to_vec();
    v.extend_from_slice(&tail[..k]);
    out.push((p, v));
    // pull the loop entry back over a prefix cell equal to the last loop cell
    if let Some((last, rest)) = prefix.split_last() {
        if Some(last) == tail.last() {
            let mut v = vec![*last];
            v.extend_from_slice(&tail[..tail.len() - 1]);
            out.push((rest.to_vec(), v));
        }
    }
    out
}

fn check_unrolling(
    formula: &T,
    prefix: &[Cell],
    tail: &[Cell],
    rotate: usize,
    positions: u64,
) -> Result<(), TestCaseError> {
    let built = build(formula, &[]);
    let base = lasso(prefix, tail);
    for (p, v) in rewritings(prefix, tail, rotate) {
        let other = lasso(&p, &v);
        for position in 0..positions {
            let (a, b) = (at(&built, &base, position), at(&built, &other, position));
            prop_assert_eq!(
                disp(&a),
                disp(&b),
                "{:?}: {:?}.{:?} vs {:?}.{:?} at {}",
                formula,
                prefix,
                tail,
                p,
                v,
                position
            );
            let reference = fixpoint(&built, &other, position);
            prop_assert!(disagreement(&b, &reference).is_none());
        }
    }
    Ok(())
}

// Trace: TC-150, TC-151, FR-032-AC-1
#[test]
fn rewriting_a_lasso_never_changes_the_verdict() {
    run(
        "lasso_unrolling",
        (tree(), complete_lasso(), 0usize..3),
        |(formula, (prefix, tail), rotate)| check_unrolling(&formula, &prefix, &tail, rotate, 8),
    );
}

/// The helper must produce the same infinite word, or the law proves nothing.
#[test]
fn control_every_rewriting_denotes_the_same_infinite_word() {
    let at_time = |prefix: &[Cell], tail: &[Cell], t: usize| {
        if t < prefix.len() {
            prefix[t]
        } else {
            tail[(t - prefix.len()) % tail.len()]
        }
    };
    let (prefix, tail) = (vec![[F, Tr], [Tr, Tr]], vec![[Tr, F], [F, F], [Tr, Tr]]);
    let mut seen = 0;
    for rotate in 0..3 {
        for (p, v) in rewritings(&prefix, &tail, rotate) {
            for t in 0..24 {
                assert_eq!(at_time(&p, &v, t), at_time(&prefix, &tail, t));
            }
            seen += 1;
        }
    }
    assert_eq!(seen, 12);
    // The prefix-pull rewriting applies when the last prefix cell equals the
    // last loop cell.
    let pulled = rewritings(&[[F, F], [Tr, Tr]], &[[F, Tr], [Tr, Tr]], 0);
    assert_eq!(pulled.len(), 4);
    assert_eq!(pulled[3].0, vec![[F, F]]);
}

// ----------------------------------------------------- law: bounded embedding

fn finite_nodes(tree: &T, nodes: &mut Vec<Node>) -> FiniteId {
    let iv = |i: &Iv| match i {
        Iv::Closed(a, b) => Interval::new(*a, *b).unwrap(),
        Iv::Open(_) => unreachable!("bounded fragment"),
    };
    let kind = match tree {
        T::True => NodeKind::True,
        T::False => NodeKind::False,
        T::P(p) => NodeKind::Proposition {
            proposition: PropositionId(*p),
        },
        T::Not(x) => NodeKind::Not {
            operand: finite_nodes(x, nodes),
        },
        T::And(x, y) => NodeKind::And {
            left: finite_nodes(x, nodes),
            right: finite_nodes(y, nodes),
        },
        T::Or(x, y) => NodeKind::Or {
            left: finite_nodes(x, nodes),
            right: finite_nodes(y, nodes),
        },
        T::F(i, x) => NodeKind::Future {
            interval: iv(i),
            operand: finite_nodes(x, nodes),
        },
        T::G(i, x) => NodeKind::Globally {
            interval: iv(i),
            operand: finite_nodes(x, nodes),
        },
        T::U(i, x, y) => NodeKind::Until {
            interval: iv(i),
            left: finite_nodes(x, nodes),
            right: finite_nodes(y, nodes),
        },
        T::R(i, x, y) => NodeKind::Release {
            interval: iv(i),
            left: finite_nodes(x, nodes),
            right: finite_nodes(y, nodes),
        },
        other => unreachable!("outside the bounded fragment: {other:?}"),
    };
    nodes.push(Node::new(kind));
    FiniteId(u32::try_from(nodes.len() - 1).unwrap())
}

/// Sum of interval upper bounds along the deepest path: a safe lookahead.
fn lookahead(tree: &T) -> u64 {
    let end = |i: &Iv| match i {
        Iv::Closed(_, b) => u64::from(*b),
        Iv::Open(_) => 0,
    };
    match tree {
        T::True | T::False | T::P(_) => 0,
        T::Not(x) => lookahead(x),
        T::And(x, y) | T::Or(x, y) => lookahead(x).max(lookahead(y)),
        T::F(i, x) | T::G(i, x) => end(i) + lookahead(x),
        T::U(i, x, y) | T::R(i, x, y) => end(i) + lookahead(x).max(lookahead(y)),
        _ => unreachable!("bounded fragment"),
    }
}

fn unrolled(prefix: &[Cell], tail: &[Cell], len: usize) -> Vec<Vec<PropositionId>> {
    (0..len)
        .map(|t| {
            let cell = if t < prefix.len() {
                prefix[t]
            } else {
                tail[(t - prefix.len()) % tail.len()]
            };
            (0..2)
                .filter(|a| cell[*a] == Tr)
                .map(|a| PropositionId(a as u32))
                .collect()
        })
        .collect()
}

fn check_embedding(
    formula: &T,
    prefix: &[Cell],
    tail: &[Cell],
    position: u64,
    extra: u64,
) -> Result<(), TestCaseError> {
    let built = build(formula, &[]);
    let word = lasso(prefix, tail);
    let lasso_verdict = at(&built, &word, position).disposition;
    let mut nodes = Vec::new();
    finite_nodes(formula, &mut nodes);
    let finite = FiniteFormula::new(
        SemanticProfile::ClosedTraceV1,
        FiniteId(u32::try_from(nodes.len() - 1).unwrap()),
        &nodes,
    )
    .unwrap();
    let length = usize::try_from(position + lookahead(formula) + 1 + extra).unwrap();
    let trace = unrolled(prefix, tail, length);
    let verdict = evaluate_closed_at(
        finite,
        "embedding",
        &trace,
        "embedding-trace",
        position,
        EvaluationLimits::default(),
    )
    .unwrap()
    .verdict;
    let expected = match verdict {
        TruthValue::True => Disposition::Proved,
        TruthValue::False => Disposition::Refuted,
        TruthValue::Pending => panic!("a long enough prefix must decide {formula:?}"),
    };
    prop_assert_eq!(lasso_verdict, expected, "{:?} at {}", formula, position);
    Ok(())
}

// Trace: TC-149, FR-031-AC-3
#[test]
fn a_bounded_formula_on_a_lasso_equals_the_finite_evaluator_on_an_unrolled_prefix() {
    run(
        "bounded_embedding",
        (bounded_tree(), complete_lasso(), 0u64..6, 0u64..3),
        |(formula, (prefix, tail), position, extra)| {
            check_embedding(&formula, &prefix, &tail, position, extra)
        },
    );
}

/// FR-001 (finite) requires the left operand of `U[a,b]` on `[a,i)`; FR-031
/// (infinite) requires it on every offset from zero, as QSpec FR-161 does. For
/// `a > 0` the two profiles therefore disagree on the same word. The embedding
/// law is restricted to `a = 0` for `U` and `R`; this test records the
/// divergence so a later spec decision shows up as a test change.
// Trace: TC-149, FR-001-AC-1, FR-031-AC-3
#[test]
fn until_guard_range_differs_between_the_finite_and_infinite_profiles() {
    let formula = T::U(Iv::Closed(1, 1), bx(T::P(0)), bx(not(T::P(0))));
    let (prefix, tail) = (vec![[F, F], [F, F]], vec![[F, F]]);
    // p0 is false everywhere: the witness not-p0 holds at offset 1, and the
    // guard p0 is false at offset 0.
    let infinite = at(&build(&formula, &[]), &lasso(&prefix, &tail), 0);
    assert_eq!(infinite.disposition, Disposition::Refuted);
    assert!(check_embedding(&formula, &prefix, &tail, 0, 0).is_err());
}

// ------------------------------------------- law: prefix refutation soundness

fn prefix_result(built: &Built, prefix: &[Cell]) -> Option<InfiniteResult> {
    let word = lasso(prefix, &[[F, F]]);
    let graph_id = built.doc.content_identity().unwrap();
    evaluate_prefix_safety(&PrefixRequest {
        formula: &built.doc,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: word.propositions(),
        observations: word.prefix(),
        limit: EvaluationLimit::default(),
    })
    .ok()
}

/// `G[0,) psi` with a bounded-future `psi`: the exact safety fragment.
fn safety_claim() -> impl Strategy<Value = T> {
    bounded_tree().prop_map(|psi| T::G(Iv::Open(0), bx(psi)))
}

fn check_prefix_refutation(
    claim: &T,
    prefix: &[Cell],
    extension: &[Cell],
    tail: &[Cell],
) -> Result<(), TestCaseError> {
    let built = build(claim, &[]);
    let Some(result) = prefix_result(&built, prefix) else {
        return Ok(());
    };
    if result.disposition != Disposition::Refuted {
        return Ok(());
    }
    let mut full = prefix.to_vec();
    full.extend_from_slice(extension);
    let word = lasso(&full, tail);
    let exact = at(&built, &word, 0);
    prop_assert_eq!(
        exact.disposition,
        Disposition::Refuted,
        "{:?}: prefix {:?} refuted but extension {:?}.{:?} is {:?}",
        claim,
        prefix,
        extension,
        tail,
        exact.disposition
    );
    let reference = fixpoint(&built, &word, 0).unwrap();
    prop_assert_eq!(reference.verdict, tl_oracle::Verdict::Refuted);
    Ok(())
}

// Trace: TC-157, FR-033-AC-2
#[test]
fn a_prefix_refutation_holds_for_every_infinite_extension() {
    run(
        "prefix_refutation_sound",
        (
            safety_claim(),
            prop::collection::vec(partial_cell(), 1..=4),
            prop::collection::vec(partial_cell(), 0..=2),
            prop::collection::vec(partial_cell(), 1..=2),
        ),
        |(claim, prefix, extension, tail)| {
            check_prefix_refutation(&claim, &prefix, &extension, &tail)
        },
    );
}

#[test]
fn control_prefix_refutation_is_not_vacuous() {
    // The prefix route does refute a violated safety claim, so the law above
    // exercises its refutation branch.
    let built = build(&T::G(Iv::Open(0), bx(T::P(0))), &[]);
    let result = prefix_result(&built, &[[Tr, F], [F, F]]).unwrap();
    assert_eq!(result.disposition, Disposition::Refuted);
    // A wrong refutation would be caught: pretend the claim holds elsewhere.
    let holds = build(&T::G(Iv::Open(0), bx(T::P(1))), &[]);
    let word = lasso(&[[Tr, F], [F, F]], &[[Tr, Tr]]);
    assert_eq!(at(&holds, &word, 0).disposition, Disposition::Refuted);
}

// -------------------------------------------- law: no liveness from a prefix

fn check_no_liveness(
    formula: &T,
    prefix: &[Cell],
    limit: EvaluationLimit,
) -> Result<(), TestCaseError> {
    let built = build(formula, &[]);
    let word = lasso(prefix, &[[F, F]]);
    let graph_id = built.doc.content_identity().unwrap();
    let result = evaluate_prefix_safety(&PrefixRequest {
        formula: &built.doc,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: word.propositions(),
        observations: word.prefix(),
        limit,
    });
    if let Ok(result) = result {
        prop_assert_ne!(
            result.disposition,
            Disposition::Proved,
            "{:?} {:?}",
            formula,
            prefix
        );
        prop_assert_ne!(
            result.truth,
            TruthAvailability::True,
            "{:?} {:?}",
            formula,
            prefix
        );
    }
    Ok(())
}

// Trace: TC-089, TC-157, FR-029-AC-2, FR-033-AC-2
#[test]
fn a_finite_prefix_never_proves_a_property_even_when_evaluation_stops_early() {
    let limits = [
        EvaluationLimit::default(),
        EvaluationLimit {
            max_steps: 1,
            ..EvaluationLimit::default()
        },
        EvaluationLimit {
            max_positions: 2,
            ..EvaluationLimit::default()
        },
        EvaluationLimit {
            deadline: Some(std::time::Instant::now()),
            ..EvaluationLimit::default()
        },
    ];
    run(
        "no_liveness_from_prefix",
        (
            prop_oneof![tree(), safety_claim()],
            prop::collection::vec(partial_cell(), 0..=5),
            0usize..4,
        ),
        |(formula, prefix, which)| check_no_liveness(&formula, &prefix, limits[which]),
    );
}

#[test]
fn control_no_liveness_checker_sees_a_proved_result() {
    // The lasso route can prove F[0,) p0; the checker's predicate is the same
    // one applied to the prefix route, so it must fire on that result.
    let built = build(&T::F(Iv::Open(0), bx(T::P(0))), &[]);
    let proved = at(&built, &lasso(&[], &[[Tr, F]]), 0);
    assert_eq!(proved.disposition, Disposition::Proved);
    assert_eq!(proved.truth, TruthAvailability::True);
}

// ---------------------------------------- law: partial-valuation monotonicity

/// Refines every missing value of a lasso with the given bits.
fn refine(cells: &[Cell], bits: &mut impl Iterator<Item = bool>) -> Vec<Cell> {
    cells
        .iter()
        .map(|cell| {
            let mut out = *cell;
            for v in &mut out {
                if *v == M {
                    *v = if bits.next().unwrap_or(false) { Tr } else { F };
                }
            }
            out
        })
        .collect()
}

fn check_monotone(
    formula: &T,
    fairness: &[T],
    prefix: &[Cell],
    tail: &[Cell],
    bits: &[bool],
) -> Result<(), TestCaseError> {
    let built = build(formula, fairness);
    let coarse = lasso(prefix, tail);
    let mut iter = bits.iter().copied().cycle();
    let (p2, t2) = (refine(prefix, &mut iter), refine(tail, &mut iter));
    let fine = lasso(&p2, &t2);
    for position in 0..5 {
        let (a, b) = (at(&built, &coarse, position), at(&built, &fine, position));
        let conclusive = |d: Disposition| matches!(d, Disposition::Proved | Disposition::Refuted);
        if conclusive(a.disposition) {
            // A refined fair set is a subset of the coarse one, so a conclusive
            // verdict is kept unless fairness removed every completion.
            if b.admitted_completions > 0 {
                prop_assert_eq!(
                    a.disposition,
                    b.disposition,
                    "{:?} fairness {:?}: {:?}.{:?} refined to {:?}.{:?} at {}",
                    formula,
                    fairness,
                    prefix,
                    tail,
                    p2,
                    t2,
                    position
                );
            } else {
                prop_assert_eq!(b.reason, Some(ResultReason::EmptyFairAdmission));
                prop_assert!(!fairness.is_empty());
            }
        }
    }
    Ok(())
}

// Trace: TC-143, FR-030-AC-2
#[test]
fn more_information_never_flips_a_conclusive_verdict() {
    run(
        "partial_monotone",
        (
            tree(),
            prop_oneof![
                Just(vec![]),
                Just(vec![T::P(0)]),
                Just(vec![T::F(Iv::Open(0), bx(T::P(1)))])
            ],
            partial_lasso(),
            prop::collection::vec(any::<bool>(), 1..8),
        ),
        |(formula, fairness, (prefix, tail), bits)| {
            check_monotone(&formula, &fairness, &prefix, &tail, &bits)
        },
    );
}

#[test]
fn control_monotonicity_has_conclusive_verdicts_to_constrain() {
    // A tautology is conclusive under missing evidence, and refining the
    // missing cell keeps it: the law's premise is reachable.
    let built = build(&T::Or(bx(T::P(0)), bx(not(T::P(0)))), &[]);
    assert_eq!(
        at(&built, &lasso(&[], &[[M, F]]), 0).disposition,
        Disposition::Proved
    );
    assert!(check_monotone(&T::P(0), &[], &[], &[[M, F]], &[true]).is_ok());
    // A claim that depends on the missing cell is inconclusive until refined,
    // then conclusive: refinement only ever moves in that direction.
    let depends = build(&T::P(0), &[]);
    assert_eq!(
        at(&depends, &lasso(&[], &[[M, F]]), 0).disposition,
        Disposition::Inconclusive
    );
    assert_eq!(
        at(&depends, &lasso(&[], &[[Tr, F]]), 0).disposition,
        Disposition::Proved
    );
}

// ------------------------------------- law: missing and conflicting are distinct

/// Swaps `from` for `to` in every cell.
fn swap(cells: &[Cell], from: tl_syntax::PartialValue, to: tl_syntax::PartialValue) -> Vec<Cell> {
    cells
        .iter()
        .map(|cell| {
            let mut out = *cell;
            for v in &mut out {
                if *v == from {
                    *v = to;
                }
            }
            out
        })
        .collect()
}

fn check_missing_vs_conflicting(
    formula: &T,
    prefix: &[Cell],
    tail: &[Cell],
) -> Result<(), TestCaseError> {
    let built = build(formula, &[]);
    let as_missing = lasso(&swap(prefix, C, M), &swap(tail, C, M));
    let as_conflict = lasso(&swap(prefix, M, C), &swap(tail, M, C));
    for position in 0..5 {
        let (m, c) = (
            at(&built, &as_missing, position),
            at(&built, &as_conflict, position),
        );
        // Same possibilities: same disposition and admitted completions.
        prop_assert_eq!(m.disposition, c.disposition);
        prop_assert_eq!(m.admitted_completions, c.admitted_completions);
        // Distinct outcomes whenever the result is inconclusive for lack of
        // evidence and the trace holds only one of the two kinds.
        let has = |word: &[Cell], v| word.iter().any(|cell| cell.contains(&v));
        let missing_only = has(prefix, M) || has(tail, M);
        let conflict_only = has(prefix, C) || has(tail, C);
        if m.disposition == Disposition::Inconclusive && m.reason.is_some() {
            if missing_only && !conflict_only {
                prop_assert_eq!(m.reason, Some(ResultReason::MissingObservation));
                prop_assert_eq!(c.reason, Some(ResultReason::ConflictingObservation));
            }
            if conflict_only && !missing_only {
                prop_assert_eq!(m.reason, Some(ResultReason::MissingObservation));
                prop_assert_eq!(c.reason, Some(ResultReason::ConflictingObservation));
            }
        }
    }
    Ok(())
}

// Trace: TC-143, FR-030-AC-2
#[test]
fn a_missing_value_and_a_conflicting_value_stay_distinct_outcomes() {
    run(
        "missing_not_conflicting",
        (tree(), partial_lasso()),
        |(formula, (prefix, tail))| check_missing_vs_conflicting(&formula, &prefix, &tail),
    );
}

#[test]
fn control_missing_vs_conflicting_is_not_vacuous() {
    let built = build(&T::P(0), &[]);
    let m = at(&built, &lasso(&[], &[[M, F]]), 0);
    let c = at(&built, &lasso(&[], &[[C, F]]), 0);
    assert_eq!(m.reason, Some(ResultReason::MissingObservation));
    assert_eq!(c.reason, Some(ResultReason::ConflictingObservation));
    assert_ne!(m.reason, c.reason);
}

// ------------------------------------------- law: fairness-premise weakening

fn flags(result: &InfiniteResult) -> (bool, bool) {
    match &result.evidence {
        Some(SettlementEvidence::ExhaustiveTrace(t)) => {
            (t.satisfying.is_some(), t.falsifying.is_some())
        }
        _ => (false, false),
    }
}

fn check_weakening(
    formula: &T,
    premises: &[T],
    prefix: &[Cell],
    tail: &[Cell],
    keep: usize,
) -> Result<(), TestCaseError> {
    let strong = build(formula, premises);
    let weak = build(formula, &premises[..keep.min(premises.len())]);
    let word = lasso(prefix, tail);
    for position in 0..5 {
        let (s, w) = (at(&strong, &word, position), at(&weak, &word, position));
        prop_assert!(w.admitted_completions >= s.admitted_completions);
        let (s_true, s_false) = flags(&s);
        let (w_true, w_false) = flags(&w);
        // Dropping a premise only adds admitted completions: nothing the
        // strong run could witness is lost.
        prop_assert!(!s_true || w_true);
        prop_assert!(!s_false || w_false);
        if s.disposition == Disposition::Refuted {
            prop_assert_ne!(w.disposition, Disposition::Proved);
        }
        if s.disposition == Disposition::Proved {
            prop_assert_ne!(w.disposition, Disposition::Refuted);
        }
    }
    Ok(())
}

// Trace: TC-152, TC-153, FR-032-AC-2, FR-032-AC-3
#[test]
fn dropping_a_fairness_premise_never_turns_a_refutation_into_satisfaction() {
    run(
        "fairness_weakening",
        (
            tree(),
            prop::collection::vec(tree(), 1..=2),
            partial_lasso(),
            0usize..2,
        ),
        |(formula, premises, (prefix, tail), keep)| {
            check_weakening(&formula, &premises, &prefix, &tail, keep)
        },
    );
}

#[test]
fn control_weakening_is_not_vacuous() {
    // Fairness p0 removes the completions where p0 is false: with it the claim
    // p0 at 0 on (M)^w is proved (every fair completion has p0 infinitely
    // often... which forces the single loop cell true). Without it, only
    // inconclusive. The law's checker sees the strong run's evidence flags.
    let word = lasso(&[], &[[M, F]]);
    let strong = at(&build(&T::P(0), &[T::P(0)]), &word, 0);
    let weak = at(&build(&T::P(0), &[]), &word, 0);
    assert_eq!(strong.disposition, Disposition::Proved);
    assert_eq!(weak.disposition, Disposition::Inconclusive);
    assert!(check_weakening(&T::P(0), &[T::P(0)], &[], &[[M, F]], 0).is_ok());
}

// -------------------------------------- law: provider matches the reference

#[test]
fn the_provider_matches_the_fixpoint_reference_on_generated_partial_lassos() {
    run(
        "provider_matches_fixpoint",
        (
            tree(),
            prop::collection::vec(tree(), 0..=2),
            partial_lasso(),
            0u64..6,
        ),
        |(formula, premises, (prefix, tail), position)| {
            let built = build(&formula, &premises);
            let word = lasso(&prefix, &tail);
            let why = disagreement(
                &at(&built, &word, position),
                &fixpoint(&built, &word, position),
            );
            prop_assert!(why.is_none(), "{:?} {:?}: {:?}", formula, premises, why);
            let scan = scanning(&built, &word, position);
            prop_assert_eq!(fixpoint(&built, &word, position), scan);
            Ok(())
        },
    );
}
