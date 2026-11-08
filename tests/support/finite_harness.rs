//! Shared finite-profile harness: the oracle formula tree, its lowering to a
//! production `tl-syntax` arena, both evaluation sides, and a JSON codec so a
//! minimised disagreement or a hand-written corpus case is one portable shape.
//!
//! The expected answer always comes from `tl-oracle` (or, for the online-prefix
//! profile which the oracle does not model, from a hand-checked record). The
//! production side shares only `tl-syntax` input types with the oracle.
#![allow(dead_code)]

use std::collections::BTreeMap;

use serde_json::{json, Value};
use tl_mltl::{
    evaluate_closed_at, evaluate_past, ClockBinding, EvaluationLimits, PastEvaluationLimits,
    PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TruthValue,
};
use tl_oracle::population::CaseResult;
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as O, Interval as OI, Limits,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

/// A cell holds the truth of propositions 0 and 1. The oracle pins its own
/// `tl-syntax` revision, so proposition identities cross the seam as integers.
pub type Cell = [bool; 2];
pub type Word = Vec<Cell>;

pub fn oracle_atom(n: u32) -> O {
    O::Atom(serde_json::from_value(json!(n)).expect("oracle proposition id"))
}

pub fn atom_number(p: &impl serde::Serialize) -> u32 {
    serde_json::to_value(p)
        .unwrap()
        .as_u64()
        .expect("numeric proposition id") as u32
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Profile {
    ClosedFuture,
    OriginPast,
}

impl Profile {
    pub fn token(self) -> &'static str {
        match self {
            Profile::ClosedFuture => "mltl.closed-trace/v1",
            Profile::OriginPast => "mltl.origin-complete-history/v1",
        }
    }
}

pub fn oi((start, end): (usize, usize)) -> OI {
    OI::Closed { start, end }
}

