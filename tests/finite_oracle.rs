//! TL-35: production finite evaluators against the independent `tl-oracle`.
//!
//! The expected answer for every case comes from `tl-oracle`, which shares only
//! `tl-syntax` input types with production. The harness records each case once
//! in an exact-count ledger; seeded production faults must turn it red.

use std::collections::BTreeMap;

use tl_mltl::{
    evaluate_closed_at, evaluate_past, ClockBinding, EvaluationLimits, PastEvaluationLimits,
    PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TruthValue,
};
use tl_oracle::population::{CaseResult, PopulationError, PopulationLedger, PopulationSummary};
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as O, Interval as OI, Limits,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

/// A cell holds the truth of propositions 0 and 1. The oracle pins its own
/// `tl-syntax` revision, so proposition identities cross the seam as integers.
type Cell = [bool; 2];
type Word = Vec<Cell>;

fn oracle_atom(n: u32) -> O {
    O::Atom(serde_json::from_value(serde_json::json!(n)).expect("oracle proposition id"))
}

fn atom_number(p: &impl serde::Serialize) -> u32 {
    serde_json::to_value(p)
        .unwrap()
        .as_u64()
        .expect("numeric proposition id") as u32
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Profile {
    ClosedFuture,
    OriginPast,
}

fn intervals() -> Vec<(usize, usize)> {
    let mut values = Vec::new();
    for end in 0..=2 {
        for start in 0..=end {
            values.push((start, end));
        }
    }
    values
}

fn oi((start, end): (usize, usize)) -> OI {
    OI::Closed { start, end }
}

fn leaves() -> Vec<O> {
    vec![O::False, O::True, oracle_atom(0), oracle_atom(1)]
}

/// Every depth-one formula over four leaves and closed intervals within 0..=2.
fn formulas() -> Vec<O> {
    let leaves = leaves();
    let mut out = leaves.clone();
    for p in &leaves {
        out.push(O::Not(Box::new(p.clone())));
        out.push(O::StrongPrevious(Box::new(p.clone())));
        for r in intervals() {
            let r = oi(r);
            out.extend([
                O::Future(r, Box::new(p.clone())),
                O::Globally(r, Box::new(p.clone())),
                O::Once(r, Box::new(p.clone())),
                O::Historically(r, Box::new(p.clone())),
            ]);
        }
    }
    for p in &leaves {
        for q in &leaves {
            let (a, b) = (Box::new(p.clone()), Box::new(q.clone()));
            out.extend([
                O::And(a.clone(), b.clone()),
                O::Or(a.clone(), b.clone()),
                O::Implies(a.clone(), b.clone()),
                O::Equivalent(a.clone(), b.clone()),
            ]);
            for r in intervals() {
                let r = oi(r);
                out.extend([
                    O::Until(r, a.clone(), b.clone()),
                    O::Release(r, a.clone(), b.clone()),
                    O::Since(r, a.clone(), b.clone()),
                    O::Triggered(r, a.clone(), b.clone()),
                ]);
            }
        }
    }
    out
}

fn words() -> Vec<Word> {
    let cells: Vec<Cell> = (0..4).map(|bits| [bits & 1 != 0, bits & 2 != 0]).collect();
    let mut words: Vec<Word> = Vec::new();
    let mut frontier: Vec<Word> = vec![Vec::new()];
    for _ in 0..3 {
        let mut next = Vec::new();
        for prefix in &frontier {
            for cell in &cells {
                let mut word = prefix.clone();
                word.push(*cell);
                next.push(word);
            }
        }
        words.extend(next.iter().cloned());
        frontier = next;
    }
    words
}

/// Lowers the oracle tree to a production `tl-syntax` node arena.
fn lower(formula: &O, nodes: &mut Vec<Node>) -> NodeId {
    let iv = |i: &OI| match i {
        OI::Closed { start, end } => {
            Interval::new(*start as u32, *end as u32).expect("closed interval")
        }
        OI::Unbounded { .. } => unreachable!("finite population has no open interval"),
    };
    let push = |nodes: &mut Vec<Node>, kind| {
        nodes.push(Node::new(kind));
        NodeId((nodes.len() - 1) as u32)
    };
    match formula {
        O::False => push(nodes, NodeKind::False),
        O::True => push(nodes, NodeKind::True),
        O::Atom(p) => push(
            nodes,
            NodeKind::Proposition {
                proposition: PropositionId(atom_number(p)),
            },
        ),
        O::Not(a) => {
            let operand = lower(a, nodes);
            push(nodes, NodeKind::Not { operand })
        }
        O::StrongPrevious(a) => {
            let operand = lower(a, nodes);
            push(nodes, NodeKind::StrongPrevious { operand })
        }
        O::And(a, b) | O::Or(a, b) | O::Implies(a, b) | O::Equivalent(a, b) => {
            let (left, right) = (lower(a, nodes), lower(b, nodes));
            let kind = match formula {
                O::And(..) => NodeKind::And { left, right },
                O::Or(..) => NodeKind::Or { left, right },
                O::Implies(..) => NodeKind::Implies { left, right },
                _ => NodeKind::Equivalent { left, right },
            };
            push(nodes, kind)
        }
        O::Future(i, a) | O::Globally(i, a) | O::Once(i, a) | O::Historically(i, a) => {
            let operand = lower(a, nodes);
            let interval = iv(i);
            let kind = match formula {
                O::Future(..) => NodeKind::Future { interval, operand },
                O::Globally(..) => NodeKind::Globally { interval, operand },
                O::Once(..) => NodeKind::Once { interval, operand },
                _ => NodeKind::Historically { interval, operand },
            };
            push(nodes, kind)
        }
        O::Until(i, a, b) | O::Release(i, a, b) | O::Since(i, a, b) | O::Triggered(i, a, b) => {
            let (left, right) = (lower(a, nodes), lower(b, nodes));
            let interval = iv(i);
            let kind = match formula {
                O::Until(..) => NodeKind::Until {
                    interval,
                    left,
                    right,
                },
                O::Release(..) => NodeKind::Release {
                    interval,
                    left,
                    right,
                },
                O::Since(..) => NodeKind::Since {
                    interval,
                    left,
                    right,
                },
                _ => NodeKind::Triggered {
                    interval,
                    left,
                    right,
                },
            };
            push(nodes, kind)
        }
    }
}

fn reference(profile: Profile, f: &O, word: &Word, at: usize) -> CaseResult<bool> {
    let cells: Vec<BTreeMap<_, bool>> = word
        .iter()
        .map(|cell| {
            (0..2u32)
                .map(|n| {
                    (
                        serde_json::from_value(serde_json::json!(n)).unwrap(),
                        cell[n as usize],
                    )
                })
                .collect()
        })
        .collect();
    let result = match profile {
        Profile::ClosedFuture => evaluate_closed_trace_v1(f, &cells, at, Limits::default()),
        Profile::OriginPast => evaluate_origin_complete(f, &cells, at, Limits::default()),
    };
    match result {
        Ok(value) => CaseResult::Admitted(value),
        Err(_) => CaseResult::Refused,
    }
}

fn present(cell: &Cell) -> Vec<PropositionId> {
    (0..2u32)
        .filter(|n| cell[*n as usize])
        .map(PropositionId)
        .collect()
}

/// Production side. `f` is whatever the candidate chooses to lower, so a seeded
/// fault can corrupt the formula it hands to the production evaluator.
fn production(profile: Profile, f: &O, word: &Word, at: usize) -> CaseResult<bool> {
    let mut nodes = Vec::new();
    let root = lower(f, &mut nodes);
    let semantic = match profile {
        Profile::ClosedFuture => SemanticProfile::ClosedTraceV1,
        Profile::OriginPast => SemanticProfile::OriginCompleteHistoryV1,
    };
    let Ok(formula) = Formula::new(semantic, root, &nodes) else {
        return CaseResult::Refused;
    };
    match profile {
        Profile::ClosedFuture => {
            let trace: Vec<Vec<PropositionId>> = word.iter().map(present).collect();
            match evaluate_closed_at(
                formula,
                "f",
                &trace,
                "t",
                at as u64,
                EvaluationLimits::default(),
            ) {
                Ok(report) => match report.verdict {
                    TruthValue::True => CaseResult::Admitted(true),
                    TruthValue::False => CaseResult::Admitted(false),
                    TruthValue::Pending => panic!("closed trace gave a pending verdict"),
                },
                Err(_) => CaseResult::Refused,
            }
        }
        Profile::OriginPast => {
            let observations = word
                .iter()
                .enumerate()
                .map(|(i, cell)| PositionObservation::new(i as u64, present(cell), None))
                .collect();
            let history = PositionHistoryDocument::new(
                "h",
                1,
                0,
                (word.len() - 1) as u64,
                Some(ClockBinding::EventPosition),
                observations,
            )
            .expect("history");
            match evaluate_past(
                formula,
                "f",
                &history,
                at as u64,
                "m",
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits::default(),
            ) {
                Ok(report) => CaseResult::Admitted(report.verdict),
                Err(_) => CaseResult::Refused,
            }
        }
    }
}

/// Runs the whole declared population; `candidate` may corrupt the formula.
fn sweep(
    profile: Profile,
    candidate: impl Fn(&O) -> O,
) -> Result<PopulationSummary, PopulationError> {
    let (formulas, words) = (formulas(), words());
    let cases: u64 = (formulas.len() * words.iter().map(Vec::len).sum::<usize>()) as u64;
    let mut ledger = PopulationLedger::new(cases).unwrap();
    let mut id = 0;
    for f in &formulas {
        let mutated = candidate(f);
        for word in &words {
            for at in 0..word.len() {
                let recorded = ledger.record(
                    id,
                    reference(profile, f, word, at),
                    production(profile, &mutated, word, at),
                );
                if recorded.is_err() {
                    // The ledger keeps the first failure; later cases add nothing.
                    return ledger.finish();
                }
                id += 1;
            }
        }
    }
    ledger.finish()
}

fn assert_agrees(profile: Profile) {
    let summary = sweep(profile, Clone::clone).unwrap_or_else(|e| panic!("{profile:?}: {e:?}"));
    assert!(summary.admitted > 0 && summary.refused > 0, "{profile:?}");
    assert_eq!(summary.admitted + summary.refused, summary.declared);
}

/// The full declared partition agrees with the oracle under the closed future profile.
#[test]
fn closed_future_agrees_with_oracle_over_complete_finite_partition() {
    assert_agrees(Profile::ClosedFuture);
}

/// The full declared partition agrees with the oracle under the origin-complete past profile.
#[test]
fn origin_past_agrees_with_oracle_over_complete_finite_partition() {
    assert_agrees(Profile::OriginPast);
}

fn shrink_end(f: &O) -> O {
    use O::*;
    let r = |i: &OI| match *i {
        OI::Closed { start, end } if end > start => OI::Closed {
            start,
            end: end - 1,
        },
        other => other,
    };
    let b = |x: &O| Box::new(shrink_end(x));
    match f {
        Not(a) => Not(b(a)),
        StrongPrevious(a) => StrongPrevious(b(a)),
        And(a, c) => And(b(a), b(c)),
        Or(a, c) => Or(b(a), b(c)),
        Implies(a, c) => Implies(b(a), b(c)),
        Equivalent(a, c) => Equivalent(b(a), b(c)),
        Future(i, a) => Future(r(i), b(a)),
        Globally(i, a) => Globally(r(i), b(a)),
        Once(i, a) => Once(r(i), b(a)),
        Historically(i, a) => Historically(r(i), b(a)),
        Until(i, a, c) => Until(r(i), b(a), b(c)),
        Release(i, a, c) => Release(r(i), b(a), b(c)),
        Since(i, a, c) => Since(r(i), b(a), b(c)),
        Triggered(i, a, c) => Triggered(r(i), b(a), b(c)),
        other => other.clone(),
    }
}

fn swap_dual(f: &O) -> O {
    use O::*;
    match f {
        Until(i, a, c) => Release(*i, a.clone(), c.clone()),
        Release(i, a, c) => Until(*i, a.clone(), c.clone()),
        Since(i, a, c) => Triggered(*i, a.clone(), c.clone()),
        Triggered(i, a, c) => Since(*i, a.clone(), c.clone()),
        Future(i, a) => Globally(*i, a.clone()),
        Globally(i, a) => Future(*i, a.clone()),
        Once(i, a) => Historically(*i, a.clone()),
        Historically(i, a) => Once(*i, a.clone()),
        other => other.clone(),
    }
}

fn drop_negation(f: &O) -> O {
    match f {
        O::Not(a) => (**a).clone(),
        other => other.clone(),
    }
}

/// Seeded production faults (interval off-by-one, swapped duals, lost
/// negation) must each turn the comparison red in both profiles.
#[test]
fn seeded_production_faults_turn_the_comparison_red() {
    type Fault = fn(&O) -> O;
    let faults: [(&str, Fault); 3] = [
        ("interval end off by one", shrink_end),
        ("dual operator swapped", swap_dual),
        ("negation dropped", drop_negation),
    ];
    for profile in [Profile::ClosedFuture, Profile::OriginPast] {
        for (name, fault) in faults {
            assert!(
                matches!(sweep(profile, fault), Err(PopulationError::Mismatch { .. })),
                "{profile:?}: seeded fault `{name}` was not detected"
            );
        }
    }
}
