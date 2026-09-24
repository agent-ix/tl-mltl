//! Source-bound replay of the four retained V10 monitor cases.
//! Trace: FR-055-AC-2, TC-198.

use std::collections::BTreeMap;

use serde_json::Value;
use sha2::{Digest, Sha256};
use tl_mltl::{
    evaluate_past, evaluate_prefix_at, map_past_to_c2po, ClockBinding, EvaluationLimits,
    MappingSourceIdentity, MappingSourceState, PastEvaluationLimits, PastEvaluationRelationInput,
    PastMappingError, PositionHistoryDocument, PositionObservation, TargetOriginContract,
    TruthValue,
};
use tl_oracle::{
    evaluate as evaluate_lasso_oracle, evaluate_closed_trace_v1, evaluate_origin_complete,
    Evidence as OracleEvidence, Formula as OracleFormula, Interval as OracleInterval,
    Lasso as OracleLasso, Limits as OracleLimits, Verdict as OracleVerdict,
};
use tl_syntax::{
    Formula, FormulaDocument, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration,
    PastOperatorKind, PropositionBinding, PropositionId, SemanticProfile, SignalCatalogDocument,
    SignalDomain, SignalId,
};

#[cfg(test)]
use super::v10_replay::ComparisonClass;
use super::v10_replay::{CellDetail, DetailReason, RefusalDetail, Replay};

const BOUNDED_MANIFEST: &[u8] = include_bytes!("../../../corpus/r2u2-v4.2/manifest.json");
const BOUNDED_SPEC: &[u8] = include_bytes!("../../../corpus/r2u2-v4.2/formulas.c2po");
const BOUNDED_MAP: &[u8] = include_bytes!("../../../corpus/r2u2-v4.2/signals.map");
const BOUNDED_TRACE: &[u8] = include_bytes!("../../../corpus/r2u2-v4.2/trace.csv");
const PAST_MANIFEST: &[u8] = include_bytes!("../../../corpus/past-c2po-v1/manifest.json");
const PAST_SPEC: &[u8] = include_bytes!("../../../corpus/past-c2po-v1/target-4.2/past.c2po");
const PAST_TRACE: &[u8] = include_bytes!("../../../corpus/past-c2po-v1/target-4.2/trace.csv");
const UNSAFE_SPEC: &[u8] =
    include_bytes!("../../../corpus/past-c2po-v1/target-4.2/unsafe-since.c2po");
const UNSAFE_TRACE: &[u8] =
    include_bytes!("../../../corpus/past-c2po-v1/target-4.2/unsafe-since.csv");
const SAFETY_SPEC: &[u8] = b"INPUT\n q: bool;\nFTSPEC\n q;\n";
const SAFETY_TRACE: &[u8] = b"# q\n0\n1\n";

pub(super) struct Inputs {
    pub(super) spec: &'static [u8],
    pub(super) trace: &'static [u8],
    pub(super) map: Option<&'static [u8]>,
}