/// Lowers the oracle tree to a production `tl-syntax` node arena.
pub fn lower(formula: &O, nodes: &mut Vec<Node>) -> NodeId {
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

/// The oracle's default offset ceiling (4096) would refuse an interval at the
/// `u32` extreme that production evaluates exactly, so the ceiling is raised to
/// the `u32` range; the oracle's work budget still refuses huge windows.
pub fn oracle_limits() -> Limits {
    Limits {
        max_offset: u32::MAX as usize,
        ..Limits::default()
    }
}

pub fn reference(profile: Profile, f: &O, word: &Word, at: usize) -> CaseResult<bool> {
    let cells: Vec<BTreeMap<_, bool>> = word
        .iter()
        .map(|cell| {
            (0..2u32)
                .map(|n| (serde_json::from_value(json!(n)).unwrap(), cell[n as usize]))
                .collect()
        })
        .collect();
    let result = match profile {
        Profile::ClosedFuture => evaluate_closed_trace_v1(f, &cells, at, oracle_limits()),
        Profile::OriginPast => evaluate_origin_complete(f, &cells, at, oracle_limits()),
    };
    match result {
        Ok(value) => CaseResult::Admitted(value),
        Err(_) => CaseResult::Refused,
    }
}

pub fn present(cell: &Cell) -> Vec<PropositionId> {
    (0..2u32)
        .filter(|n| cell[*n as usize])
        .map(PropositionId)
        .collect()
}

/// Production side. `f` is whatever the candidate chooses to lower, so a seeded
/// fault can corrupt the formula it hands to the production evaluator.
pub fn production(profile: Profile, f: &O, word: &Word, at: usize) -> CaseResult<bool> {
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
            let Ok(history) = PositionHistoryDocument::new(
                "h",
                1,
                0,
                word.len().saturating_sub(1) as u64,
                Some(ClockBinding::EventPosition),
                observations,
            ) else {
                // An empty history is not a valid document.
                return CaseResult::Refused;
            };
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

// ---------------------------------------------------------------------------
// JSON codec: one portable shape for corpus cases and minimised disagreements.
// ---------------------------------------------------------------------------

fn interval_json(i: &OI) -> Value {
    match i {
        OI::Closed { start, end } => json!([start, end]),
        OI::Unbounded { .. } => unreachable!("finite shapes only"),
    }
}

/// `{"op":"until","interval":[a,b],"left":..,"right":..}` and friends.
pub fn formula_to_json(f: &O) -> Value {
    let un = |op: &str, a: &O| json!({"op": op, "arg": formula_to_json(a)});
    let bin = |op: &str, a: &O, b: &O| json!({"op": op, "left": formula_to_json(a), "right": formula_to_json(b)});
    let tun = |op: &str, i: &OI, a: &O| json!({"op": op, "interval": interval_json(i), "arg": formula_to_json(a)});
    let tbin = |op: &str, i: &OI, a: &O, b: &O| {
        json!({"op": op, "interval": interval_json(i),
               "left": formula_to_json(a), "right": formula_to_json(b)})
    };
    match f {
        O::False => json!({"op": "false"}),
        O::True => json!({"op": "true"}),
        O::Atom(p) => json!({"op": "prop", "id": atom_number(p)}),
        O::Not(a) => un("not", a),
        O::StrongPrevious(a) => un("yesterday", a),
        O::And(a, b) => bin("and", a, b),
        O::Or(a, b) => bin("or", a, b),
        O::Implies(a, b) => bin("implies", a, b),
        O::Equivalent(a, b) => bin("equivalent", a, b),
        O::Future(i, a) => tun("future", i, a),
        O::Globally(i, a) => tun("globally", i, a),
        O::Once(i, a) => tun("once", i, a),
        O::Historically(i, a) => tun("historically", i, a),
        O::Until(i, a, b) => tbin("until", i, a, b),
        O::Release(i, a, b) => tbin("release", i, a, b),
        O::Since(i, a, b) => tbin("since", i, a, b),
        O::Triggered(i, a, b) => tbin("triggered", i, a, b),
    }
}

pub fn formula_from_json(v: &Value) -> O {
    let s = |k: &str| v.get(k).unwrap_or_else(|| panic!("missing `{k}` in {v}"));
    let arg = || Box::new(formula_from_json(s("arg")));
    let l = || Box::new(formula_from_json(s("left")));
    let r = || Box::new(formula_from_json(s("right")));
    let iv = || {
        let a = s("interval").as_array().expect("interval pair");
        OI::Closed {
            start: a[0].as_u64().expect("start") as usize,
            end: a[1].as_u64().expect("end") as usize,
        }
    };
    match s("op").as_str().expect("op string") {
        "false" => O::False,
        "true" => O::True,
        "prop" => oracle_atom(s("id").as_u64().expect("id") as u32),
        "not" => O::Not(arg()),
        "yesterday" => O::StrongPrevious(arg()),
        "and" => O::And(l(), r()),
        "or" => O::Or(l(), r()),
        "implies" => O::Implies(l(), r()),
        "equivalent" => O::Equivalent(l(), r()),
        "future" => O::Future(iv(), arg()),
        "globally" => O::Globally(iv(), arg()),
        "once" => O::Once(iv(), arg()),
        "historically" => O::Historically(iv(), arg()),
        "until" => O::Until(iv(), l(), r()),
        "release" => O::Release(iv(), l(), r()),
        "since" => O::Since(iv(), l(), r()),
        "triggered" => O::Triggered(iv(), l(), r()),
        other => panic!("unknown operator `{other}`"),
    }
}

/// A trace is an array of instants, each the numeric ids of true propositions.
pub fn word_to_json(word: &Word) -> Value {
    Value::Array(
        word.iter()
            .map(|c| {
                Value::Array(
                    (0..2u32)
                        .filter(|n| c[*n as usize])
                        .map(|n| json!(n))
                        .collect(),
                )
            })
            .collect(),
    )
}

pub fn word_from_json(v: &Value) -> Word {
    v.as_array()
        .expect("trace array")
        .iter()
        .map(|instant| {
            let mut cell = [false; 2];
            for id in instant.as_array().expect("instant array") {
                cell[id.as_u64().expect("proposition id") as usize] = true;
            }
            cell
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Seeded production faults, shared by the sweep and the corpus mutation checks.
// ---------------------------------------------------------------------------

/// Seeded fault: every closed interval loses one position at its upper end.
pub fn shrink_end(f: &O) -> O {
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

/// Seeded fault: every operator is replaced by its dual.
pub fn swap_dual(f: &O) -> O {
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

/// Seeded fault: every top-level negation is lost.
pub fn drop_negation(f: &O) -> O {
    match f {
        O::Not(a) => (**a).clone(),
        other => other.clone(),
    }
}

/// Seeded fault: every closed interval's lower bound moves one position earlier.
pub fn lower_start_down(f: &O) -> O {
    use O::*;
    let r = |i: &OI| match *i {
        OI::Closed { start, end } => OI::Closed {
            start: start.saturating_sub(1),
            end,
        },
        other => other,
    };
    let b = |x: &O| Box::new(lower_start_down(x));
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
