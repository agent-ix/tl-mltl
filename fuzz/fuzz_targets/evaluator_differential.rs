#![no_main]
#![forbid(unsafe_code)]

use std::collections::BTreeMap;

use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use tl_mltl::{
    evaluate_closed_at, evaluate_past, ClockBinding, EvaluationLimits, PastEvaluationLimits,
    PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TruthValue,
};
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as O, Interval as OI, Limits,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const MAX_NODES: usize = 32;
const PROPOSITIONS: u32 = 3;
const MAX_TRACE: usize = 12;
const MAX_TREE: usize = 4096;

fn interval(u: &mut Unstructured<'_>) -> Option<Interval> {
    // Wider than the exhaustive V1 population (end <= 2): intervals reach 8.
    let start = u.int_in_range(0..=4_u32).ok()?;
    let width = u.int_in_range(0..=4_u32).ok()?;
    Interval::new(start, start + width).ok()
}

// Structure-aware generation: operands reference earlier nodes only, so every
// input is a valid DAG and no execution is spent on parse failures.
fn nodes(u: &mut Unstructured<'_>, past: bool) -> Option<Vec<Node>> {
    let count = u.int_in_range(1..=MAX_NODES).ok()?;
    let mut nodes: Vec<Node> = Vec::with_capacity(count);
    for index in 0..count {
        let operand = |u: &mut Unstructured<'_>| -> Option<NodeId> {
            let back = u.int_in_range(0..=index.saturating_sub(1).min(5)).ok()?;
            Some(NodeId(u32::try_from(index - 1 - back).ok()?))
        };
        let leaf = index == 0;
        let choice = u.int_in_range(0..=if leaf { 2_u8 } else { 13 }).ok()?;
        let kind = match choice {
            0 => NodeKind::True,
            1 => NodeKind::False,
            2 => NodeKind::Proposition {
                proposition: PropositionId(u.int_in_range(0..=PROPOSITIONS - 1).ok()?),
            },
            3 => NodeKind::Not {
                operand: operand(u)?,
            },
            4 => NodeKind::And {
                left: operand(u)?,
                right: operand(u)?,
            },
            5 => NodeKind::Or {
                left: operand(u)?,
                right: operand(u)?,
            },
            6 => NodeKind::Implies {
                left: operand(u)?,
                right: operand(u)?,
            },
            7 => NodeKind::Equivalent {
                left: operand(u)?,
                right: operand(u)?,
            },
            8 if past => NodeKind::Once {
                interval: interval(u)?,
                operand: operand(u)?,
            },
            9 if past => NodeKind::Historically {
                interval: interval(u)?,
                operand: operand(u)?,
            },
            10 if past => NodeKind::Since {
                interval: interval(u)?,
                left: operand(u)?,
                right: operand(u)?,
            },
            11 if past => NodeKind::Triggered {
                interval: interval(u)?,
                left: operand(u)?,
                right: operand(u)?,
            },
            12 | 13 if past => NodeKind::StrongPrevious {
                operand: operand(u)?,
            },
            8 => NodeKind::Future {
                interval: interval(u)?,
                operand: operand(u)?,
            },
            9 => NodeKind::Globally {
                interval: interval(u)?,
                operand: operand(u)?,
            },
            10 | 12 => NodeKind::Until {
                interval: interval(u)?,
                left: operand(u)?,
                right: operand(u)?,
            },
            _ => NodeKind::Release {
                interval: interval(u)?,
                left: operand(u)?,
                right: operand(u)?,
            },
        };
        nodes.push(Node::new(kind));
    }
    Some(nodes)
}

fn atom(n: u32) -> O {
    // The oracle pins its own tl-syntax revision, so identities cross as integers.
    O::Atom(serde_json::from_value(serde_json::json!(n)).expect("oracle proposition id"))
}

fn tree_size(nodes: &[Node]) -> usize {
    let mut sizes: Vec<usize> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let at = |id: NodeId| sizes[usize::try_from(id.0).unwrap()];
        let size = match node.kind {
            NodeKind::True | NodeKind::False | NodeKind::Proposition { .. } => 1,
            NodeKind::Not { operand }
            | NodeKind::Future { operand, .. }
            | NodeKind::Globally { operand, .. }
            | NodeKind::Once { operand, .. }
            | NodeKind::Historically { operand, .. }
            | NodeKind::StrongPrevious { operand } => 1 + at(operand),
            NodeKind::And { left, right }
            | NodeKind::Or { left, right }
            | NodeKind::Implies { left, right }
            | NodeKind::Equivalent { left, right }
            | NodeKind::Until { left, right, .. }
            | NodeKind::Release { left, right, .. }
            | NodeKind::Since { left, right, .. }
            | NodeKind::Triggered { left, right, .. } => {
                1usize.saturating_add(at(left)).saturating_add(at(right))
            }
        };
        sizes.push(size.min(MAX_TREE + 1));
    }
    sizes.last().copied().unwrap_or(0)
}