pub(super) fn inputs(case: &str) -> Option<Inputs> {
    match case {
        "bounded" => Some(Inputs {
            spec: BOUNDED_SPEC,
            trace: BOUNDED_TRACE,
            map: Some(BOUNDED_MAP),
        }),
        "past" => Some(Inputs {
            spec: PAST_SPEC,
            trace: PAST_TRACE,
            map: None,
        }),
        "unsafe-since" => Some(Inputs {
            spec: UNSAFE_SPEC,
            trace: UNSAFE_TRACE,
            map: None,
        }),
        "safety" => Some(Inputs {
            spec: SAFETY_SPEC,
            trace: SAFETY_TRACE,
            map: None,
        }),
        _ => None,
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn pinned_inputs(case: &str) -> bool {
    match case {
        "bounded" => {
            sha256(BOUNDED_MANIFEST)
                == "f1e34ef4c79d555c6a6eecdc857b1ee23182cffcefa363a9c84dd0db02511882"
                && sha256(BOUNDED_SPEC)
                    == "1b16e7446bce55e42b927ce5a3439041cec59e6bfeaff626ef0d78caf00b2938"
                && sha256(BOUNDED_MAP)
                    == "257aad1e39c430cf87490fba85d855f9f265af435687ca16ea18b331d4abac5d"
                && sha256(BOUNDED_TRACE)
                    == "4f0b0db7cfe9746c323696c3f4a3ae3e5844af4003f9e870587c7c39f40b963a"
        }
        "past" | "unsafe-since" => {
            sha256(PAST_MANIFEST)
                == "0c935abf961c765f72b6c2d7d6d05287c798daf28bec1ef668cdf1c6df1bb616"
                && sha256(PAST_SPEC)
                    == "4e0c904eccfbf7a2efdd08dfe268d1862d3a2ea473595e34afd118af4a6cb915"
                && sha256(PAST_TRACE)
                    == "8b1d83c99985139a2272bf21d83bf5a2667500db276e36ef1b44e60c1a3ced32"
                && sha256(UNSAFE_SPEC)
                    == "a76ac511612e2d2f77379e37366a7b21010331d5fc901c3388c58b90bdffa136"
                && sha256(UNSAFE_TRACE)
                    == "02e2f5935de837e57673d45a209ee353c9a4e2cec40bec5a2944307a34ed378d"
        }
        "safety" => true,
        _ => false,
    }
}

#[cfg(test)]
pub(super) fn replay(case: &str, target: &BTreeMap<(usize, usize), bool>) -> Replay {
    replay_detailed(case, target).0
}

pub(super) fn replay_detailed(
    case: &str,
    target: &BTreeMap<(usize, usize), bool>,
) -> (Replay, Vec<CellDetail>) {
    let mut cells = Vec::new();
    let verdict = replay_collect(case, target, &mut cells);
    (verdict, cells)
}

fn replay_collect(
    case: &str,
    target: &BTreeMap<(usize, usize), bool>,
    cells: &mut Vec<CellDetail>,
) -> Replay {
    if !pinned_inputs(case) {
        return Replay::Reject("v10_static_source_pin_unproved");
    }
    match case {
        "bounded" => bounded(target, cells),
        "past" => past(target, cells),
        "unsafe-since" => unsafe_since(target, cells),
        "safety" => safety(target, cells),
        _ => Replay::Reject("v10_unknown_static_monitor"),
    }
}

fn bounded_formula(index: usize) -> Option<OracleFormula> {
    use OracleFormula as O;
    let p = |id| O::Atom(PropositionId(id));
    let range = |start, end| OracleInterval::Closed { start, end };
    Some(match index {
        0 => O::Future(range(1, 2), Box::new(p(0))),
        1 => O::Globally(range(0, 2), Box::new(p(1))),
        2 => O::Future(range(0, 3), Box::new(p(2))),
        3 => O::Until(range(1, 2), Box::new(p(0)), Box::new(p(1))),
        4 => O::Release(range(1, 2), Box::new(p(3)), Box::new(p(2))),
        5 => O::Future(
            range(0, 0),
            Box::new(O::Until(range(1, 2), Box::new(p(0)), Box::new(p(1)))),
        ),
        _ => return None,
    })
}

fn bounded(target: &BTreeMap<(usize, usize), bool>, cells: &mut Vec<CellDetail>) -> Replay {
    if target.keys().any(|(formula, _)| *formula >= 6) {
        return Replay::Reject("v10_unknown_target_formula");
    }
    let Ok(manifest) = serde_json::from_slice::<Value>(BOUNDED_MANIFEST) else {
        return Replay::Reject("v10_bounded_manifest_unproved");
    };
    let Some(cases) = manifest["cases"].as_array() else {
        return Replay::Reject("v10_bounded_manifest_unproved");
    };
    if cases.len() != 8 {
        return Replay::Reject("v10_bounded_population_unproved");
    }
    let trace = [
        vec![],
        vec![PropositionId(0), PropositionId(1)],
        vec![PropositionId(1)],
        vec![PropositionId(1)],
    ];
    let word = trace
        .iter()
        .map(|row| {
            (0..4)
                .map(|id| (PropositionId(id), row.contains(&PropositionId(id))))
                .collect()
        })
        .collect::<Vec<BTreeMap<_, _>>>();
    let mut seen = std::collections::BTreeSet::new();
    let mut missing = false;
    let mut mismatch = false;
    for case in cases {
        let (Some(index), Some(position), Some(expected)) = (
            case["formulaIndex"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok()),
            case["expected"]["verdictTime"]
                .as_u64()
                .and_then(|n| usize::try_from(n).ok()),
            case["expected"]["verdict"].as_bool(),
        ) else {
            return Replay::Reject("v10_bounded_manifest_unproved");
        };
        if !seen.insert((index, position)) || position >= trace.len() {
            return Replay::Reject("v10_bounded_population_unproved");
        }
        let (Some(oracle_formula), Ok(document)) = (
            bounded_formula(index),
            serde_json::from_value::<FormulaDocument>(case["formula"].clone()),
        ) else {
            return Replay::Reject("v10_bounded_formula_unproved");
        };
        let Ok(graph) = document.validate() else {
            return Replay::Reject("v10_bounded_formula_unproved");
        };
        let Ok(tl) = evaluate_prefix_at(
            graph,
            case["id"].as_str().unwrap_or(""),
            &trace,
            "r2u2-v4.2-trace",
            false,
            position as u64,
            EvaluationLimits::default(),
        ) else {
            return Replay::Reject("v10_bounded_tl_unproved");
        };
        let Ok(oracle) =
            evaluate_closed_trace_v1(&oracle_formula, &word, position, OracleLimits::default())
        else {
            return Replay::Reject("v10_bounded_oracle_unproved");
        };
        if tl.verdict
            != if oracle {
                TruthValue::True
            } else {
                TruthValue::False
            }
            || expected != oracle
        {
            return Replay::Reject("v10_bounded_source_oracle_disagreement");
        }
        cells.push(CellDetail::compared(
            index,
            position,
            oracle,
            target.get(&(index, position)).copied(),
        ));
        match target.get(&(index, position)) {
            Some(observed) if *observed == oracle => {}
            Some(_) => mismatch = true,
            None => missing = true,
        }
    }
    if mismatch {
        Replay::Reject("v10_target_semantic_mismatch")
    } else if missing {
        Replay::Inconclusive("v10_admitted_target_row_missing")
    } else {
        Replay::Accept {
            admitted_cells: 8,
            unsupported_cells: 0,
        }
    }
}

fn past_formula(index: usize) -> Option<(Vec<Node>, OracleFormula, &'static str, bool)> {
    use OracleFormula as O;
    let p = Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    });
    let q = Node::new(NodeKind::Proposition {
        proposition: PropositionId(1),
    });
    let atom = |id| O::Atom(PropositionId(id));
    let range = |start, end| OracleInterval::Closed { start, end };
    let interval = |start, end| Interval::new(start, end).expect("fixed interval");
    Some(match index {
        0 => (
            vec![
                p,
                Node::new(NodeKind::Once {
                    interval: interval(0, 1),
                    operand: NodeId(0),
                }),
            ],
            O::Once(range(0, 1), Box::new(atom(0))),
            "O[0,1](p)",
            true,
        ),
        1 => (
            vec![
                p,
                Node::new(NodeKind::Historically {
                    interval: interval(0, 1),
                    operand: NodeId(0),
                }),
            ],
            O::Historically(range(0, 1), Box::new(atom(0))),
            "H[0,1](p)",
            false,
        ),
        2 => (
            vec![
                p,
                Node::new(NodeKind::StrongPrevious { operand: NodeId(0) }),
            ],
            O::StrongPrevious(Box::new(atom(0))),
            "O[1,1](p)",
            true,
        ),
        3 => (
            vec![
                p,
                q,
                Node::new(NodeKind::Since {
                    interval: interval(0, 2),
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ],
            O::Since(range(0, 2), Box::new(atom(0)), Box::new(atom(1))),
            "(p S[0,2] q)",
            false,
        ),
        4 => (
            vec![
                p,
                q,
                Node::new(NodeKind::Triggered {
                    interval: interval(0, 2),
                    left: NodeId(0),
                    right: NodeId(1),
                }),
            ],
            O::Triggered(range(0, 2), Box::new(atom(0)), Box::new(atom(1))),
            "(!((!p) S[0,2] (!q)))",
            false,
        ),
        5 => (
            vec![
                p,
                Node::new(NodeKind::Once {
                    interval: interval(1, 1),
                    operand: NodeId(0),
                }),
            ],
            O::Once(range(1, 1), Box::new(atom(0))),
            "O[1,1](p)",
            false,
        ),
        _ => return None,
    })
}

fn past_word(observations: &[(bool, bool)]) -> Vec<BTreeMap<PropositionId, bool>> {
    observations
        .iter()
        .map(|(p, q)| [(PropositionId(0), *p), (PropositionId(1), *q)].into())
        .collect()
}

fn history(observations: &[(bool, bool)]) -> Option<PositionHistoryDocument> {
    PositionHistoryDocument::new(
        "v10-static-past",
        1,
        0,
        (observations.len() - 1) as u64,
        Some(ClockBinding::EventPosition),
        observations
            .iter()
            .enumerate()
            .map(|(position, (p, q))| {
                PositionObservation::new(
                    position as u64,
                    [(PropositionId(0), *p), (PropositionId(1), *q)]
                        .into_iter()
                        .filter_map(|(id, value)| value.then_some(id))
                        .collect(),
                    None,
                )
            })
            .collect(),
    )
    .ok()
}

fn past_catalog() -> Option<SignalCatalogDocument> {
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(1), "p".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(SignalId(2), "q".to_owned(), SignalDomain::Boolean),
        ],
        vec![
            PropositionBinding::new(PropositionId(0), SignalId(1)),
            PropositionBinding::new(PropositionId(1), SignalId(2)),
        ],
    )
    .ok()
}

