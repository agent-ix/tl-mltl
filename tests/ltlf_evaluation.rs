//! TL-428: LTLf evaluation (FR-083), test-only reference agreement (FR-084-AC-4)
//! and the LTL3 verdict mapping (FR-085).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! UNBUILT-PIN: LTLf node kinds and profile are tl-syntax ADR-044 (PROPOSED).
//! The 18 rule cases and 2 counterexamples are written here by hand from the
//! ADR-044 rules because the tl-syntax `ltlf` corpus (FR-335) is not merged;
//! replace with `tl_syntax::CORPUS_DIR` once it is.

use tl_mltl::ltlf::{
    evaluate_ltlf, evaluate_ltlf_with_catalog, ltlf_charge, Ltl3Verdict, LtlfError, LtlfLimits,
};
use tl_mltl::record::RecordVerdict;
use tl_mltl::{
    analyze_horizon, evaluate_closed_at, evaluate_prefix_at, EvaluationError, EvaluationLimits,
    HorizonError, TruthValue,
};
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};

// --------------------------------------------------------- test-only tree

#[derive(Clone, Debug)]
enum L {
    T,
    F,
    P(u32),
    Not(Box<L>),
    And(Box<L>, Box<L>),
    Or(Box<L>, Box<L>),
    Imp(Box<L>, Box<L>),
    Eqv(Box<L>, Box<L>),
    Xs(Box<L>),
    Xw(Box<L>),
    Fut(Box<L>),
    Glob(Box<L>),
    U(Box<L>, Box<L>),
    R(Box<L>, Box<L>),
}

fn b(l: &L) -> Box<L> {
    Box::new(l.clone())
}

fn lower(f: &L, nodes: &mut Vec<Node>) -> NodeId {
    let kind = match f {
        L::T => NodeKind::True,
        L::F => NodeKind::False,
        L::P(n) => NodeKind::Proposition {
            proposition: PropositionId(*n),
        },
        L::Not(a) => NodeKind::Not {
            operand: lower(a, nodes),
        },
        L::Xs(a) => NodeKind::StrongNext {
            operand: lower(a, nodes),
        },
        L::Xw(a) => NodeKind::WeakNext {
            operand: lower(a, nodes),
        },
        L::Fut(a) => NodeKind::LtlfFuture {
            operand: lower(a, nodes),
        },
        L::Glob(a) => NodeKind::LtlfGlobally {
            operand: lower(a, nodes),
        },
        L::And(a, c) | L::Or(a, c) | L::Imp(a, c) | L::Eqv(a, c) | L::U(a, c) | L::R(a, c) => {
            let left = lower(a, nodes);
            let right = lower(c, nodes);
            match f {
                L::And(..) => NodeKind::And { left, right },
                L::Or(..) => NodeKind::Or { left, right },
                L::Imp(..) => NodeKind::Implies { left, right },
                L::Eqv(..) => NodeKind::Equivalent { left, right },
                L::U(..) => NodeKind::LtlfUntil { left, right },
                _ => NodeKind::LtlfRelease { left, right },
            }
        }
    };
    nodes.push(Node::new(kind));
    NodeId((nodes.len() - 1) as u32)
}

fn production(f: &L, trace: &[Vec<PropositionId>]) -> bool {
    let mut nodes = Vec::new();
    let root = lower(f, &mut nodes);
    let formula = Formula::new(SemanticProfile::LtlfFiniteTraceV1, root, &nodes).unwrap();
    evaluate_ltlf(formula, trace, LtlfLimits::default()).unwrap()
}