// Independent oracle tree; no evaluator logic or production semantic
// function is shared with the production side.
fn oracle_tree(nodes: &[Node]) -> O {
    let mut trees: Vec<O> = Vec::with_capacity(nodes.len());
    for node in nodes {
        let child = |id: NodeId| trees[usize::try_from(id.0).unwrap()].clone();
        let span = |value: Interval| OI::Closed {
            start: usize::try_from(value.start()).unwrap(),
            end: usize::try_from(value.end()).unwrap(),
        };
        let b = Box::new;
        let tree = match node.kind {
            NodeKind::False => O::False,
            NodeKind::True => O::True,
            NodeKind::Proposition { proposition } => atom(proposition.0),
            NodeKind::Not { operand } => O::Not(b(child(operand))),
            NodeKind::And { left, right } => O::And(b(child(left)), b(child(right))),
            NodeKind::Or { left, right } => O::Or(b(child(left)), b(child(right))),
            NodeKind::Implies { left, right } => O::Implies(b(child(left)), b(child(right))),
            NodeKind::Equivalent { left, right } => O::Equivalent(b(child(left)), b(child(right))),
            NodeKind::Future { interval, operand } => O::Future(span(interval), b(child(operand))),
            NodeKind::Globally { interval, operand } => {
                O::Globally(span(interval), b(child(operand)))
            }
            NodeKind::Until {
                interval,
                left,
                right,
            } => O::Until(span(interval), b(child(left)), b(child(right))),
            NodeKind::Release {
                interval,
                left,
                right,
            } => O::Release(span(interval), b(child(left)), b(child(right))),
            NodeKind::Once { interval, operand } => O::Once(span(interval), b(child(operand))),
            NodeKind::Historically { interval, operand } => {
                O::Historically(span(interval), b(child(operand)))
            }
            NodeKind::StrongPrevious { operand } => O::StrongPrevious(b(child(operand))),
            NodeKind::Since {
                interval,
                left,
                right,
            } => O::Since(span(interval), b(child(left)), b(child(right))),
            NodeKind::Triggered {
                interval,
                left,
                right,
            } => O::Triggered(span(interval), b(child(left)), b(child(right))),
        };
        trees.push(tree);
    }
    trees.pop().expect("at least one node")
}

// Trace: TC-001, FR-001-AC-1
fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);
    let Ok(past) = u.arbitrary::<bool>() else {
        return;
    };
    let Some(nodes) = nodes(&mut u, past) else {
        return;
    };
    if tree_size(&nodes) > MAX_TREE {
        return;
    }
    let Ok(len) = u.int_in_range(1..=MAX_TRACE) else {
        return;
    };
    let mut word: Vec<Vec<PropositionId>> = Vec::with_capacity(len);
    for _ in 0..len {
        let mut present = Vec::new();
        for id in 0..PROPOSITIONS {
            match u.arbitrary::<bool>() {
                Ok(true) => present.push(PropositionId(id)),
                Ok(false) => {}
                Err(_) => return,
            }
        }
        word.push(present);
    }
    let profile = if past {
        SemanticProfile::OriginCompleteHistoryV1
    } else {
        SemanticProfile::ClosedTraceV1
    };
    let root = NodeId(u32::try_from(nodes.len() - 1).unwrap());
    let Ok(formula) = Formula::new(profile, root, &nodes) else {
        return;
    };
    let tree = oracle_tree(&nodes);
    let cells: Vec<BTreeMap<_, bool>> = word
        .iter()
        .map(|present| {
            (0..PROPOSITIONS)
                .map(|n| {
                    let key = serde_json::from_value(serde_json::json!(n)).unwrap();
                    (key, present.contains(&PropositionId(n)))
                })
                .collect()
        })
        .collect();
    let history = past.then(|| {
        let observations = word
            .iter()
            .enumerate()
            .map(|(i, present)| PositionObservation::new(i as u64, present.clone(), None))
            .collect();
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            (word.len() - 1) as u64,
            Some(ClockBinding::EventPosition),
            observations,
        )
        .expect("history")
    });
    for at in 0..word.len() {
        let expected = if past {
            evaluate_origin_complete(&tree, &cells, at, Limits::default())
        } else {
            evaluate_closed_trace_v1(&tree, &cells, at, Limits::default())
        };
        let actual = match &history {
            Some(history) => evaluate_past(
                formula,
                "f",
                history,
                at as u64,
                "m",
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits::default(),
            )
            .map(|report| report.verdict)
            .map_err(|error| format!("{error:?}")),
            None => evaluate_closed_at(
                formula,
                "f",
                &word,
                "t",
                at as u64,
                EvaluationLimits::default(),
            )
            .map(|report| match report.verdict {
                TruthValue::True => true,
                TruthValue::False => false,
                TruthValue::Pending => panic!("closed trace gave a pending verdict"),
            })
            .map_err(|error| format!("{error:?}")),
        };
        // The oracle refusing a case is a resource decision, not a verdict;
        // every case the oracle decides must be decided identically.
        if let Ok(expected) = expected {
            assert_eq!(
                actual,
                Ok(expected),
                "production differs from oracle: past={past} at={at} word={word:?} nodes={nodes:?}"
            );
        }
    }
});
