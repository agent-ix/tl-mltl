//! Independent, source-bound replay of the 21 generated TL-217 past-grid runs.
//! Trace: FR-052-AC-1, FR-055-AC-2, TC-191, TC-198.

use std::collections::BTreeMap;

use serde::Serialize;
use sha2::{Digest, Sha256};
use tl_mltl::{
    evaluate_past, map_past_to_c2po, ClockBinding, MappingSourceIdentity, MappingSourceState,
    PastEvaluationLimits, PastEvaluationRelationInput, PastMappingError, PositionHistoryDocument,
    PositionObservation, TargetOriginContract,
};
use tl_oracle::{
    evaluate_origin_complete, Formula as OracleFormula, Interval as OracleInterval,
    Limits as OracleLimits,
};
use tl_syntax::{
    Formula, FormulaDocument, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration,
    PastOperatorKind, PropositionBinding, PropositionId, SemanticProfile, SignalCatalogDocument,
    SignalDomain, SignalId,
};

const STEPS: usize = 6;
const INTERVALS: [(&str, u32, u32); 6] = [
    ("zero-singleton", 0, 0),
    ("zero-unit", 0, 1),
    ("zero-upper", 0, 2),
    ("nonzero-singleton-one", 1, 1),
    ("nonzero-range", 1, 2),
    ("nonzero-singleton", 2, 2),
];
const TRACES: [&str; 3] = ["all-true", "all-false", "boundary-toggle"];

// Trace: FR-052-AC-1, FR-052-AC-2, TC-191, TC-192.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CellDetail {
    pub(super) formula_index: usize,
    pub(super) position: usize,
    pub(super) comparison_class: &'static str,
    pub(super) expected: Option<bool>,
    pub(super) observed: Option<bool>,
    pub(super) refusal: Option<String>,
}

impl CellDetail {
    pub(super) fn compared(
        formula_index: usize,
        position: usize,
        expected: bool,
        observed: Option<bool>,
    ) -> Self {
        let comparison_class = match observed {
            Some(value) if value == expected => "agreement",
            Some(_) => "semantic_mismatch",
            None => "unavailable_target",
        };
        Self {
            formula_index,
            position,
            comparison_class,
            expected: Some(expected),
            observed,
            refusal: None,
        }
    }

    pub(super) fn unsupported(
        formula_index: usize,
        position: usize,
        expected: bool,
        observed: Option<bool>,
        refusal: String,
    ) -> Self {
        Self {
            formula_index,
            position,
            comparison_class: "unsupported_mapping",
            expected: Some(expected),
            observed,
            refusal: Some(refusal),
        }
    }