fn expected_past_refusal(index: usize, error: &PastMappingError) -> bool {
    let (expected_operator, expected_interval) = match index {
        1 => (
            PastOperatorKind::Historically,
            Interval::new(0, 1).expect("fixed interval"),
        ),
        3 => (
            PastOperatorKind::Since,
            Interval::new(0, 2).expect("fixed interval"),
        ),
        4 => (
            PastOperatorKind::Triggered,
            Interval::new(0, 2).expect("fixed interval"),
        ),
        5 => (
            PastOperatorKind::Once,
            Interval::new(1, 1).expect("fixed interval"),
        ),
        _ => return false,
    };
    matches!(
        error,
        PastMappingError::TargetOriginIntervalMismatch { operator, interval }
            if *operator == expected_operator && *interval == expected_interval
    )
}

fn past_case(
    index: usize,
    observations: &[(bool, bool)],
    target: &BTreeMap<(usize, usize), bool>,
    cells: &mut Vec<CellDetail>,
) -> Replay {
    let Some((nodes, oracle_formula, expression, expected_admitted)) = past_formula(index) else {
        return Replay::Reject("v10_past_formula_unproved");
    };
    let Some(source) = std::str::from_utf8(PAST_SPEC).ok() else {
        return Replay::Reject("v10_past_spec_expression_unproved");
    };
    let Some((_, section)) = source.split_once("PTSPEC") else {
        return Replay::Reject("v10_past_spec_expression_unproved");
    };
    let expressions = section
        .split(';')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if expressions.len() != 6 || expressions[index] != expression {
        return Replay::Reject("v10_past_spec_expression_unproved");
    }
    let Ok(graph) = Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).expect("fixed node count")),
        &nodes,
    ) else {
        return Replay::Reject("v10_past_formula_unproved");
    };
    let Some(history) = history(observations) else {
        return Replay::Reject("v10_past_history_unproved");
    };
    let Some(catalog) = past_catalog() else {
        return Replay::Reject("v10_past_catalog_unproved");
    };
    let Ok(document) = FormulaDocument::from_formula_v2(graph) else {
        return Replay::Reject("v10_past_document_unproved");
    };
    let Ok(bytes) = document.canonical_json_bytes() else {
        return Replay::Reject("v10_past_document_unproved");
    };
    let mapping = map_past_to_c2po(
        graph,
        "v10-static-past",
        &bytes,
        MappingSourceIdentity {
            revision: "v10-static-past".to_owned(),
            state: MappingSourceState::Clean,
        },
        &catalog,
        &TargetOriginContract::reviewed_r2u2_4_2(),
        100,
    );
    let refusal = match (expected_admitted, mapping) {
        (true, Ok(mapped))
            if mapped.expression == expression
                && mapped.output_sha256 == sha256(expression.as_bytes()) =>
        {
            None
        }
        (false, Err(refusal)) if expected_past_refusal(index, &refusal) => {
            Some(RefusalDetail::validated(&refusal))
        }
        _ => return Replay::Reject("v10_mapping_partition_unproved"),
    };
    let word = past_word(observations);
    let mut missing = false;
    let mut mismatch = false;
    for position in 0..observations.len() {
        let Ok(tl) = evaluate_past(
            graph,
            "v10-static-past",
            &history,
            position as u64,
            "map",
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits::default(),
        ) else {
            return Replay::Reject("v10_past_tl_unproved");
        };
        let Ok(oracle) =
            evaluate_origin_complete(&oracle_formula, &word, position, OracleLimits::default())
        else {
            return Replay::Reject("v10_past_oracle_unproved");
        };
        if tl.verdict != oracle {
            return Replay::Reject("v10_past_source_oracle_disagreement");
        }
        if expected_admitted {
            cells.push(CellDetail::compared(
                index,
                position,
                oracle,
                target.get(&(index, position)).copied(),
            ));
            match target.get(&(index, position)) {
                Some(observed) if *observed == oracle => {}
                Some(_) => mismatch = true,
                None => missing = true,
            }
        } else {
            cells.push(CellDetail::unsupported(
                index,
                position,
                oracle,
                target.get(&(index, position)).copied(),
                refusal
                    .clone()
                    .expect("validated unsupported mapping has refusal"),
            ));
        }
    }
    if mismatch {
        Replay::Reject("v10_target_semantic_mismatch")
    } else if missing {
        Replay::Inconclusive("v10_admitted_target_row_missing")
    } else {
        Replay::Accept {
            admitted_cells: usize::from(expected_admitted) * observations.len(),
            unsupported_cells: usize::from(!expected_admitted) * observations.len(),
        }
    }
}

