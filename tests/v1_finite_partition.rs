//! A completed depth-one finite/past partition for the V1 campaign.
//! Scope: leaves {false,true,p0}; all one-level Boolean/future/past roots;
//! closed intervals 0 <= a <= b <= 2; one-atom words of length 1..=3.
//! Binary operands are ordered and no symmetry reductions are applied. The
//! grammar includes only operators supported by their applicable profile, so
//! this partition declares no cross-profile cells and records zero refusals.

use std::collections::BTreeMap;

use tl_mltl::{
    evaluate_closed_at, evaluate_past, ClockBinding, EvaluationLimits, PastEvaluationLimits,
    PastEvaluationRelationInput, PositionHistoryDocument, PositionObservation, TruthValue,
};
use tl_oracle::{
    evaluate_closed_trace_v1, evaluate_origin_complete, Formula as OracleFormula,
    Interval as OracleInterval, Limits,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const LEAVES: u64 = 3;
const CLOSED_INTERVALS: u64 = 6; // sum_{b=0}^2 (b + 1)
const BOOL_PROFILES: u64 = 2;
const FORMULAS: u64 = BOOL_PROFILES * (LEAVES + LEAVES + 4 * LEAVES * LEAVES)
    + 2 * (2 * CLOSED_INTERVALS * LEAVES + 2 * CLOSED_INTERVALS * LEAVES * LEAVES)
    + LEAVES;
const WORD_POSITIONS: u64 = 2 + 2 * 4 + 3 * 8;
const DECLARED: u64 = FORMULAS * WORD_POSITIONS;
const SCOPE: &str = "depth1_atom1_closed0_2_words1_3";
const EXTENDED_FORMULAS: u64 = 807;
const EXTENDED_WORD_POSITIONS: u64 = 642;
const EXTENDED_DECLARED: u64 = EXTENDED_FORMULAS * EXTENDED_WORD_POSITIONS;
const FULL_CLOSED_FORMULAS: u64 = 1_031_120_211_193_068;
const FULL_PAST_FORMULAS: u64 = 1_062_364_497_622_965;
const FULL_DECLARED: u64 = (FULL_CLOSED_FORMULAS + FULL_PAST_FORMULAS) * EXTENDED_WORD_POSITIONS;

#[derive(Clone, Copy)]
enum Leaf {
    False,
    True,
    P0,
}

impl Leaf {
    fn node(self) -> Node {
        let kind = match self {
            Self::False => NodeKind::False,
            Self::True => NodeKind::True,
            Self::P0 => NodeKind::Proposition {
                proposition: PropositionId(0),
            },
        };
        Node::new(kind)
    }

    fn oracle(self) -> OracleFormula {
        match self {
            Self::False => OracleFormula::False,
            Self::True => OracleFormula::True,
            Self::P0 => OracleFormula::Atom(PropositionId(0)),
        }
    }
}

struct Case {
    profile: SemanticProfile,
    nodes: Vec<Node>,
    oracle: OracleFormula,
}

fn unary(profile: SemanticProfile, leaf: Leaf, kind: NodeKind, oracle: OracleFormula) -> Case {
    Case {
        profile,
        nodes: vec![leaf.node(), Node::new(kind)],
        oracle,
    }
}

fn binary(
    profile: SemanticProfile,
    left: Leaf,
    right: Leaf,
    kind: NodeKind,
    oracle: OracleFormula,
) -> Case {
    Case {
        profile,
        nodes: vec![left.node(), right.node(), Node::new(kind)],
        oracle,
    }
}

fn cases(max_interval: u32) -> Vec<Case> {
    use OracleFormula as O;
    let leaves = [Leaf::False, Leaf::True, Leaf::P0];
    let mut out = Vec::new();
    for profile in [
        SemanticProfile::ClosedTraceV1,
        SemanticProfile::OriginCompleteHistoryV1,
    ] {
        for left in leaves {
            out.push(Case {
                profile,
                nodes: vec![left.node()],
                oracle: left.oracle(),
            });
            out.push(unary(
                profile,
                left,
                NodeKind::Not { operand: NodeId(0) },
                O::Not(Box::new(left.oracle())),
            ));
            for right in leaves {
                let p = || Box::new(left.oracle());
                let q = || Box::new(right.oracle());
                out.push(binary(
                    profile,
                    left,
                    right,
                    NodeKind::And {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    O::And(p(), q()),
                ));
                out.push(binary(
                    profile,
                    left,
                    right,
                    NodeKind::Or {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    O::Or(p(), q()),
                ));
                out.push(binary(
                    profile,
                    left,
                    right,
                    NodeKind::Implies {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    O::Implies(p(), q()),
                ));
                out.push(binary(
                    profile,
                    left,
                    right,
                    NodeKind::Equivalent {
                        left: NodeId(0),
                        right: NodeId(1),
                    },
                    O::Equivalent(p(), q()),
                ));
            }
        }
    }
    for end in 0..=max_interval {
        for start in 0..=end {
            let interval = Interval::new(start, end).unwrap();
            let oracle_interval = OracleInterval::Closed {
                start: usize::try_from(start).unwrap(),
                end: usize::try_from(end).unwrap(),
            };
            for left in leaves {
                let p = || Box::new(left.oracle());
                out.push(unary(
                    SemanticProfile::ClosedTraceV1,
                    left,
                    NodeKind::Future {
                        interval,
                        operand: NodeId(0),
                    },
                    O::Future(oracle_interval, p()),
                ));
                out.push(unary(
                    SemanticProfile::ClosedTraceV1,
                    left,
                    NodeKind::Globally {
                        interval,
                        operand: NodeId(0),
                    },
                    O::Globally(oracle_interval, p()),
                ));
                out.push(unary(
                    SemanticProfile::OriginCompleteHistoryV1,
                    left,
                    NodeKind::Once {
                        interval,
                        operand: NodeId(0),
                    },
                    O::Once(oracle_interval, p()),
                ));
                out.push(unary(
                    SemanticProfile::OriginCompleteHistoryV1,
                    left,
                    NodeKind::Historically {
                        interval,
                        operand: NodeId(0),
                    },
                    O::Historically(oracle_interval, p()),
                ));
                for right in leaves {
                    let q = || Box::new(right.oracle());
                    out.push(binary(
                        SemanticProfile::ClosedTraceV1,
                        left,
                        right,
                        NodeKind::Until {
                            interval,
                            left: NodeId(0),
                            right: NodeId(1),
                        },
                        O::Until(oracle_interval, p(), q()),
                    ));
                    out.push(binary(
                        SemanticProfile::ClosedTraceV1,
                        left,
                        right,
                        NodeKind::Release {
                            interval,
                            left: NodeId(0),
                            right: NodeId(1),
                        },
                        O::Release(oracle_interval, p(), q()),
                    ));
                    out.push(binary(
                        SemanticProfile::OriginCompleteHistoryV1,
                        left,
                        right,
                        NodeKind::Since {
                            interval,
                            left: NodeId(0),
                            right: NodeId(1),
                        },
                        O::Since(oracle_interval, p(), q()),
                    ));
                    out.push(binary(
                        SemanticProfile::OriginCompleteHistoryV1,
                        left,
                        right,
                        NodeKind::Triggered {
                            interval,
                            left: NodeId(0),
                            right: NodeId(1),
                        },
                        O::Triggered(oracle_interval, p(), q()),
                    ));
                }
            }
        }
    }
    for leaf in leaves {
        out.push(unary(
            SemanticProfile::OriginCompleteHistoryV1,
            leaf,
            NodeKind::StrongPrevious { operand: NodeId(0) },
            O::StrongPrevious(Box::new(leaf.oracle())),
        ));
    }
    out
}

fn formula_count(max_depth: u32, interval_count: u64, past: bool) -> u64 {
    let mut count = LEAVES;
    for _ in 0..max_depth {
        let unary_roots = 1 + 2 * interval_count + u64::from(past);
        let binary_roots = 4 + 2 * interval_count;
        count = LEAVES + unary_roots * count + binary_roots * count * count;
    }
    count
}

fn word_positions(max_length: usize) -> u64 {
    (1..=max_length)
        .map(|length| u64::try_from(length * (1_usize << length)).unwrap())
        .sum()
}

#[derive(Debug, PartialEq, Eq)]
enum LedgerError {
    OutOfDomain,
    Duplicate,
    Mismatch,
    Incomplete,
}

#[derive(Debug, PartialEq, Eq)]
struct Summary {
    declared: u64,
    visited: u64,
    refused: u64,
    failed: u64,
}

struct Ledger {
    seen: Vec<bool>,
    visited: u64,
    refused: u64,
    failed: u64,
}

impl Ledger {
    fn new(declared: usize) -> Self {
        Self {
            seen: vec![false; declared],
            visited: 0,
            refused: 0,
            failed: 0,
        }
    }

    fn record(
        &mut self,
        id: usize,
        expected: Option<bool>,
        actual: Option<bool>,
    ) -> Result<(), LedgerError> {
        let seen = self.seen.get_mut(id).ok_or(LedgerError::OutOfDomain)?;
        if *seen {
            return Err(LedgerError::Duplicate);
        }
        *seen = true;
        if expected != actual {
            self.failed += 1;
            return Err(LedgerError::Mismatch);
        }
        if actual.is_some() {
            self.visited += 1;
        } else {
            self.refused += 1;
        }
        Ok(())
    }

    fn finish(self) -> Result<Summary, LedgerError> {
        let declared = u64::try_from(self.seen.len()).unwrap();
        if self.seen.iter().any(|seen| !seen) || self.visited + self.refused != declared {
            return Err(LedgerError::Incomplete);
        }
        if self.failed != 0 {
            return Err(LedgerError::Mismatch);
        }
        Ok(Summary {
            declared,
            visited: self.visited,
            refused: self.refused,
            failed: self.failed,
        })
    }
}

fn run_partition(cases: &[Case], max_length: usize) -> (Summary, u64) {
    let positions = word_positions(max_length);
    let declared = u64::try_from(cases.len()).unwrap() * positions;
    let mut ledger = Ledger::new(usize::try_from(declared).unwrap());
    let mut id = 0;
    let mut position_count = 0;
    for length in 1..=max_length {
        for bits in 0..(1_usize << length) {
            let rows: Vec<Vec<PropositionId>> = (0..length)
                .map(|position| {
                    if bits & (1 << position) != 0 {
                        vec![PropositionId(0)]
                    } else {
                        vec![]
                    }
                })
                .collect();
            let oracle_rows: Vec<BTreeMap<PropositionId, bool>> = (0..length)
                .map(|position| BTreeMap::from([(PropositionId(0), bits & (1 << position) != 0)]))
                .collect();
            let history = PositionHistoryDocument::new(
                "v1-partition",
                1,
                0,
                u64::try_from(length - 1).unwrap(),
                Some(ClockBinding::EventPosition),
                rows.iter()
                    .enumerate()
                    .map(|(position, row)| {
                        PositionObservation::new(
                            u64::try_from(position).unwrap(),
                            row.clone(),
                            None,
                        )
                    })
                    .collect(),
            )
            .unwrap();
            for position in 0..length {
                position_count += 1;
                for case in cases {
                    let root = NodeId(u32::try_from(case.nodes.len() - 1).unwrap());
                    let syntax = Formula::new(case.profile, root, &case.nodes).unwrap();
                    let expected = if case.profile == SemanticProfile::ClosedTraceV1 {
                        evaluate_closed_trace_v1(
                            &case.oracle,
                            &oracle_rows,
                            position,
                            Limits::default(),
                        )
                    } else {
                        evaluate_origin_complete(
                            &case.oracle,
                            &oracle_rows,
                            position,
                            Limits::default(),
                        )
                    }
                    .unwrap();
                    let actual = if case.profile == SemanticProfile::ClosedTraceV1 {
                        let verdict = evaluate_closed_at(
                            syntax,
                            "v1-partition",
                            &rows,
                            "word",
                            u64::try_from(position).unwrap(),
                            EvaluationLimits::default(),
                        )
                        .unwrap()
                        .verdict;
                        match verdict {
                            TruthValue::True => true,
                            TruthValue::False => false,
                            TruthValue::Pending => {
                                panic!("closed trace returned a pending verdict")
                            }
                        }
                    } else {
                        evaluate_past(
                            syntax,
                            "v1-partition",
                            &history,
                            u64::try_from(position).unwrap(),
                            "map",
                            1,
                            PastEvaluationRelationInput::Original,
                            PastEvaluationLimits::default(),
                        )
                        .unwrap()
                        .verdict
                    };
                    ledger
                        .record(id, Some(expected), Some(actual))
                        .unwrap_or_else(|error| {
                            panic!(
                                "{error:?}: case={id}, length={length}, bits={bits}, position={position}"
                            )
                        });
                    id += 1;
                }
            }
        }
    }
    assert_eq!(position_count, positions);
    assert_eq!(u64::try_from(id).unwrap(), declared);
    let summary = ledger.finish().unwrap();
    assert_eq!(summary.declared, declared);
    assert_eq!(summary.visited, declared);
    (summary, positions)
}

// Trace: TC-177, TC-178; FR-044-AC-1, FR-044-AC-2, NFR-008-AC-1
#[test]
fn completed_depth_one_production_oracle_partition_emits_native_census() {
    let cases = cases(2);
    assert_eq!(u64::try_from(cases.len()).unwrap(), FORMULAS);
    let (summary, positions) = run_partition(&cases, 3);
    assert_eq!(positions, WORD_POSITIONS);
    assert_eq!(summary.declared, DECLARED);
    println!(
        "TL_CAMPAIGN_POPULATION {}",
        serde_json::json!({
            "schema": "tl-mltl.finite-partition/v1",
            "scope": SCOPE,
            "formulas": FORMULAS,
            "word_positions": WORD_POSITIONS,
            "declared": summary.declared,
            "visited": summary.visited,
            "refused": summary.refused,
            "failed": summary.failed,
            "atom_basis": ["p0"],
            "max_depth": 1,
            "interval_max": 2,
            "trace_max_len": 3,
            "full_target_complete": false
        })
    );
}

// Trace: TC-177, TC-178; FR-044-AC-1, FR-044-AC-2
#[test]
fn extended_partition_reports_exact_full_domain_unvisited_count() {
    let intervals = 15; // sum_{b=0}^4 (b + 1)
    assert_eq!(formula_count(1, intervals, false), 402);
    assert_eq!(formula_count(1, intervals, true), 405);
    assert_eq!(formula_count(3, intervals, false), FULL_CLOSED_FORMULAS);
    assert_eq!(formula_count(3, intervals, true), FULL_PAST_FORMULAS);
    assert_eq!(word_positions(6), EXTENDED_WORD_POSITIONS);
    let cases = cases(4);
    assert_eq!(u64::try_from(cases.len()).unwrap(), EXTENDED_FORMULAS);
    let (summary, positions) = run_partition(&cases, 6);
    assert_eq!(positions, EXTENDED_WORD_POSITIONS);
    assert_eq!(summary.declared, EXTENDED_DECLARED);
    println!(
        "TL_CAMPAIGN_FULL_DOMAIN {}",
        serde_json::json!({
            "schema": "tl-mltl.full-domain-census/v1",
            "scope": "depth3_atom1_closed0_4_words1_6_with_depth1_partition",
            "atom_basis": ["p0"],
            "symmetry_reductions": [],
            "grammar": "ordered_trees_all_boolean_and_applicable_temporal_roots",
            "max_depth": 3,
            "interval_max": 4,
            "trace_max_len": 6,
            "closed_formulas": FULL_CLOSED_FORMULAS,
            "past_formulas": FULL_PAST_FORMULAS,
            "word_positions": EXTENDED_WORD_POSITIONS,
            "declared": FULL_DECLARED,
            "visited": summary.visited,
            "unvisited": FULL_DECLARED - summary.visited,
            "refused": summary.refused,
            "failed": summary.failed,
            "completed_partition": {
                "max_depth": 1,
                "formulas": EXTENDED_FORMULAS,
                "word_positions": positions,
                "declared": summary.declared,
                "visited": summary.visited,
            },
            "full_target_complete": false,
        })
    );
}

// Trace: TC-178; FR-044-AC-2
#[test]
fn seeded_ledger_faults_refuse_omission_duplication_wrong_verdict_and_bad_id() {
    let mut ledger = Ledger::new(2);
    assert_eq!(
        ledger.record(0, Some(true), Some(false)),
        Err(LedgerError::Mismatch)
    );
    assert_eq!(ledger.record(0, None, None), Err(LedgerError::Duplicate));
    assert_eq!(ledger.record(2, None, None), Err(LedgerError::OutOfDomain));
    assert_eq!(ledger.finish(), Err(LedgerError::Incomplete));
    let mut omitted = Ledger::new(2);
    omitted.record(0, Some(true), Some(true)).unwrap();
    assert_eq!(omitted.finish(), Err(LedgerError::Incomplete));
    let mut refused = Ledger::new(1);
    refused.record(0, None, None).unwrap();
    assert_eq!(
        refused.finish(),
        Ok(Summary {
            declared: 1,
            visited: 0,
            refused: 1,
            failed: 0
        })
    );
}
