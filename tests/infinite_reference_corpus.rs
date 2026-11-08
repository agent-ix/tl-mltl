//! TL-221 / TL-238: the infinite-trace provider against two independent
//! reference procedures on a counted lasso corpus, and the rule that finite
//! evidence never settles a liveness claim as satisfied.
//!
//! References, both in the dev-only `tl-oracle` crate and sharing no code with
//! the provider or with each other:
//! * the offset-scanning evaluator (`tl_oracle::evaluate_documents`);
//! * the successor-fixpoint procedure (`tl_oracle::fixpoint`).
//!
//! The corpus is a closed, enumerated population. `PopulationLedger` counts
//! every declared case exactly once, so a skipped or duplicated case fails the
//! run rather than shrinking the corpus silently.

#![cfg(feature = "infinite-trace")]

#[path = "support/lasso_kit.rs"]
mod kit;

use kit::*;
use tl_mltl::infinite::{
    evaluate_prefix_safety, Disposition, EvaluationLimit, EvidenceClosure, PrefixRequest,
    TruthAvailability,
};
use tl_oracle::population::{CaseResult, PopulationLedger};

fn all_intervals() -> Vec<Iv> {
    vec![
        Iv::Closed(0, 0),
        Iv::Closed(0, 2),
        Iv::Closed(1, 3),
        Iv::Open(0),
        Iv::Open(2),
    ]
}

/// Every Boolean connective and temporal operator over the leaf set, with every
/// interval shape: the single-operator layer of the infinite profile.
fn single_operator_formulas() -> Vec<T> {
    let leaves = [T::P(0), T::P(1), not(T::P(0))];
    let mut out = Vec::new();
    for x in &leaves {
        out.push(not(x.clone()));
        out.push(T::Y(bx(x.clone())));
        for i in all_intervals() {
            out.push(T::F(i, bx(x.clone())));
            out.push(T::G(i, bx(x.clone())));
            out.push(T::Once(i, bx(x.clone())));
            out.push(T::Hist(i, bx(x.clone())));
        }
        for y in &leaves {
            out.push(T::And(bx(x.clone()), bx(y.clone())));
            out.push(T::Or(bx(x.clone()), bx(y.clone())));
            out.push(T::Implies(bx(x.clone()), bx(y.clone())));
            out.push(T::Equiv(bx(x.clone()), bx(y.clone())));
            for i in all_intervals() {
                out.push(T::U(i, bx(x.clone()), bx(y.clone())));
                out.push(T::R(i, bx(x.clone()), bx(y.clone())));
                out.push(T::Since(i, bx(x.clone()), bx(y.clone())));
                out.push(T::Trig(i, bx(x.clone()), bx(y.clone())));
            }
        }
    }
    out
}

/// Nesting layer: temporal operators over single-operator children, including
/// future-over-past and past-over-future, and the liveness shapes `GF`, `FG`.
fn nested_formulas() -> Vec<T> {
    let children: Vec<T> = single_operator_formulas().into_iter().step_by(31).collect();
    let mut out = vec![
        T::G(Iv::Open(0), bx(T::F(Iv::Open(0), bx(T::P(0))))),
        T::F(Iv::Open(0), bx(T::G(Iv::Open(0), bx(T::P(0))))),
        T::G(
            Iv::Open(0),
            bx(T::Implies(bx(T::P(0)), bx(T::F(Iv::Open(1), bx(T::P(1)))))),
        ),
        T::U(Iv::Open(0), bx(T::P(0)), bx(T::G(Iv::Open(0), bx(T::P(1))))),
    ];
    for c in &children {
        for i in [Iv::Closed(0, 1), Iv::Open(0), Iv::Open(1)] {
            out.push(T::F(i, bx(c.clone())));
            out.push(T::G(i, bx(c.clone())));
            out.push(T::Once(i, bx(c.clone())));
            out.push(T::Hist(i, bx(c.clone())));
            out.push(T::U(i, bx(c.clone()), bx(T::P(1))));
            out.push(T::Since(i, bx(T::P(0)), bx(c.clone())));
        }
    }
    out
}

fn complete_cells() -> Vec<Cell> {
    let mut v = Vec::new();
    for a in [F, Tr] {
        for b in [F, Tr] {
            v.push([a, b]);
        }
    }
    v
}