fn past(target: &BTreeMap<(usize, usize), bool>, cells: &mut Vec<CellDetail>) -> Replay {
    if target.keys().any(|(formula, _)| *formula >= 6) {
        return Replay::Reject("v10_unknown_target_formula");
    }
    let observations = [(false, false), (true, false), (true, true)];
    let mut admitted = 0;
    let mut unsupported = 0;
    let mut missing = false;
    let mut mismatch = false;
    for index in 0..6 {
        match past_case(index, &observations, target, cells) {
            Replay::Accept {
                admitted_cells,
                unsupported_cells,
            } => {
                admitted += admitted_cells;
                unsupported += unsupported_cells;
            }
            Replay::Reject("v10_target_semantic_mismatch") => {
                mismatch = true;
                admitted += 3;
            }
            Replay::Inconclusive("v10_admitted_target_row_missing") => {
                missing = true;
                admitted += 3;
            }
            other => return other,
        }
    }
    if (admitted, unsupported) != (6, 12) {
        Replay::Reject("v10_past_population_unproved")
    } else if mismatch {
        Replay::Reject("v10_target_semantic_mismatch")
    } else if missing {
        Replay::Inconclusive("v10_admitted_target_row_missing")
    } else {
        Replay::Accept {
            admitted_cells: admitted,
            unsupported_cells: unsupported,
        }
    }
}