    pub(super) fn non_conclusive(
        formula_index: usize,
        position: usize,
        expected: Option<bool>,
        observed: Option<bool>,
        refusal: &str,
    ) -> Self {
        Self {
            formula_index,
            position,
            comparison_class: "non_conclusive",
            expected,
            observed,
            refusal: Some(refusal.into()),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Operator {
    Once,
    Historically,
    Since,
    Triggered,
    Previous,
}

impl Operator {
    const INTERVALLED: [Self; 4] = [Self::Once, Self::Historically, Self::Since, Self::Triggered];
}

struct Case {
    operator: Operator,
    interval: Option<(u32, u32)>,
    depth: usize,
    expression: String,
    nodes: Vec<Node>,
    oracle: OracleFormula,
}

fn expected_refusal(case: &Case, error: &PastMappingError) -> bool {
    match (case.operator, case.interval, case.depth, error) {
        (Operator::Previous, None, 3, PastMappingError::TargetOriginShapeUnverified(node)) => {
            *node == NodeId(4)
        }
        (
            operator,
            Some((start, end)),
            _,
            PastMappingError::TargetOriginIntervalMismatch {
                operator: refused_operator,
                interval,
            },
        ) => {
            let expected_operator = match operator {
                Operator::Once => PastOperatorKind::Once,
                Operator::Historically => PastOperatorKind::Historically,
                Operator::Since => PastOperatorKind::Since,
                Operator::Triggered => PastOperatorKind::Triggered,
                Operator::Previous => return false,
            };
            *refused_operator == expected_operator
                && *interval == Interval::new(start, end).expect("fixed interval")
        }
        _ => false,
    }
}

fn make_case(operator: Operator, interval: Option<(u32, u32)>, depth: usize) -> Case {
    let mut nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(1),
        }),
    ];
    let mut current = NodeId(0);
    let mut expression = "p".to_owned();
    let mut oracle = OracleFormula::Atom(PropositionId(0));
    for _ in 0..depth {
        let range = interval.map(|(a, b)| Interval::new(a, b).expect("fixed interval"));
        let oracle_range = interval.map(|(a, b)| OracleInterval::Closed {
            start: a as usize,
            end: b as usize,
        });
        let node = match operator {
            Operator::Once => {
                let (a, b) = interval.expect("once interval");
                expression = format!("O[{a},{b}]({expression})");
                oracle = OracleFormula::Once(oracle_range.expect("once range"), Box::new(oracle));
                NodeKind::Once {
                    interval: range.expect("once range"),
                    operand: current,
                }
            }
            Operator::Historically => {
                let (a, b) = interval.expect("historically interval");
                expression = format!("H[{a},{b}]({expression})");
                oracle = OracleFormula::Historically(
                    oracle_range.expect("historically range"),
                    Box::new(oracle),
                );
                NodeKind::Historically {
                    interval: range.expect("historically range"),
                    operand: current,
                }
            }
            Operator::Since => {
                let (a, b) = interval.expect("since interval");
                expression = format!("({expression} S[{a},{b}] q)");
                oracle = OracleFormula::Since(
                    oracle_range.expect("since range"),
                    Box::new(oracle),
                    Box::new(OracleFormula::Atom(PropositionId(1))),
                );
                NodeKind::Since {
                    interval: range.expect("since range"),
                    left: current,
                    right: NodeId(1),
                }
            }
            Operator::Triggered => {
                let (a, b) = interval.expect("triggered interval");
                expression = format!("(!((!{expression}) S[{a},{b}] (!q)))");
                oracle = OracleFormula::Triggered(
                    oracle_range.expect("triggered range"),
                    Box::new(oracle),
                    Box::new(OracleFormula::Atom(PropositionId(1))),
                );
                NodeKind::Triggered {
                    interval: range.expect("triggered range"),
                    left: current,
                    right: NodeId(1),
                }
            }
            Operator::Previous => {
                expression = format!("O[1,1]({expression})");
                oracle = OracleFormula::StrongPrevious(Box::new(oracle));
                NodeKind::StrongPrevious { operand: current }
            }
        };
        nodes.push(Node::new(node));
        current = NodeId(u32::try_from(nodes.len() - 1).expect("fixed node count"));
    }
    Case {
        operator,
        interval,
        depth,
        expression,
        nodes,
        oracle,
    }
}

pub(super) struct Run {
    cases: Vec<Case>,
    observations: Vec<(bool, bool)>,
    pub(super) spec: Vec<u8>,
    pub(super) trace: Vec<u8>,
}

impl Run {
    pub(super) fn for_member(case_id: &str) -> Option<Self> {
        let (group, trace_kind) = TRACES.iter().find_map(|kind| {
            case_id
                .strip_suffix(&format!("-{kind}"))
                .map(|group| (group, *kind))
        })?;
        let (cases, boundary) = if group == "previous" {
            (
                (1..=3)
                    .map(|depth| make_case(Operator::Previous, None, depth))
                    .collect::<Vec<_>>(),
                1,
            )
        } else {
            let (_, a, b) = INTERVALS.iter().find(|(name, _, _)| *name == group)?;
            (
                Operator::INTERVALLED
                    .into_iter()
                    .flat_map(|operator| {
                        (1..=3).map(move |depth| make_case(operator, Some((*a, *b)), depth))
                    })
                    .collect::<Vec<_>>(),
                *b as usize,
            )
        };
        let observations = (0..STEPS)
            .map(|position| match trace_kind {
                "all-true" => (true, true),
                "all-false" => (false, false),
                "boundary-toggle" => {
                    let p = position >= boundary;
                    (p, !p)
                }
                _ => unreachable!("fixed trace inventory"),
            })
            .collect::<Vec<_>>();
        let spec = format!(
            "INPUT\n p,q: bool;\nPTSPEC\n{}\n",
            cases
                .iter()
                .map(|case: &Case| format!(" {};", case.expression))
                .collect::<Vec<_>>()
                .join("\n")
        )
        .into_bytes();
        let mut trace = String::from("# p,q\n");
        for (p, q) in &observations {
            trace.push_str(&format!("{},{}\n", u8::from(*p), u8::from(*q)));
        }
        Some(Self {
            cases,
            observations,
            spec,
            trace: trace.into_bytes(),
        })
    }