/// Direct definition-table evaluator (ADR-044 item 3), sharing no code with
/// production. `flip_strong_next_at_end` is the mutation control.
fn direct(f: &L, trace: &[u8], i: usize, flip_strong_next_at_end: bool) -> bool {
    let last = trace.len() - 1;
    let d = |g: &L, j: usize| direct(g, trace, j, flip_strong_next_at_end);
    match f {
        L::T => true,
        L::F => false,
        L::P(n) => trace[i] >> n & 1 == 1,
        L::Not(a) => !d(a, i),
        L::And(a, c) => d(a, i) && d(c, i),
        L::Or(a, c) => d(a, i) || d(c, i),
        L::Imp(a, c) => !d(a, i) || d(c, i),
        L::Eqv(a, c) => d(a, i) == d(c, i),
        L::Xs(a) => {
            if i == last {
                flip_strong_next_at_end
            } else {
                d(a, i + 1)
            }
        }
        L::Xw(a) => i == last || d(a, i + 1),
        L::Fut(a) => (i..=last).any(|j| d(a, j)),
        L::Glob(a) => (i..=last).all(|j| d(a, j)),
        L::U(a, c) => (i..=last).any(|j| d(c, j) && (i..j).all(|k| d(a, k))),
        L::R(a, c) => (i..=last).all(|j| d(c, j) || (i..j).any(|k| d(a, k))),
    }
}

fn base() -> Vec<L> {
    vec![L::P(0), L::P(1), L::T, L::F]
}

/// 4 + 5 x 4 + 6 x 16 = 120 depth-one formulas.
fn depth_one() -> Vec<L> {
    let mut all = base();
    for a in &base() {
        all.push(L::Not(b(a)));
        all.push(L::Xs(b(a)));
        all.push(L::Xw(b(a)));
        all.push(L::Fut(b(a)));
        all.push(L::Glob(b(a)));
    }
    for a in &base() {
        for c in &base() {
            all.push(L::And(b(a), b(c)));
            all.push(L::Or(b(a), b(c)));
            all.push(L::Imp(b(a), b(c)));
            all.push(L::Eqv(b(a), b(c)));
            all.push(L::U(b(a), b(c)));
            all.push(L::R(b(a), b(c)));
        }
    }
    all
}

/// 4 + 16 + 64 + 256 = 340 traces of length 1..=4 over two atoms (letters 0..4).
fn traces() -> Vec<Vec<u8>> {
    let mut all = Vec::new();
    for length in 1..=4u32 {
        for code in 0..4usize.pow(length) {
            let mut rest = code;
            let mut trace = Vec::new();
            for _ in 0..length {
                trace.push((rest % 4) as u8);
                rest /= 4;
            }
            all.push(trace);
        }
    }
    all
}

fn letters(trace: &[u8]) -> Vec<Vec<PropositionId>> {
    trace
        .iter()
        .map(|letter| {
            (0..2u32)
                .filter(|n| letter >> n & 1 == 1)
                .map(PropositionId)
                .collect()
        })
        .collect()
}

const P: u8 = 0b01;
const Q: u8 = 0b10;

/// The 18 rule cases and the 2 counterexamples: (formula, trace, verdict).
fn rule_cases() -> Vec<(L, Vec<u8>, bool)> {
    let p = || Box::new(L::P(0));
    let q = || Box::new(L::P(1));
    vec![
        (L::Xs(p()), vec![P], false),
        (L::Xs(p()), vec![0, P], true),
        (L::Xs(p()), vec![P, 0], false),
        (L::Xw(p()), vec![0], true),
        (L::Xw(p()), vec![0, P], true),
        (L::Xw(p()), vec![P, 0], false),
        (L::Fut(p()), vec![P], true),
        (L::Fut(p()), vec![0, P], true),
        (L::Fut(p()), vec![0, 0], false),
        (L::Glob(p()), vec![P], true),
        (L::Glob(p()), vec![P, P], true),
        (L::Glob(p()), vec![P, 0], false),
        (L::U(p(), q()), vec![Q], true),
        (L::U(p(), q()), vec![P, Q], true),
        (L::U(p(), q()), vec![P, P], false),
        (L::R(p(), q()), vec![Q], true),
        (L::R(p(), q()), vec![Q, Q], true),
        (L::R(p(), q()), vec![Q, 0], false),
        // CE-1 `G Xs true` is false on every nonempty trace.
        (L::Glob(Box::new(L::Xs(Box::new(L::T)))), vec![P, Q, P], false),
        // CE-2 `F Xw false` is true on every nonempty trace.
        (L::Fut(Box::new(L::Xw(Box::new(L::F)))), vec![0, P], true),
    ]
}