fn unsafe_since(target: &BTreeMap<(usize, usize), bool>, cells: &mut Vec<CellDetail>) -> Replay {
    if target.keys().any(|(formula, _)| *formula != 0) {
        return Replay::Reject("v10_unknown_target_formula");
    }
    let observations = [(false, true), (false, false), (true, false)];
    let mut remapped = BTreeMap::new();
    for ((formula, position), verdict) in target {
        remapped.insert((formula + 3, *position), *verdict);
    }
    match past_case(3, &observations, &remapped, cells) {
        Replay::Accept {
            admitted_cells: 0,
            unsupported_cells: 3,
        } => {}
        other => return other,
    }
    for cell in cells.iter_mut() {
        cell.formula_index = 0;
    }
    if let Some(cell) = cells.iter_mut().find(|cell| cell.position == 2) {
        let observed = target.get(&(0, 2)).copied();
        *cell = match observed {
            Some(true) => CellDetail::non_conclusive(
                0,
                2,
                cell.expected,
                observed,
                DetailReason::KnownOriginMismatchUnsupported,
            ),
            Some(false) => CellDetail::non_conclusive(
                0,
                2,
                cell.expected,
                observed,
                DetailReason::KnownOriginMismatchChanged,
            ),
            None => CellDetail::non_conclusive(
                0,
                2,
                cell.expected,
                observed,
                DetailReason::KnownTargetRowMissing,
            ),
        };
    }
    match target.get(&(0, 2)) {
        Some(true) => Replay::Inconclusive("v10_static_known_origin_mismatch_unsupported"),
        Some(false) => Replay::Reject("v10_static_known_origin_mismatch_changed"),
        None => Replay::Inconclusive("v10_known_target_row_missing"),
    }
}