    #[cfg(test)]
    pub(super) fn replay(&self, target: &BTreeMap<(usize, usize), bool>) -> Replay {
        self.replay_detailed(target).0
    }

    pub(super) fn replay_detailed(
        &self,
        target: &BTreeMap<(usize, usize), bool>,
    ) -> (Replay, Vec<CellDetail>) {
        let mut cells = Vec::new();
        let verdict = self.replay_collect(target, &mut cells);
        (verdict, cells)
    }

    fn replay_collect(
        &self,
        target: &BTreeMap<(usize, usize), bool>,
        cells: &mut Vec<CellDetail>,
    ) -> Replay {
        if target
            .keys()
            .any(|(formula, _)| *formula >= self.cases.len())
        {
            return Replay::Reject("v10_unknown_target_formula");
        }
        let observations = self
            .observations
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
            .collect();
        let Ok(history) = PositionHistoryDocument::new(
            "tl217-grid",
            1,
            0,
            (STEPS - 1) as u64,
            Some(ClockBinding::EventPosition),
            observations,
        ) else {
            return Replay::Reject("v10_source_history_unproved");
        };
        let word = self
            .observations
            .iter()
            .map(|(p, q)| [(PropositionId(0), *p), (PropositionId(1), *q)].into())
            .collect::<Vec<BTreeMap<_, _>>>();
        let Ok(catalog) = SignalCatalogDocument::new(
            vec![
                OwnedSignalDeclaration::new(SignalId(1), "p".to_owned(), SignalDomain::Boolean),
                OwnedSignalDeclaration::new(SignalId(2), "q".to_owned(), SignalDomain::Boolean),
            ],
            vec![
                PropositionBinding::new(PropositionId(0), SignalId(1)),
                PropositionBinding::new(PropositionId(1), SignalId(2)),
            ],
        ) else {
            return Replay::Reject("v10_source_catalog_unproved");
        };
        let mut admitted_cells = 0;
        let mut unsupported_cells = 0;
        let mut missing = false;
        let mut mismatch = false;
        for (formula_id, case) in self.cases.iter().enumerate() {
            let Ok(graph) = Formula::new(
                SemanticProfile::OriginCompleteHistoryV1,
                NodeId(u32::try_from(case.nodes.len() - 1).expect("fixed node count")),
                &case.nodes,
            ) else {
                return Replay::Reject("v10_source_formula_unproved");
            };
            let expected_admitted = expected_admission(case);
            let Ok(document) = FormulaDocument::from_formula_v2(graph) else {
                return Replay::Reject("v10_source_document_unproved");
            };
            let Ok(document_bytes) = document.canonical_json_bytes() else {
                return Replay::Reject("v10_source_document_unproved");
            };
            let mapping = map_past_to_c2po(
                graph,
                "tl217-grid",
                &document_bytes,
                MappingSourceIdentity {
                    revision: "tl217-grid".to_owned(),
                    state: MappingSourceState::Clean,
                },
                &catalog,
                &TargetOriginContract::reviewed_r2u2_4_2(),
                100,
            );
            let refusal = match (expected_admitted, mapping) {
                (true, Ok(mapped))
                    if mapped.expression == case.expression
                        && mapped.output_sha256 == sha256(case.expression.as_bytes()) =>
                {
                    None
                }
                (false, Err(refusal)) if expected_refusal(case, &refusal) => {
                    Some(format!("{refusal:?}"))
                }
                _ => return Replay::Reject("v10_mapping_partition_unproved"),
            };
            for position in 0..STEPS {
                let Ok(tl) = evaluate_past(
                    graph,
                    "tl217-grid",
                    &history,
                    position as u64,
                    "map",
                    1,
                    PastEvaluationRelationInput::Original,
                    PastEvaluationLimits::default(),
                ) else {
                    return Replay::Reject("v10_source_evaluation_unproved");
                };
                let Ok(oracle) = evaluate_origin_complete(
                    &case.oracle,
                    &word,
                    position,
                    OracleLimits::default(),
                ) else {
                    return Replay::Reject("v10_oracle_evaluation_unproved");
                };
                if tl.verdict != oracle {
                    return Replay::Reject("v10_source_oracle_disagreement");
                }
                if expected_admitted {
                    admitted_cells += 1;
                    cells.push(CellDetail::compared(
                        formula_id,
                        position,
                        tl.verdict,
                        target.get(&(formula_id, position)).copied(),
                    ));
                    match target.get(&(formula_id, position)) {
                        Some(observed) if *observed == tl.verdict => {}
                        Some(_) => mismatch = true,
                        None => missing = true,
                    }
                } else {
                    unsupported_cells += 1;
                    cells.push(CellDetail::unsupported(
                        formula_id,
                        position,
                        tl.verdict,
                        target.get(&(formula_id, position)).copied(),
                        refusal
                            .clone()
                            .expect("validated unsupported mapping has refusal"),
                    ));
                }
            }
        }
        // The reviewed partition has 20 admitted shapes and 55 refused shapes,
        // repeated over three traces and six input positions. Per-run counts
        // prevent a silent reduction of mapped target obligations.
        let expected_admitted = self
            .cases
            .iter()
            .filter(|case| expected_admission(case))
            .count()
            * STEPS;
        if admitted_cells != expected_admitted
            || admitted_cells + unsupported_cells != self.cases.len() * STEPS
        {
            return Replay::Reject("v10_grid_population_unproved");
        }
        if mismatch {
            Replay::Reject("v10_target_semantic_mismatch")
        } else if missing {
            Replay::Inconclusive("v10_admitted_target_row_missing")
        } else {
            Replay::Accept {
                admitted_cells,
                unsupported_cells,
            }
        }
    }
}