/// Complete lassos over two propositions, prefix 0..=1 and loop 1..=2.
fn complete_lassos() -> Vec<(Vec<Cell>, Vec<Cell>)> {
    let cs = complete_cells();
    let mut out = Vec::new();
    for pre in 0..=1usize {
        for lp in 1..=2usize {
            let total = pre + lp;
            for code in 0..cs.len().pow(total as u32) {
                let mut rest = code;
                let picked: Vec<Cell> = (0..total)
                    .map(|_| {
                        let c = cs[rest % cs.len()];
                        rest /= cs.len();
                        c
                    })
                    .collect();
                out.push((picked[..pre].to_vec(), picked[pre..].to_vec()));
            }
        }
    }
    out
}

/// A fixed set of partial lassos: missing, conflicting and mixed cells in the
/// prefix and in the loop.
fn partial_lassos() -> Vec<(Vec<Cell>, Vec<Cell>)> {
    vec![
        (vec![], vec![[M, F]]),
        (vec![], vec![[C, Tr]]),
        (vec![], vec![[M, C]]),
        (vec![[M, Tr]], vec![[Tr, C], [F, F]]),
        (vec![[F, F]], vec![[M, Tr], [F, M]]),
        (vec![[C, M], [Tr, Tr]], vec![[F, Tr]]),
        (vec![], vec![[Tr, Tr], [M, M]]),
        (vec![[Tr, F]], vec![[C, C], [Tr, F]]),
    ]
}

struct Tally {
    declared: u64,
    ledger: PopulationLedger,
    next: u64,
}

impl Tally {
    fn new(declared: u64) -> Self {
        Self {
            declared,
            ledger: PopulationLedger::new(declared).unwrap(),
            next: 0,
        }
    }

    /// Checks one case: provider against both references, and the references
    /// against each other. Panics with the case on any disagreement.
    fn check(&mut self, what: &str, built: &Built, word: &tl_syntax::LassoTraceDocument, at: u64) {
        let actual = provider(built, word, at);
        let fix = fixpoint(built, word, at);
        let scan = scanning(built, word, at);
        assert_eq!(fix, scan, "references disagree: {what} at {at}");
        if let Some(why) = disagreement(&actual, &fix) {
            panic!(
                "provider vs references: {why}\n{what} at {at}\n{:?}",
                cells(word)
            );
        }
        let observed = match &fix {
            Ok(out) => CaseResult::Admitted(out.verdict as u8),
            Err(_) => CaseResult::Refused,
        };
        self.ledger.record(self.next, observed, observed).unwrap();
        self.next += 1;
    }

    fn finish(self) -> tl_oracle::population::PopulationSummary {
        assert_eq!(
            self.next, self.declared,
            "declared population fully visited"
        );
        self.ledger.finish().unwrap()
    }
}

// Trace: TC-141, TC-145, TC-146, TC-147, TC-148, TC-151, FR-030-AC-1, FR-031-AC-1, FR-031-AC-2, FR-032-AC-1
#[test]
fn provider_agrees_with_both_references_on_the_complete_lasso_corpus() {
    let formulas = single_operator_formulas();
    let words = complete_lassos();
    let positions = 0..4u64;
    let declared = (formulas.len() * words.len() * positions.clone().count()) as u64;
    // 3 leaves: 3*(2+4*5) unary + 9*(4+4*5) binary; lassos: sum over prefix
    // 0..=1, loop 1..=2 of 4^(prefix+loop); four positions each.
    assert_eq!(formulas.len(), 3 * (2 + 4 * 5) + 9 * (4 + 4 * 5));
    assert_eq!(words.len(), 4 + 16 + 16 + 64);
    let mut tally = Tally::new(declared);
    for (f, formula) in formulas.iter().enumerate() {
        let built = build(formula, &[]);
        for (w, (prefix, tail)) in words.iter().enumerate() {
            let word = lasso(prefix, tail);
            for at in positions.clone() {
                tally.check(
                    &format!("formula#{f} {formula:?} word#{w}"),
                    &built,
                    &word,
                    at,
                );
            }
        }
    }
    let summary = tally.finish();
    assert_eq!(summary.declared, declared);
    assert_eq!(summary.admitted, declared);
    assert_eq!(summary.refused, 0);
}