fn safety(target: &BTreeMap<(usize, usize), bool>, cells: &mut Vec<CellDetail>) -> Replay {
    if target.keys().any(|(formula, _)| *formula != 0) {
        return Replay::Reject("v10_unknown_target_formula");
    }
    let proposition = PropositionId(1);
    // The exported monitor is exactly `q`. Its false verdict at position zero
    // is a sound bad-prefix witness for G q. A later true verdict never proves
    // the unbounded safety property, so it is observed but earns no agreement.
    let nodes = [Node::new(NodeKind::Proposition { proposition })];
    let Ok(graph) = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(0), &nodes) else {
        return Replay::Reject("v10_safety_atom_unproved");
    };
    let trace = [vec![], vec![proposition]];
    for (position, expected) in [(0, TruthValue::False), (1, TruthValue::True)] {
        let Ok(tl) = evaluate_prefix_at(
            graph,
            "v10-safety-q",
            &trace,
            "v10-safety-trace",
            false,
            position,
            EvaluationLimits::default(),
        ) else {
            return Replay::Reject("v10_safety_atom_unproved");
        };
        if tl.verdict != expected {
            return Replay::Reject("v10_safety_atom_unproved");
        }
    }
    let Ok(lasso) = OracleLasso::new(
        vec![[(proposition, OracleEvidence::False)].into()],
        vec![[(proposition, OracleEvidence::True)].into()],
    ) else {
        return Replay::Reject("v10_safety_oracle_unproved");
    };
    let Ok(oracle) = evaluate_lasso_oracle(
        &OracleFormula::Globally(
            OracleInterval::Unbounded { start: 0 },
            Box::new(OracleFormula::Atom(proposition)),
        ),
        &[],
        &lasso,
        0,
        OracleLimits::default(),
    ) else {
        return Replay::Reject("v10_safety_oracle_unproved");
    };
    if oracle.verdict != OracleVerdict::Refuted {
        return Replay::Reject("v10_safety_source_oracle_disagreement");
    }
    cells.push(CellDetail::compared(
        0,
        0,
        false,
        target.get(&(0, 0)).copied(),
    ));
    let second = target.get(&(0, 1)).copied();
    cells.push(match second {
        Some(true) => CellDetail::non_conclusive(
            0,
            1,
            Some(true),
            second,
            DetailReason::TargetPassDoesNotProveUnboundedSafety,
        ),
        _ => CellDetail::compared(0, 1, true, second),
    });
    let (Some(first), Some(second)) = (target.get(&(0, 0)), target.get(&(0, 1))) else {
        return Replay::Inconclusive("v10_admitted_target_row_missing");
    };
    if *first || !*second {
        return Replay::Reject("v10_safety_target_semantic_mismatch");
    }
    Replay::Accept {
        admitted_cells: 1,
        unsupported_cells: 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn target(bytes: &[u8]) -> BTreeMap<(usize, usize), bool> {
        super::super::v10_target_rows(bytes).unwrap()
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn unsupported_past_cases_require_their_exact_origin_interval_refusal() {
        for (index, operator, start, end) in [
            (1, PastOperatorKind::Historically, 0, 1),
            (3, PastOperatorKind::Since, 0, 2),
            (4, PastOperatorKind::Triggered, 0, 2),
            (5, PastOperatorKind::Once, 1, 1),
        ] {
            let expected = PastMappingError::TargetOriginIntervalMismatch {
                operator,
                interval: Interval::new(start, end).unwrap(),
            };
            assert!(expected_past_refusal(index, &expected));
            assert!(!expected_past_refusal(
                index,
                &PastMappingError::ResourceIncomplete
            ));
            assert!(!expected_past_refusal(
                index,
                &PastMappingError::TargetOriginMismatch
            ));
            assert!(!expected_past_refusal(0, &expected));
            assert!(!expected_past_refusal(
                index,
                &PastMappingError::TargetOriginIntervalMismatch {
                    operator: PastOperatorKind::StrongPrevious,
                    interval: Interval::new(start, end).unwrap(),
                },
            ));
            assert!(!expected_past_refusal(
                index,
                &PastMappingError::TargetOriginIntervalMismatch {
                    operator,
                    interval: Interval::new(0, 0).unwrap(),
                },
            ));
        }
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn retained_static_runs_preserve_only_supported_agreement_and_known_mismatch() {
        assert_eq!(
            replay(
                "bounded",
                &target(include_bytes!("../../../corpus/r2u2-v4.2/r2u2.stdout"))
            ),
            Replay::Accept {
                admitted_cells: 8,
                unsupported_cells: 0
            }
        );
        assert_eq!(
            replay(
                "past",
                &target(include_bytes!(
                    "../../../corpus/past-c2po-v1/target-4.2/r2u2.stdout"
                ))
            ),
            Replay::Accept {
                admitted_cells: 6,
                unsupported_cells: 12
            }
        );
        assert_eq!(
            replay(
                "unsafe-since",
                &target(include_bytes!(
                    "../../../corpus/past-c2po-v1/target-4.2/unsafe-since.stdout"
                ))
            ),
            Replay::Inconclusive("v10_static_known_origin_mismatch_unsupported")
        );
        assert_eq!(
            replay("safety", &target(b"0:0,F\n0:1,T\n")),
            Replay::Accept {
                admitted_cells: 1,
                unsupported_cells: 1
            }
        );
    }

    // Trace: FR-052-AC-1, FR-052-AC-2, TC-191, TC-192.
    #[test]
    fn static_cell_details_cover_exact_cases_and_non_proving_safety_pass() {
        let (verdict, bounded) = replay_detailed(
            "bounded",
            &target(include_bytes!("../../../corpus/r2u2-v4.2/r2u2.stdout")),
        );
        assert!(matches!(verdict, Replay::Accept { .. }));
        assert_eq!(bounded.len(), 8);
        assert!(bounded
            .iter()
            .all(|cell| cell.comparison_class == ComparisonClass::Agreement));

        let (verdict, past) = replay_detailed(
            "past",
            &target(include_bytes!(
                "../../../corpus/past-c2po-v1/target-4.2/r2u2.stdout"
            )),
        );
        assert!(matches!(verdict, Replay::Accept { .. }));
        assert_eq!(past.len(), 18);
        assert_eq!(
            past.iter()
                .filter(|cell| cell.comparison_class == ComparisonClass::Agreement)
                .count(),
            6
        );
        assert_eq!(
            past.iter()
                .filter(|cell| cell.comparison_class == ComparisonClass::UnsupportedMapping)
                .count(),
            12
        );
        assert!(past
            .iter()
            .filter(|cell| cell.comparison_class == ComparisonClass::UnsupportedMapping)
            .all(|cell| matches!(
                cell.refusal,
                Some(RefusalDetail::TargetOriginIntervalMismatch { .. })
            )));
        let refused = past
            .iter()
            .find(|cell| cell.formula_index == 1 && cell.position == 0)
            .unwrap();
        assert_eq!(
            serde_json::to_value(refused).unwrap()["refusal"],
            serde_json::json!({
                "kind": "target_origin_interval_mismatch",
                "operator": "historically",
                "start": 0,
                "end": 1
            })
        );

        let (verdict, safety) = replay_detailed("safety", &target(b"0:0,F\n0:1,T\n"));
        assert!(matches!(verdict, Replay::Accept { .. }));
        assert_eq!(safety.len(), 2);
        assert_eq!(safety[0].comparison_class, ComparisonClass::Agreement);
        assert_eq!(safety[1].comparison_class, ComparisonClass::NonConclusive);

        let (verdict, unsafe_cells) = replay_detailed(
            "unsafe-since",
            &target(include_bytes!(
                "../../../corpus/past-c2po-v1/target-4.2/unsafe-since.stdout"
            )),
        );
        assert_eq!(
            verdict,
            Replay::Inconclusive("v10_static_known_origin_mismatch_unsupported")
        );
        assert_eq!(unsafe_cells.len(), 3);
        assert_eq!(
            unsafe_cells
                .iter()
                .filter(|cell| cell.comparison_class == ComparisonClass::NonConclusive)
                .count(),
            1
        );
        let drift = unsafe_cells.iter().find(|cell| cell.position == 2).unwrap();
        assert_eq!(drift.expected, Some(false));
        assert_eq!(drift.observed, Some(true));
        assert_eq!(
            drift.reason,
            Some(DetailReason::KnownOriginMismatchUnsupported)
        );
    }

    // Trace: FR-052-AC-1, TC-191.
    #[test]
    fn static_cell_details_preserve_mismatch_and_missing_rows() {
        let mut bounded = target(include_bytes!("../../../corpus/r2u2-v4.2/r2u2.stdout"));
        let key = *bounded.keys().next().unwrap();
        bounded.insert(key, !bounded[&key]);
        let (verdict, cells) = replay_detailed("bounded", &bounded);
        assert_eq!(verdict, Replay::Reject("v10_target_semantic_mismatch"));
        assert_eq!(
            cells
                .iter()
                .filter(|cell| cell.comparison_class == ComparisonClass::SemanticMismatch)
                .count(),
            1
        );
        bounded.remove(&key);
        let (verdict, cells) = replay_detailed("bounded", &bounded);
        assert_eq!(
            verdict,
            Replay::Inconclusive("v10_admitted_target_row_missing")
        );
        assert_eq!(
            cells
                .iter()
                .filter(
                    |cell| cell.comparison_class == ComparisonClass::NonConclusive
                        && cell.reason == Some(DetailReason::AdmittedTargetRowMissing)
                )
                .count(),
            1
        );

        let mut past = target(include_bytes!(
            "../../../corpus/past-c2po-v1/target-4.2/r2u2.stdout"
        ));
        let admitted = (0, 0);
        past.insert(admitted, !past[&admitted]);
        let (verdict, cells) = replay_detailed("past", &past);
        assert_eq!(verdict, Replay::Reject("v10_target_semantic_mismatch"));
        assert_eq!(cells.len(), 18);
        past.remove(&admitted);
        let (verdict, cells) = replay_detailed("past", &past);
        assert_eq!(
            verdict,
            Replay::Inconclusive("v10_admitted_target_row_missing")
        );
        assert_eq!(cells.len(), 18);
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn static_target_faults_cannot_gain_agreement() {
        let mut bounded = target(include_bytes!("../../../corpus/r2u2-v4.2/r2u2.stdout"));
        bounded.insert((1, 0), true);
        assert_eq!(
            replay("bounded", &bounded),
            Replay::Reject("v10_target_semantic_mismatch")
        );
        bounded.remove(&(1, 0));
        assert_eq!(
            replay("bounded", &bounded),
            Replay::Inconclusive("v10_admitted_target_row_missing")
        );

        let mut past = target(include_bytes!(
            "../../../corpus/past-c2po-v1/target-4.2/r2u2.stdout"
        ));
        past.insert((0, 0), true);
        assert_eq!(
            replay("past", &past),
            Replay::Reject("v10_target_semantic_mismatch")
        );
        past.insert((0, 0), false);
        past.insert((1, 0), true);
        assert_eq!(
            replay("past", &past),
            Replay::Accept {
                admitted_cells: 6,
                unsupported_cells: 12
            }
        );

        let mut unsafe_rows = target(include_bytes!(
            "../../../corpus/past-c2po-v1/target-4.2/unsafe-since.stdout"
        ));
        unsafe_rows.insert((0, 2), false);
        let (verdict, cells) = replay_detailed("unsafe-since", &unsafe_rows);
        assert_eq!(
            verdict,
            Replay::Reject("v10_static_known_origin_mismatch_changed")
        );
        let drift = cells.iter().find(|cell| cell.position == 2).unwrap();
        assert_eq!(drift.comparison_class, ComparisonClass::NonConclusive);
        assert_eq!(drift.expected, Some(false));
        assert_eq!(drift.observed, Some(false));
        assert_eq!(drift.reason, Some(DetailReason::KnownOriginMismatchChanged));
        assert!(drift.refusal.is_none());

        let mut safety_rows = target(b"0:0,F\n0:1,T\n");
        safety_rows.insert((0, 0), true);
        assert_eq!(
            replay("safety", &safety_rows),
            Replay::Reject("v10_safety_target_semantic_mismatch")
        );
        safety_rows.remove(&(0, 1));
        assert_eq!(
            replay("safety", &safety_rows),
            Replay::Inconclusive("v10_admitted_target_row_missing")
        );
    }
}