// ------------------------------------------------------------------ tests

// Trace: TC-300, FR-083-AC-1
#[test]
fn rule_cases_and_counterexamples_return_their_verdicts() {
    let cases = rule_cases();
    assert_eq!(cases.len(), 20);
    for (formula, trace, expected) in &cases {
        assert_eq!(production(formula, &letters(trace)), *expected, "{formula:?} {trace:?}");
    }
    // The counterexamples hold on every trace of the population.
    for trace in traces() {
        assert!(!production(&cases[18].0, &letters(&trace)));
        assert!(production(&cases[19].0, &letters(&trace)));
    }
}

fn catalog(bound: &[u32]) -> SignalCatalogDocument {
    SignalCatalogDocument::new(
        bound
            .iter()
            .map(|n| {
                OwnedSignalDeclaration::new(SignalId(*n), format!("s{n}"), SignalDomain::Boolean)
            })
            .collect(),
        bound
            .iter()
            .map(|n| PropositionBinding::new(PropositionId(*n), SignalId(*n)))
            .collect(),
    )
    .unwrap()
}

// Trace: TC-301, FR-083-AC-2
#[test]
fn empty_trace_foreign_profile_unbound_signal_and_cross_profile_calls_refuse() {
    let mut nodes = Vec::new();
    let root = lower(&L::Fut(Box::new(L::P(0))), &mut nodes);
    let ltlf = Formula::new(SemanticProfile::LtlfFiniteTraceV1, root, &nodes).unwrap();
    assert_eq!(
        evaluate_ltlf(ltlf, &[], LtlfLimits::default()),
        Err(LtlfError::EmptyTrace)
    );
    let bounded_nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(0),
        }),
    ];
    let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(1), &bounded_nodes).unwrap();
    assert!(matches!(
        evaluate_ltlf(closed, &[vec![]], LtlfLimits::default()),
        Err(LtlfError::ProfileMismatch { .. })
    ));
    assert!(matches!(
        evaluate_ltlf_with_catalog(ltlf, &[vec![]], &catalog(&[1]), LtlfLimits::default()),
        Err(LtlfError::UnboundSignal(_))
    ));
    assert_eq!(
        evaluate_ltlf_with_catalog(
            ltlf,
            &[vec![PropositionId(0)]],
            &catalog(&[0]),
            LtlfLimits::default()
        ),
        Ok(true)
    );
    assert!(matches!(
        evaluate_closed_at(ltlf, "f", &[vec![]], "t", 0, EvaluationLimits::default()),
        Err(EvaluationError::UnsupportedProfile { .. })
    ));
    assert!(matches!(
        evaluate_prefix_at(ltlf, "f", &[vec![]], "t", false, 0, EvaluationLimits::default()),
        Err(EvaluationError::UnsupportedProfile { .. })
    ));
}

// Trace: TC-302, FR-083-AC-3
#[test]
fn next_globally_and_release_duality_laws_hold_on_the_population() {
    let not = |f: &L| L::Not(b(f));
    let population = depth_one();
    assert_eq!(population.len(), 120);
    let all = traces();
    assert_eq!(all.len(), 340);
    for f in &population {
        for trace in &all {
            let t = letters(trace);
            assert_eq!(
                production(&L::Xw(b(f)), &t),
                production(&not(&L::Xs(Box::new(not(f)))), &t)
            );
            assert_eq!(
                production(&L::Glob(b(f)), &t),
                production(&not(&L::Fut(Box::new(not(f)))), &t)
            );
            for g in base() {
                assert_eq!(
                    production(&L::R(b(f), b(&g)), &t),
                    production(&not(&L::U(Box::new(not(f)), Box::new(not(&g)))), &t)
                );
            }
        }
    }
}