fn expected_admission(case: &Case) -> bool {
    match case.operator {
        Operator::Previous => case.depth <= 2,
        Operator::Once | Operator::Since => {
            matches!(case.interval, Some((0, 0) | (0, 1)))
        }
        Operator::Historically | Operator::Triggered => case.interval == Some((0, 0)),
    }
}

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum Replay {
    Accept {
        admitted_cells: usize,
        unsupported_cells: usize,
    },
    Reject(&'static str),
    Inconclusive(&'static str),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oracle_rows(run: &Run) -> BTreeMap<(usize, usize), bool> {
        let word = run
            .observations
            .iter()
            .map(|(p, q)| [(PropositionId(0), *p), (PropositionId(1), *q)].into())
            .collect::<Vec<BTreeMap<_, _>>>();
        let mut rows = BTreeMap::new();
        for (formula, case) in run.cases.iter().enumerate() {
            for position in 0..STEPS {
                rows.insert(
                    (formula, position),
                    evaluate_origin_complete(
                        &case.oracle,
                        &word,
                        position,
                        OracleLimits::default(),
                    )
                    .unwrap(),
                );
            }
        }
        rows
    }

    // Trace: FR-052-AC-1, TC-191
    #[test]
    fn all_generated_cases_have_reviewed_population_and_exact_inputs() {
        let mut admitted = 0;
        let mut unsupported = 0;
        for group in INTERVALS
            .iter()
            .map(|(name, _, _)| *name)
            .chain(std::iter::once("previous"))
        {
            for trace in TRACES {
                let run = Run::for_member(&format!("{group}-{trace}")).unwrap();
                let count = run.cases.len() * STEPS;
                admitted += run
                    .cases
                    .iter()
                    .filter(|case| expected_admission(case))
                    .count()
                    * STEPS;
                unsupported += count
                    - run
                        .cases
                        .iter()
                        .filter(|case| expected_admission(case))
                        .count()
                        * STEPS;
                assert!(matches!(run.cases.len(), 3 | 12));
                assert_eq!(run.observations.len(), STEPS);
                assert!(run.spec.starts_with(b"INPUT\n p,q: bool;\nPTSPEC\n"));
                assert!(run.trace.starts_with(b"# p,q\n"));
            }
        }
        assert_eq!((admitted, unsupported), (360, 990));
        assert!(Run::for_member("safety").is_none());
        assert!(Run::for_member("bounded").is_none());
    }

    // Trace: FR-052-AC-1, TC-191
    #[test]
    fn reviewed_input_bytes_match_the_pinned_generator_census() {
        let mut runs = BTreeMap::new();
        for group in INTERVALS
            .iter()
            .map(|(name, _, _)| *name)
            .chain(std::iter::once("previous"))
        {
            for trace in TRACES {
                let id = format!("{group}-{trace}");
                runs.insert(id.clone(), Run::for_member(&id).unwrap());
            }
        }
        assert_eq!(runs.len(), 21);
        let mut digest = Sha256::new();
        for (id, run) in &runs {
            for (extension, bytes) in [("c2po", &run.spec), ("csv", &run.trace)] {
                digest.update(id.as_bytes());
                digest.update([0]);
                digest.update(extension.as_bytes());
                digest.update([0]);
                digest.update(bytes);
                digest.update([0]);
            }
        }
        assert_eq!(
            format!("{:x}", digest.finalize()),
            "c5c8b0b6f93842ab6ba787c9087a2d5db13f6d2b83e2bbebd19810c8dc4e2da6"
        );
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn every_generated_run_replays_source_oracle_mapping_and_target() {
        let mut admitted = 0;
        let mut unsupported = 0;
        for group in INTERVALS
            .iter()
            .map(|(name, _, _)| *name)
            .chain(std::iter::once("previous"))
        {
            for trace in TRACES {
                let run = Run::for_member(&format!("{group}-{trace}")).unwrap();
                match run.replay(&oracle_rows(&run)) {
                    Replay::Accept {
                        admitted_cells,
                        unsupported_cells,
                    } => {
                        admitted += admitted_cells;
                        unsupported += unsupported_cells;
                    }
                    other => panic!("{group}-{trace}: {other:?}"),
                }
            }
        }
        assert_eq!((admitted, unsupported), (360, 990));
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn unsupported_grid_cells_require_exact_origin_refusals() {
        let mut refused_cells = 0;
        for group in INTERVALS
            .iter()
            .map(|(name, _, _)| *name)
            .chain(std::iter::once("previous"))
        {
            for trace in TRACES {
                let run = Run::for_member(&format!("{group}-{trace}")).unwrap();
                for case in run.cases.iter().filter(|case| !expected_admission(case)) {
                    refused_cells += STEPS;
                    let expected = match (case.operator, case.interval, case.depth) {
                        (Operator::Previous, None, 3) => {
                            PastMappingError::TargetOriginShapeUnverified(NodeId(4))
                        }
                        (operator, Some((start, end)), _) => {
                            let operator = match operator {
                                Operator::Once => PastOperatorKind::Once,
                                Operator::Historically => PastOperatorKind::Historically,
                                Operator::Since => PastOperatorKind::Since,
                                Operator::Triggered => PastOperatorKind::Triggered,
                                Operator::Previous => unreachable!("previous has no interval"),
                            };
                            PastMappingError::TargetOriginIntervalMismatch {
                                operator,
                                interval: Interval::new(start, end).unwrap(),
                            }
                        }
                        _ => panic!("unexpected unsupported shape"),
                    };
                    assert!(expected_refusal(case, &expected), "{group}-{trace}");
                    assert!(!expected_refusal(
                        case,
                        &PastMappingError::ResourceIncomplete
                    ));
                    assert!(!expected_refusal(
                        case,
                        &PastMappingError::TargetOriginMismatch
                    ));
                    assert!(!expected_refusal(
                        case,
                        &PastMappingError::TargetOriginShapeUnverified(NodeId(0))
                    ));
                    if let PastMappingError::TargetOriginIntervalMismatch { operator, .. } =
                        expected
                    {
                        assert!(!expected_refusal(
                            case,
                            &PastMappingError::TargetOriginIntervalMismatch {
                                operator,
                                interval: Interval::new(9, 9).unwrap(),
                            }
                        ));
                    }
                }
            }
        }
        assert_eq!(refused_cells, 990);
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn admitted_target_faults_reject_or_remain_inconclusive() {
        let run = Run::for_member("zero-unit-boundary-toggle").unwrap();
        let mut rows = oracle_rows(&run);
        assert!(matches!(run.replay(&rows), Replay::Accept { .. }));
        let admitted = run.cases.iter().position(expected_admission).unwrap();
        let unsupported = run
            .cases
            .iter()
            .position(|case| !expected_admission(case))
            .unwrap();
        let key = (admitted, 0);
        rows.insert(key, !rows[&key]);
        assert_eq!(
            run.replay(&rows),
            Replay::Reject("v10_target_semantic_mismatch")
        );
        rows.remove(&key);
        assert_eq!(
            run.replay(&rows),
            Replay::Inconclusive("v10_admitted_target_row_missing")
        );
        rows = oracle_rows(&run);
        let unsupported_key = (unsupported, 0);
        rows.insert(unsupported_key, !rows[&unsupported_key]);
        assert!(matches!(run.replay(&rows), Replay::Accept { .. }));
        rows.remove(&unsupported_key);
        assert!(matches!(run.replay(&rows), Replay::Accept { .. }));
    }

    // Trace: FR-052-AC-1, TC-191.
    #[test]
    fn generated_cell_details_cover_exact_population_and_target_faults() {
        let run = Run::for_member("zero-unit-boundary-toggle").unwrap();
        let mut rows = oracle_rows(&run);
        let (verdict, cells) = run.replay_detailed(&rows);
        assert!(matches!(verdict, Replay::Accept { .. }));
        assert_eq!(cells.len(), run.cases.len() * STEPS);
        assert_eq!(
            cells
                .iter()
                .filter(|cell| cell.comparison_class == "agreement")
                .count(),
            run.cases
                .iter()
                .filter(|case| expected_admission(case))
                .count()
                * STEPS
        );
        assert_eq!(
            cells
                .iter()
                .filter(|cell| cell.comparison_class == "unsupported_mapping")
                .count(),
            run.cases
                .iter()
                .filter(|case| !expected_admission(case))
                .count()
                * STEPS
        );
        assert!(cells
            .iter()
            .filter(|cell| cell.comparison_class == "unsupported_mapping")
            .all(|cell| cell.refusal.is_some()));
        let admitted = cells
            .iter()
            .find(|cell| cell.comparison_class == "agreement")
            .unwrap();
        let key = (admitted.formula_index, admitted.position);
        rows.insert(key, !rows[&key]);
        let (verdict, cells) = run.replay_detailed(&rows);
        assert_eq!(verdict, Replay::Reject("v10_target_semantic_mismatch"));
        assert_eq!(
            cells
                .iter()
                .filter(|cell| cell.comparison_class == "semantic_mismatch")
                .count(),
            1
        );
        rows.remove(&key);
        let (verdict, cells) = run.replay_detailed(&rows);
        assert_eq!(
            verdict,
            Replay::Inconclusive("v10_admitted_target_row_missing")
        );
        assert_eq!(
            cells
                .iter()
                .filter(|cell| cell.comparison_class == "unavailable_target")
                .count(),
            1
        );
    }

    // Trace: FR-055-AC-2, TC-198
    #[test]
    fn target_extra_positions_do_not_supply_missing_input_rows() {
        let run = Run::for_member("previous-all-true").unwrap();
        let mut rows = oracle_rows(&run);
        rows.insert((0, STEPS), true);
        assert!(matches!(run.replay(&rows), Replay::Accept { .. }));
        rows.remove(&(0, 0));
        assert_eq!(
            run.replay(&rows),
            Replay::Inconclusive("v10_admitted_target_row_missing")
        );
        rows.insert((run.cases.len(), 0), true);
        assert_eq!(
            run.replay(&rows),
            Replay::Reject("v10_unknown_target_formula")
        );
    }
}
