//! TL-226: algebraic laws every finite-trace MLTL evaluation must satisfy.
//!
//! A law compares the evaluator with the mathematics, not with another
//! implementation: both sides of each law are production evaluations of
//! different formulas, so a defect in an interval bound, a duality case or the
//! horizon arithmetic breaks the equality even where an oracle and the
//! evaluator might share the defect.
//!
//! Every property runs under a fixed ChaCha seed and accepted-case count from
//! `CAMPAIGN`, with no discards (strategies construct only admissible inputs),
//! so a run is reproducible byte for byte. `CAMPAIGN` is the record the
//! campaign report cites.

use proptest::prelude::*;
use proptest::test_runner::{Config, RngAlgorithm, TestRng, TestRunner};
use tl_mltl::{
    analyze_horizon, evaluate_closed_at, evaluate_past, evaluate_prefix_at, ClockBinding,
    EvaluationLimits, PastEvaluationLimits, PastEvaluationRelationInput, PositionHistoryDocument,
    PositionObservation, TruthValue,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

/// (property, seed byte, accepted cases). The 32-byte ChaCha seed repeats the
/// seed byte.
const CAMPAIGN: &[(&str, u8, u32)] = &[
    ("boolean_duality", 0x21, 10_000),
    ("future_duality", 0x22, 10_000),
    ("past_duality", 0x23, 10_000),
    ("interval_split", 0x24, 10_000),
    ("past_interval_split", 0x25, 10_000),
    ("interval_shift", 0x26, 10_000),
    ("suffix_evaluation", 0x27, 10_000),
    ("suffix_monotonicity", 0x28, 10_000),
    ("horizon_stability", 0x29, 10_000),
    ("horizon_arithmetic", 0x2a, 10_000),
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

// ------------------------------------------------------------------ formulas

/// A formula tree over two propositions. Lowering to the node arena shares
/// nothing with evaluation.
#[derive(Clone, Debug)]
enum T {
    True,
    False,
    P(u32),
    Not(Box<T>),
    And(Box<T>, Box<T>),
    Or(Box<T>, Box<T>),
    Implies(Box<T>, Box<T>),
    Equivalent(Box<T>, Box<T>),
    F(u32, u32, Box<T>),
    G(u32, u32, Box<T>),
    U(u32, u32, Box<T>, Box<T>),
    R(u32, u32, Box<T>, Box<T>),
    Once(u32, u32, Box<T>),
    Hist(u32, u32, Box<T>),
    Since(u32, u32, Box<T>, Box<T>),
    Trig(u32, u32, Box<T>, Box<T>),
}

fn b(t: T) -> Box<T> {
    Box::new(t)
}

fn not(t: T) -> T {
    T::Not(b(t))
}

fn lower(tree: &T, nodes: &mut Vec<Node>) -> NodeId {
    let iv = |a: u32, c: u32| Interval::new(a, c).unwrap();
    let kind = match tree {
        T::True => NodeKind::True,
        T::False => NodeKind::False,
        T::P(p) => NodeKind::Proposition {
            proposition: PropositionId(*p),
        },
        T::Not(x) => NodeKind::Not {
            operand: lower(x, nodes),
        },
        T::And(x, y) => NodeKind::And {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Or(x, y) => NodeKind::Or {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Implies(x, y) => NodeKind::Implies {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Equivalent(x, y) => NodeKind::Equivalent {
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::F(a, c, x) => NodeKind::Future {
            interval: iv(*a, *c),
            operand: lower(x, nodes),
        },
        T::G(a, c, x) => NodeKind::Globally {
            interval: iv(*a, *c),
            operand: lower(x, nodes),
        },
        T::U(a, c, x, y) => NodeKind::Until {
            interval: iv(*a, *c),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::R(a, c, x, y) => NodeKind::Release {
            interval: iv(*a, *c),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Once(a, c, x) => NodeKind::Once {
            interval: iv(*a, *c),
            operand: lower(x, nodes),
        },
        T::Hist(a, c, x) => NodeKind::Historically {
            interval: iv(*a, *c),
            operand: lower(x, nodes),
        },
        T::Since(a, c, x, y) => NodeKind::Since {
            interval: iv(*a, *c),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
        T::Trig(a, c, x, y) => NodeKind::Triggered {
            interval: iv(*a, *c),
            left: lower(x, nodes),
            right: lower(y, nodes),
        },
    };
    nodes.push(Node::new(kind));
    NodeId(u32::try_from(nodes.len() - 1).unwrap())
}

fn arena(tree: &T) -> Vec<Node> {
    let mut nodes = Vec::new();
    lower(tree, &mut nodes);
    nodes
}

fn formula(profile: SemanticProfile, nodes: &[Node]) -> Formula<'_> {
    Formula::new(
        profile,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
}

fn interval() -> impl Strategy<Value = (u32, u32)> {
    (0_u32..=4, 0_u32..=3).prop_map(|(a, width)| (a, a + width))
}

fn leaf() -> impl Strategy<Value = T> {
    prop_oneof![
        1 => Just(T::True),
        1 => Just(T::False),
        4 => (0_u32..2).prop_map(T::P),
    ]
}

/// Boolean and future-time formulas, depth at most 3.
fn future_tree() -> impl Strategy<Value = T> {
    leaf().prop_recursive(3, 24, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(not),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::And(b(x), b(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::Or(b(x), b(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::Implies(b(x), b(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::Equivalent(b(x), b(y))),
            (interval(), inner.clone()).prop_map(|((a, c), x)| T::F(a, c, b(x))),
            (interval(), inner.clone()).prop_map(|((a, c), x)| T::G(a, c, b(x))),
            (interval(), inner.clone(), inner.clone()).prop_map(|((a, c), x, y)| T::U(
                a,
                c,
                b(x),
                b(y)
            )),
            (interval(), inner.clone(), inner).prop_map(|((a, c), x, y)| T::R(a, c, b(x), b(y))),
        ]
    })
}

/// Boolean and past-time formulas, depth at most 3.
fn past_tree() -> impl Strategy<Value = T> {
    leaf().prop_recursive(3, 24, 2, |inner| {
        prop_oneof![
            inner.clone().prop_map(not),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::And(b(x), b(y))),
            (inner.clone(), inner.clone()).prop_map(|(x, y)| T::Or(b(x), b(y))),
            (interval(), inner.clone()).prop_map(|((a, c), x)| T::Once(a, c, b(x))),
            (interval(), inner.clone()).prop_map(|((a, c), x)| T::Hist(a, c, b(x))),
            (interval(), inner.clone(), inner.clone()).prop_map(|((a, c), x, y)| T::Since(
                a,
                c,
                b(x),
                b(y)
            )),
            (interval(), inner.clone(), inner).prop_map(|((a, c), x, y)| T::Trig(a, c, b(x), b(y))),
        ]
    })
}

/// A trace of `0..=10` instants over two propositions.
fn word() -> impl Strategy<Value = Vec<[bool; 2]>> {
    prop::collection::vec(any::<[bool; 2]>(), 0..=10)
}

fn trace(word: &[[bool; 2]]) -> Vec<Vec<PropositionId>> {
    word.iter()
        .map(|cell| {
            (0..2)
                .filter(|index| cell[*index])
                .map(|index| PropositionId(index as u32))
                .collect()
        })
        .collect()
}

// --------------------------------------------------------------- evaluation

fn closed(tree: &T, word: &[[bool; 2]], at: u64) -> TruthValue {
    let nodes = arena(tree);
    evaluate_closed_at(
        formula(SemanticProfile::ClosedTraceV1, &nodes),
        "law",
        &trace(word),
        "law-trace",
        at,
        EvaluationLimits::default(),
    )
    .expect("generated closed evaluation succeeds")
    .verdict
}

fn prefix(tree: &T, word: &[[bool; 2]], at: u64) -> TruthValue {
    let nodes = arena(tree);
    evaluate_prefix_at(
        formula(SemanticProfile::OnlinePrefixV1, &nodes),
        "law",
        &trace(word),
        "law-trace",
        false,
        at,
        EvaluationLimits::default(),
    )
    .expect("generated prefix evaluation succeeds")
    .verdict
}

fn lookahead(tree: &T) -> u64 {
    let nodes = arena(tree);
    analyze_horizon(formula(SemanticProfile::ClosedTraceV1, &nodes), "law")
        .expect("generated horizon succeeds")
        .lookahead
}

fn past(tree: &T, word: &[[bool; 2]], anchor: u64) -> bool {
    let nodes = arena(tree);
    let observations = word
        .iter()
        .enumerate()
        .map(|(position, cell)| {
            let propositions = (0..2)
                .filter(|index| cell[*index])
                .map(|index| PropositionId(index as u32))
                .collect();
            PositionObservation::new(position as u64, propositions, None)
        })
        .collect();
    let history = PositionHistoryDocument::new(
        "law-history",
        1,
        0,
        word.len() as u64 - 1,
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap();
    evaluate_past(
        formula(SemanticProfile::OriginCompleteHistoryV1, &nodes),
        "law",
        &history,
        anchor,
        "law-map",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .expect("generated past evaluation succeeds")
    .verdict
}

/// Both sides must agree under both evaluators, `Pending` included.
fn assert_equivalent(lhs: &T, rhs: &T, word: &[[bool; 2]], at: u64) -> Result<(), TestCaseError> {
    prop_assert_eq!(
        closed(lhs, word, at),
        closed(rhs, word, at),
        "closed {:?} vs {:?}",
        lhs,
        rhs
    );
    prop_assert_eq!(
        prefix(lhs, word, at),
        prefix(rhs, word, at),
        "prefix {:?} vs {:?}",
        lhs,
        rhs
    );
    Ok(())
}

// ------------------------------------------------------------------- duality

// Trace: TC-002, FR-001-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
// spec-correctness: row=FR-001-AC-1 property=universal extraction=extractable origin=regex
#[test]
fn boolean_connectives_satisfy_de_morgan_and_definitional_identities() {
    run(
        "boolean_duality",
        (future_tree(), future_tree(), word(), 0_u64..=10),
        |(x, y, word, at)| {
            let laws = [
                (
                    not(T::And(b(x.clone()), b(y.clone()))),
                    T::Or(b(not(x.clone())), b(not(y.clone()))),
                ),
                (
                    not(T::Or(b(x.clone()), b(y.clone()))),
                    T::And(b(not(x.clone())), b(not(y.clone()))),
                ),
                (not(not(x.clone())), x.clone()),
                (
                    T::Implies(b(x.clone()), b(y.clone())),
                    T::Or(b(not(x.clone())), b(y.clone())),
                ),
                (
                    T::Equivalent(b(x.clone()), b(y.clone())),
                    T::And(
                        b(T::Implies(b(x.clone()), b(y.clone()))),
                        b(T::Implies(b(y.clone()), b(x.clone()))),
                    ),
                ),
            ];
            for (lhs, rhs) in &laws {
                assert_equivalent(lhs, rhs, &word, at)?;
            }
            Ok(())
        },
    );
}

// Trace: TC-002, FR-001-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
// spec-correctness: row=FR-001-AC-1 property=universal extraction=extractable origin=regex
#[test]
fn future_operators_satisfy_their_duals() {
    run(
        "future_duality",
        (future_tree(), future_tree(), interval(), word(), 0_u64..=10),
        |(x, y, (a, c), word, at)| {
            let laws = [
                // ¬F[a,c]φ ≡ G[a,c]¬φ and the converse.
                (not(T::F(a, c, b(x.clone()))), T::G(a, c, b(not(x.clone())))),
                (not(T::G(a, c, b(x.clone()))), T::F(a, c, b(not(x.clone())))),
                // ¬(φ U[a,c] ψ) ≡ ¬φ R[a,c] ¬ψ and the converse.
                (
                    not(T::U(a, c, b(x.clone()), b(y.clone()))),
                    T::R(a, c, b(not(x.clone())), b(not(y.clone()))),
                ),
                (
                    not(T::R(a, c, b(x.clone()), b(y.clone()))),
                    T::U(a, c, b(not(x.clone())), b(not(y.clone()))),
                ),
            ];
            for (lhs, rhs) in &laws {
                assert_equivalent(lhs, rhs, &word, at)?;
            }
            Ok(())
        },
    );
}

// Trace: TC-002, FR-001-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
// spec-correctness: row=FR-001-AC-1 property=universal extraction=extractable origin=regex
#[test]
fn past_operators_satisfy_their_duals() {
    run(
        "past_duality",
        (
            past_tree(),
            past_tree(),
            interval(),
            prop::collection::vec(any::<[bool; 2]>(), 1..=10),
            0_u64..=9,
        ),
        |(x, y, (a, c), word, anchor)| {
            let anchor = anchor % word.len() as u64;
            let laws = [
                (
                    not(T::Once(a, c, b(x.clone()))),
                    T::Hist(a, c, b(not(x.clone()))),
                ),
                (
                    not(T::Hist(a, c, b(x.clone()))),
                    T::Once(a, c, b(not(x.clone()))),
                ),
                (
                    not(T::Since(a, c, b(x.clone()), b(y.clone()))),
                    T::Trig(a, c, b(not(x.clone())), b(not(y.clone()))),
                ),
                (
                    not(T::Trig(a, c, b(x.clone()), b(y.clone()))),
                    T::Since(a, c, b(not(x.clone())), b(not(y.clone()))),
                ),
            ];
            for (lhs, rhs) in &laws {
                prop_assert_eq!(
                    past(lhs, &word, anchor),
                    past(rhs, &word, anchor),
                    "{:?} vs {:?}",
                    lhs,
                    rhs
                );
            }
            Ok(())
        },
    );
}

// --------------------------------------------------------- interval split

/// Splits `[a,c]` into `[a,m]` and `[m+1,c]`, with `a <= m < c`.
fn split() -> impl Strategy<Value = (u32, u32, u32)> {
    (0_u32..=4, 1_u32..=4, any::<prop::sample::Index>()).prop_map(|(a, width, index)| {
        let c = a + width;
        (a, a + index.index(width as usize) as u32, c)
    })
}

// Trace: TC-002, FR-001-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
// spec-correctness: row=FR-001-AC-1 property=universal extraction=extractable origin=regex
#[test]
fn future_intervals_split_at_any_interior_point() {
    run(
        "interval_split",
        (future_tree(), future_tree(), split(), word(), 0_u64..=10),
        |(x, y, (a, m, c), word, at)| {
            let laws = [
                // F[a,c]φ ≡ F[a,m]φ ∨ F[m+1,c]φ
                (
                    T::F(a, c, b(x.clone())),
                    T::Or(b(T::F(a, m, b(x.clone()))), b(T::F(m + 1, c, b(x.clone())))),
                ),
                // G[a,c]φ ≡ G[a,m]φ ∧ G[m+1,c]φ
                (
                    T::G(a, c, b(x.clone())),
                    T::And(b(T::G(a, m, b(x.clone()))), b(T::G(m + 1, c, b(x.clone())))),
                ),
                // φ U[a,c] ψ ≡ φ U[a,m] ψ ∨ (G[a,m]φ ∧ φ U[m+1,c] ψ)
                (
                    T::U(a, c, b(x.clone()), b(y.clone())),
                    T::Or(
                        b(T::U(a, m, b(x.clone()), b(y.clone()))),
                        b(T::And(
                            b(T::G(a, m, b(x.clone()))),
                            b(T::U(m + 1, c, b(x.clone()), b(y.clone()))),
                        )),
                    ),
                ),
                // φ R[a,c] ψ ≡ φ R[a,m] ψ ∧ (F[a,m]φ ∨ φ R[m+1,c] ψ), the dual of the Until split.
                (
                    T::R(a, c, b(x.clone()), b(y.clone())),
                    T::And(
                        b(T::R(a, m, b(x.clone()), b(y.clone()))),
                        b(T::Or(
                            b(T::F(a, m, b(x.clone()))),
                            b(T::R(m + 1, c, b(x.clone()), b(y.clone()))),
                        )),
                    ),
                ),
            ];
            for (lhs, rhs) in &laws {
                assert_equivalent(lhs, rhs, &word, at)?;
            }
            Ok(())
        },
    );
}

// Trace: TC-002, FR-001-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
// spec-correctness: row=FR-001-AC-1 property=universal extraction=extractable origin=regex
#[test]
fn past_intervals_split_at_any_interior_point() {
    run(
        "past_interval_split",
        (
            past_tree(),
            split(),
            prop::collection::vec(any::<[bool; 2]>(), 1..=10),
            0_u64..=9,
        ),
        |(x, (a, m, c), word, anchor)| {
            let anchor = anchor % word.len() as u64;
            let laws = [
                (
                    T::Once(a, c, b(x.clone())),
                    T::Or(
                        b(T::Once(a, m, b(x.clone()))),
                        b(T::Once(m + 1, c, b(x.clone()))),
                    ),
                ),
                (
                    T::Hist(a, c, b(x.clone())),
                    T::And(
                        b(T::Hist(a, m, b(x.clone()))),
                        b(T::Hist(m + 1, c, b(x.clone()))),
                    ),
                ),
            ];
            for (lhs, rhs) in &laws {
                prop_assert_eq!(
                    past(lhs, &word, anchor),
                    past(rhs, &word, anchor),
                    "{:?} vs {:?}",
                    lhs,
                    rhs
                );
            }
            Ok(())
        },
    );
}

// ------------------------------------------------------------------- shift

// Trace: TC-002, FR-001-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
// spec-correctness: row=FR-001-AC-1 property=universal extraction=extractable origin=regex
#[test]
fn shifting_every_bound_by_k_equals_evaluating_k_steps_later() {
    run(
        "interval_shift",
        (
            future_tree(),
            future_tree(),
            interval(),
            0_u32..=4,
            word(),
            0_u64..=8,
        ),
        |(x, y, (a, c), k, word, at)| {
            let later = at + u64::from(k);
            let shifted = [
                (T::F(a + k, c + k, b(x.clone())), T::F(a, c, b(x.clone()))),
                (T::G(a + k, c + k, b(x.clone())), T::G(a, c, b(x.clone()))),
                (
                    T::U(a + k, c + k, b(x.clone()), b(y.clone())),
                    T::U(a, c, b(x.clone()), b(y.clone())),
                ),
                (
                    T::R(a + k, c + k, b(x.clone()), b(y.clone())),
                    T::R(a, c, b(x.clone()), b(y.clone())),
                ),
            ];
            for (shifted, base) in &shifted {
                prop_assert_eq!(
                    closed(shifted, &word, at),
                    closed(base, &word, later),
                    "closed {:?} at {} vs {:?} at {}",
                    shifted,
                    at,
                    base,
                    later
                );
                prop_assert_eq!(
                    prefix(shifted, &word, at),
                    prefix(base, &word, later),
                    "prefix {:?} at {} vs {:?} at {}",
                    shifted,
                    at,
                    base,
                    later
                );
            }
            Ok(())
        },
    );
}

// Trace: TC-002, FR-001-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
// spec-correctness: row=FR-001-AC-1 property=universal extraction=extractable origin=regex
#[test]
fn evaluating_at_t_equals_evaluating_the_suffix_from_t_at_zero() {
    run(
        "suffix_evaluation",
        (future_tree(), word(), 0_u64..=10),
        |(x, word, at)| {
            let from = at as usize % (word.len() + 1);
            let suffix = &word[from..];
            prop_assert_eq!(closed(&x, &word, from as u64), closed(&x, suffix, 0));
            prop_assert_eq!(prefix(&x, &word, from as u64), prefix(&x, suffix, 0));
            Ok(())
        },
    );
}

// ----------------------------------------------------- monotonicity, horizon

// Trace: TC-009, FR-003-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
#[test]
fn a_settled_prefix_verdict_survives_every_extension() {
    run(
        "suffix_monotonicity",
        (future_tree(), word(), word()),
        |(x, base, extra)| {
            let settled = prefix(&x, &base, 0);
            let mut extended = base.clone();
            for cell in &extra {
                extended.push(*cell);
                let later = prefix(&x, &extended, 0);
                if settled != TruthValue::Pending {
                    prop_assert_eq!(
                        later,
                        settled,
                        "{:?} settled on {:?} then changed after extending to {:?}",
                        x,
                        base,
                        extended
                    );
                }
            }
            // A settled prefix verdict is already the closed verdict.
            if settled != TruthValue::Pending {
                prop_assert_eq!(settled, closed(&x, &base, 0));
            }
            Ok(())
        },
    );
}

// Trace: TC-008, FR-002-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
#[test]
fn the_verdict_is_fixed_once_the_prefix_reaches_the_horizon() {
    run(
        "horizon_stability",
        (
            future_tree(),
            prop::collection::vec(any::<[bool; 2]>(), 0..=20),
            word(),
        ),
        |(x, mut word, extra)| {
            let horizon = lookahead(&x) as usize;
            // Make the prefix reach the horizon: required buffer = lookahead + 1.
            while word.len() < horizon + 1 {
                word.push([false, false]);
            }
            let verdict = prefix(&x, &word, 0);
            prop_assert_ne!(
                verdict,
                TruthValue::Pending,
                "{:?} still pending at length {} >= horizon {}",
                x,
                word.len(),
                horizon
            );
            prop_assert_eq!(verdict, closed(&x, &word, 0));
            let mut extended = word.clone();
            for cell in &extra {
                extended.push(*cell);
                prop_assert_eq!(prefix(&x, &extended, 0), verdict);
            }
            // Instants beyond the horizon do not influence the verdict.
            let truncated = &word[..horizon + 1];
            prop_assert_eq!(closed(&x, truncated, 0), verdict);
            Ok(())
        },
    );
}

// Trace: TC-008, FR-002-AC-1, TC-104, TC-107, FR-020-AC-1, FR-020-AC-4
#[test]
fn horizon_follows_the_nesting_of_interval_upper_bounds() {
    run(
        "horizon_arithmetic",
        (future_tree(), interval(), interval()),
        |(x, (a, c), (a2, c2))| {
            let base = lookahead(&x);
            // F and G add their upper bound to the operand's horizon.
            prop_assert_eq!(lookahead(&T::F(a, c, b(x.clone()))), base + u64::from(c));
            prop_assert_eq!(lookahead(&T::G(a, c, b(x.clone()))), base + u64::from(c));
            // Until and Release add their bound to the larger operand horizon;
            // the sibling here is a bare proposition, whose horizon is zero.
            prop_assert_eq!(
                lookahead(&T::U(a, c, b(x.clone()), b(T::P(0)))),
                base + u64::from(c)
            );
            prop_assert_eq!(
                lookahead(&T::R(a, c, b(T::P(0)), b(x.clone()))),
                base + u64::from(c)
            );
            // Nesting accumulates.
            prop_assert_eq!(
                lookahead(&T::F(a2, c2, b(T::G(a, c, b(x))))),
                base + u64::from(c) + u64::from(c2)
            );
            Ok(())
        },
    );
}