// Trace: TC-303, FR-083-AC-4
#[test]
fn horizon_is_typed_unbounded_and_work_limit_edge_is_exact() {
    let mut nodes = Vec::new();
    let root = lower(&L::U(Box::new(L::P(0)), Box::new(L::Xs(Box::new(L::P(1))))), &mut nodes);
    let formula = Formula::new(SemanticProfile::LtlfFiniteTraceV1, root, &nodes).unwrap();
    assert_eq!(
        analyze_horizon(formula, "f"),
        Err(HorizonError::UnboundedByTraceLength)
    );
    let trace = letters(&[P, Q, P]);
    let charge = ltlf_charge(formula, trace.len());
    assert_eq!(charge, (nodes.len() * 3) as u128);
    let exact = LtlfLimits {
        max_work: charge as u64,
    };
    assert!(evaluate_ltlf(formula, &trace, exact).is_ok());
    let below = LtlfLimits {
        max_work: charge as u64 - 1,
    };
    assert_eq!(
        evaluate_ltlf(formula, &trace, below),
        Err(LtlfError::WorkLimitExceeded {
            charge,
            limit: charge as u64 - 1
        })
    );
}

// Trace: TC-307, FR-084-AC-4
#[test]
fn direct_reference_agrees_on_40820_rows_and_detects_a_flipped_rule() {
    let mut rows = 0usize;
    let mut flipped_disagreements = 0usize;
    for f in depth_one() {
        for trace in traces() {
            let actual = production(&f, &letters(&trace));
            assert_eq!(actual, direct(&f, &trace, 0, false), "{f:?} {trace:?}");
            if actual != direct(&f, &trace, 0, true) {
                flipped_disagreements += 1;
            }
            rows += 1;
        }
    }
    for (f, trace, _) in rule_cases() {
        assert_eq!(production(&f, &letters(&trace)), direct(&f, &trace, 0, false));
        rows += 1;
    }
    assert_eq!(rows, 40_820);
    assert!(flipped_disagreements > 0, "the mutation control must be caught");
}

// Trace: TC-309, FR-085-AC-1, FR-085-AC-2
#[test]
fn every_verdict_name_maps_to_ltl3_and_serialized_names_are_unchanged() {
    // Exhaustive matches: adding a variant breaks compilation here.
    for value in [TruthValue::True, TruthValue::False, TruthValue::Pending] {
        let expected = match value {
            TruthValue::True => Ltl3Verdict::True,
            TruthValue::False => Ltl3Verdict::False,
            TruthValue::Pending => Ltl3Verdict::Inconclusive,
        };
        assert_eq!(value.ltl3(), Some(expected));
    }
    for value in [RecordVerdict::True, RecordVerdict::False, RecordVerdict::Pending] {
        let expected = match value {
            RecordVerdict::True => Ltl3Verdict::True,
            RecordVerdict::False => Ltl3Verdict::False,
            RecordVerdict::Pending => Ltl3Verdict::Inconclusive,
        };
        assert_eq!(value.ltl3(), Some(expected));
    }
    assert_eq!(serde_json::to_string(&TruthValue::Pending).unwrap(), "\"pending\"");
    assert_eq!(serde_json::to_string(&TruthValue::True).unwrap(), "\"true\"");
    assert_eq!(serde_json::to_string(&RecordVerdict::Pending).unwrap(), "\"Pending\"");

    #[cfg(feature = "infinite-trace")]
    {
        use tl_mltl::infinite::Disposition;
        for value in [
            Disposition::Proved,
            Disposition::Refuted,
            Disposition::Inconclusive,
            Disposition::Unsupported,
            Disposition::Failed,
        ] {
            let expected = match value {
                Disposition::Proved => Some(Ltl3Verdict::True),
                Disposition::Refuted => Some(Ltl3Verdict::False),
                Disposition::Inconclusive => Some(Ltl3Verdict::Inconclusive),
                Disposition::Unsupported | Disposition::Failed => None,
            };
            assert_eq!(value.ltl3(), expected);
        }
        assert_eq!(
            serde_json::to_string(&Disposition::Inconclusive).unwrap(),
            "\"inconclusive\""
        );
    }
}