// Trace: TC-141, TC-148, TC-150, FR-031-AC-2, FR-032-AC-1
#[test]
fn provider_agrees_with_both_references_on_nested_formulas() {
    let formulas = nested_formulas();
    let words: Vec<_> = complete_lassos().into_iter().step_by(7).collect();
    let positions = [0u64, 1, 2, 5];
    let declared = (formulas.len() * words.len() * positions.len()) as u64;
    let mut tally = Tally::new(declared);
    for (f, formula) in formulas.iter().enumerate() {
        let built = build(formula, &[]);
        for (w, (prefix, tail)) in words.iter().enumerate() {
            let word = lasso(prefix, tail);
            for at in positions {
                tally.check(
                    &format!("nested#{f} {formula:?} word#{w}"),
                    &built,
                    &word,
                    at,
                );
            }
        }
    }
    let summary = tally.finish();
    assert_eq!(summary.declared, declared);
    assert_eq!(summary.refused, 0);
}

// Trace: TC-142, TC-143, TC-144, TC-152, TC-153, FR-030-AC-1, FR-030-AC-2, FR-032-AC-2
#[test]
fn provider_agrees_on_partial_evidence_with_and_without_fairness() {
    let claims: Vec<T> = single_operator_formulas().into_iter().step_by(5).collect();
    let fairness_sets: Vec<Vec<T>> = vec![
        vec![],
        vec![T::P(0)],
        vec![T::P(0), not(T::P(1))],
        vec![T::F(Iv::Open(0), bx(T::P(1)))],
    ];
    let words = partial_lassos();
    let positions = [0u64, 1, 3];
    let declared = (claims.len() * fairness_sets.len() * words.len() * positions.len()) as u64;
    let mut tally = Tally::new(declared);
    for (c, claim) in claims.iter().enumerate() {
        for (s, fairness) in fairness_sets.iter().enumerate() {
            let built = build(claim, fairness);
            for (w, (prefix, tail)) in words.iter().enumerate() {
                let word = lasso(prefix, tail);
                for at in positions {
                    tally.check(
                        &format!("claim#{c} {claim:?} fairness#{s} word#{w}"),
                        &built,
                        &word,
                        at,
                    );
                }
            }
        }
    }
    let summary = tally.finish();
    assert_eq!(summary.declared, declared);
    // Empty fair admissions are classified Refused by the references; they
    // are still compared (the provider must report empty fair admission).
    assert_eq!(summary.admitted + summary.refused, declared);
}

fn liveness_formulas() -> Vec<T> {
    vec![
        T::F(Iv::Open(0), bx(T::P(0))),
        T::F(Iv::Open(3), bx(T::P(0))),
        T::G(Iv::Open(0), bx(T::F(Iv::Open(0), bx(T::P(0))))),
        T::F(Iv::Open(0), bx(T::G(Iv::Open(0), bx(T::P(0))))),
        T::U(Iv::Open(0), bx(T::P(0)), bx(T::P(1))),
        T::U(Iv::Open(2), bx(T::P(0)), bx(T::P(1))),
        T::R(Iv::Open(0), bx(T::P(0)), bx(T::P(1))),
        T::G(Iv::Open(0), bx(T::P(0))),
        T::G(
            Iv::Open(0),
            bx(T::Implies(
                bx(T::P(0)),
                bx(T::F(Iv::Closed(0, 2), bx(T::P(1)))),
            )),
        ),
        T::G(Iv::Open(1), bx(T::P(0))),
        T::Hist(Iv::Open(0), bx(T::P(0))),
        T::And(bx(T::P(0)), bx(T::F(Iv::Open(0), bx(T::P(1))))),
    ]
}

fn finite_prefixes() -> Vec<Vec<Cell>> {
    let mut out = Vec::new();
    for len in 1..=3usize {
        let cs = complete_cells();
        for code in 0..cs.len().pow(len as u32) {
            let mut rest = code;
            out.push(
                (0..len)
                    .map(|_| {
                        let c = cs[rest % cs.len()];
                        rest /= cs.len();
                        c
                    })
                    .collect(),
            );
        }
    }
    out.push(vec![[M, Tr], [Tr, C]]);
    out.push(vec![[M, M]]);
    out
}

fn prefix_result(
    built: &Built,
    prefix: &[Cell],
    limit: EvaluationLimit,
) -> Result<tl_mltl::infinite::InfiniteResult, tl_mltl::infinite::InfiniteError> {
    let word = lasso(prefix, &[[F, F]]);
    let graph_id = built.doc.content_identity().unwrap();
    evaluate_prefix_safety(&PrefixRequest {
        formula: &built.doc,
        graph_id: &graph_id,
        proposition_map_id: "map",
        propositions: word.propositions(),
        observations: word.prefix(),
        limit,
    })
}

// Trace: TC-089, TC-140, TC-156, TC-157, FR-029-AC-2, FR-033-AC-1, FR-033-AC-2
#[test]
fn finite_evidence_never_settles_a_liveness_claim_as_satisfied() {
    let mut checked = 0u64;
    let mut refuted = 0u64;
    for formula in liveness_formulas() {
        let built = build(&formula, &[]);
        for prefix in finite_prefixes() {
            for limit in [
                EvaluationLimit::default(),
                EvaluationLimit {
                    max_steps: 1,
                    ..EvaluationLimit::default()
                },
                EvaluationLimit {
                    max_positions: 1,
                    ..EvaluationLimit::default()
                },
                EvaluationLimit {
                    deadline: Some(std::time::Instant::now()),
                    ..EvaluationLimit::default()
                },
            ] {
                checked += 1;
                // Unsupported shapes refuse with a typed error: not a verdict.
                let Ok(result) = prefix_result(&built, &prefix, limit) else {
                    continue;
                };
                assert_ne!(
                    result.disposition,
                    Disposition::Proved,
                    "{formula:?} on prefix {prefix:?}"
                );
                assert_ne!(
                    result.truth,
                    TruthAvailability::True,
                    "{formula:?} on prefix {prefix:?}"
                );
                if result.disposition == Disposition::Refuted {
                    // Only a bad-prefix refutation of a G[0,) safety claim is
                    // admitted, and it must hold on every continuation.
                    refuted += 1;
                    for tail in [[F, F], [Tr, Tr], [Tr, F], [F, Tr]] {
                        let word = lasso(&prefix, &[tail]);
                        let exact = provider(&built, &word, 0);
                        assert_eq!(exact.disposition, Disposition::Refuted);
                    }
                }
            }
        }
    }
    assert_eq!(checked, 12 * (4 + 16 + 64 + 2) as u64 * 4);
    assert!(refuted > 0, "the safety claims must produce refutations");
}

// Trace: TC-139, TC-158, TC-159, FR-033-AC-3, FR-034-AC-2
#[test]
fn an_exhausted_budget_or_expired_deadline_on_a_lasso_never_settles_satisfied() {
    let word = lasso(&[[Tr, Tr]], &[[Tr, Tr], [M, Tr]]);
    for formula in liveness_formulas() {
        let built = build(&formula, &[]);
        for limit in [
            EvaluationLimit {
                max_steps: 1,
                ..EvaluationLimit::default()
            },
            EvaluationLimit {
                max_states: 1,
                ..EvaluationLimit::default()
            },
            EvaluationLimit {
                max_completions: 1,
                ..EvaluationLimit::default()
            },
            EvaluationLimit {
                max_positions: 1,
                ..EvaluationLimit::default()
            },
            EvaluationLimit {
                deadline: Some(std::time::Instant::now()),
                ..EvaluationLimit::default()
            },
        ] {
            let result = provider_with(&built, &word, 0, EvidenceClosure::Closed, limit);
            assert_ne!(
                result.disposition,
                Disposition::Proved,
                "{formula:?} {limit:?}"
            );
            assert_ne!(
                result.truth,
                TruthAvailability::True,
                "{formula:?} {limit:?}"
            );
        }
    }
    // Not vacuous: the same traces prove under the default budget.
    let proving = build(&T::F(Iv::Open(0), bx(T::P(0))), &[]);
    assert_eq!(
        provider(&proving, &word, 0).disposition,
        Disposition::Proved
    );
}
