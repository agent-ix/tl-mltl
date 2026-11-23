//! TL-425 (evaluation half): assume-guarantee contract outcome series (FR-059).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use tl_mltl::contract::{
    contract_truth, evaluate_contracts, ContractEvaluationError, ContractEvaluationLimits,
    ContractEvaluationReport, ContractRoots, ContractSet,
};
use tl_mltl::numeric::{
    Comparison, ExprNode, NumericInstant, NumericObservation, NumericTraceDocument,
    NumericTraceSchemaVersion, PredicateDefinition, PredicateTable,
};
use tl_mltl::wire::OwnerLimits;
use tl_mltl::TruthValue;
use tl_syntax::{
    contract_outcome, ContractOutcome, ContractStatus,
    Formula, FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
    SignalId,
};

fn atom(proposition: u32) -> Node {
    Node::new(NodeKind::Proposition {
        proposition: PropositionId(proposition),
    })
}

/// Nodes 0 = p0, 1 = p1, root 2 = p0 && p1.
fn two_atoms() -> Vec<Node> {
    vec![
        atom(0),
        atom(1),
        Node::new(NodeKind::And {
            left: NodeId(0),
            right: NodeId(1),
        }),
    ]
}

fn identity(formula: Formula<'_>) -> String {
    FormulaDocument::from_formula_v2(formula)
        .unwrap()
        .content_identity()
        .unwrap()
}

fn doc(formula: Formula<'_>) -> FormulaDocument {
    FormulaDocument::from_formula_v2(formula).unwrap()
}

const INACTIVE: ContractOutcome = ContractOutcome::Status(ContractStatus::Inactive);
const INVALID: ContractOutcome = ContractOutcome::Status(ContractStatus::Invalid);
const VERIFIED: ContractOutcome = ContractOutcome::Status(ContractStatus::Verified);
const PENDING: ContractOutcome = ContractOutcome::Pending;

/// The tl-syntax FR-310 table applied to evaluator truth values.
fn outcome(assumption: TruthValue, guarantee: TruthValue) -> ContractOutcome {
    contract_outcome(contract_truth(assumption), contract_truth(guarantee))
}

fn set(formula: Formula<'_>, contracts: &[(u32, &str, u32, u32)]) -> ContractSet {
    ContractSet {
        graph_identity: identity(formula),
        semantic_profile: formula.profile(),
        contracts: contracts
            .iter()
            .map(|&(id, name, assumption, guarantee)| ContractRoots {
                id,
                name: name.to_owned(),
                assumption: NodeId(assumption),
                guarantee: NodeId(guarantee),
            })
            .collect(),
    }
}

fn trace(closed: bool, rows: Vec<(Vec<u32>, Vec<NumericObservation>)>) -> NumericTraceDocument {
    NumericTraceDocument {
        schema_version: NumericTraceSchemaVersion::V1,
        trace_id: "contracts".to_owned(),
        closed,
        instants: rows
            .into_iter()
            .map(|(propositions, values)| NumericInstant {
                propositions: propositions.into_iter().map(PropositionId).collect(),
                values,
            })
            .collect(),
    }
}

fn boolean(closed: bool, rows: &[&[u32]]) -> NumericTraceDocument {
    trace(
        closed,
        rows.iter().map(|row| (row.to_vec(), Vec::new())).collect(),
    )
}

/// p0 := x > 0, p1 := y > 0 (signals 1 and 2).
fn predicates() -> PredicateTable {
    PredicateTable {
        expressions: vec![
            ExprNode::Signal { signal: SignalId(1) },
            ExprNode::Integer { value: 0 },
            ExprNode::Signal { signal: SignalId(2) },
        ],
        predicates: vec![
            PredicateDefinition {
                proposition: PropositionId(0),
                comparison: Comparison::Gt,
                left: 0,
                right: 1,
            },
            PredicateDefinition {
                proposition: PropositionId(1),
                comparison: Comparison::Gt,
                left: 2,
                right: 1,
            },
        ],
    }
}

fn value(signal: u32, truth: TruthValue) -> Option<NumericObservation> {
    match truth {
        TruthValue::True => Some(NumericObservation::integer(signal, 1)),
        TruthValue::False => Some(NumericObservation::integer(signal, -1)),
        TruthValue::Pending => None,
    }
}

const VALUES: [TruthValue; 3] = [TruthValue::True, TruthValue::False, TruthValue::Pending];

fn nine_combination_trace(closed: bool) -> NumericTraceDocument {
    let mut rows = Vec::new();
    for a in VALUES {
        for g in VALUES {
            let values = [value(1, a), value(2, g)].into_iter().flatten().collect();
            rows.push((Vec::new(), values));
        }
    }
    trace(closed, rows)
}

// Trace: TC-1593, FR-059-AC-1
#[test]
fn nine_rows_on_prefix_and_four_decided_rows_without_pending_on_closed() {
    let nodes = two_atoms();
    let limits = ContractEvaluationLimits::default();

    let prefix = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(2), &nodes).unwrap();
    let report = evaluate_contracts(
        &doc(prefix),
        "f",
        &set(prefix, &[(1, "c", 0, 1)]),
        &predicates(),
        &nine_combination_trace(false),
        limits,
    )
    .unwrap();
    let outcomes = &report.contracts[0].outcomes;
    assert_eq!(outcomes.len(), 9);
    let mut index = 0;
    for a in VALUES {
        for g in VALUES {
            assert_eq!(outcomes[index].outcome, outcome(a, g), "{a:?} {g:?}");
            index += 1;
        }
    }
    // The table itself, row by row (tl-syntax FR-310).
    use TruthValue::{False, Pending as P, True};
    for g in VALUES {
        assert_eq!(outcome(False, g), INACTIVE);
        assert_eq!(outcome(P, g), PENDING);
    }
    assert_eq!(outcome(True, False), INVALID);
    assert_eq!(outcome(True, True), VERIFIED);
    assert_eq!(outcome(True, P), PENDING);

    let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(2), &nodes).unwrap();
    let report = evaluate_contracts(
        &doc(closed),
        "f",
        &set(closed, &[(1, "c", 0, 1)]),
        &predicates(),
        &nine_combination_trace(true),
        limits,
    )
    .unwrap();
    let seen: Vec<ContractOutcome> = report.contracts[0]
        .outcomes
        .iter()
        .map(|position| position.outcome)
        .collect();
    assert!(!seen.contains(&PENDING));
    for status in [INACTIVE, INVALID, VERIFIED] {
        assert!(seen.contains(&status));
    }
    // Closed reads an unobserved predicate as false: rows map through the table
    // with Pending replaced by False.
    let decided = |v: TruthValue| if v == P { False } else { v };
    let mut index = 0;
    for a in VALUES {
        for g in VALUES {
            assert_eq!(seen[index], outcome(decided(a), decided(g)));
            index += 1;
        }
    }
}

// Trace: TC-1594, FR-059-AC-2
#[test]
fn never_exercised_invalid_and_verified_contracts_on_the_corpus() {
    // p0 assumption-only, p1 guarantee, p2 never true.
    let nodes = vec![
        atom(0),
        atom(1),
        atom(2),
        Node::new(NodeKind::And {
            left: NodeId(0),
            right: NodeId(1),
        }),
        Node::new(NodeKind::And {
            left: NodeId(3),
            right: NodeId(2),
        }),
    ];
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(4), &nodes).unwrap();
    let corpus = boolean(true, &[&[0, 1], &[0], &[0, 1], &[1]]);
    let contracts = set(
        formula,
        &[(1, "never", 2, 1), (2, "breaks", 0, 1), (3, "holds", 1, 1)],
    );
    let report = evaluate_contracts(
        &doc(formula),
        "f",
        &contracts,
        &PredicateTable::default(),
        &corpus,
        ContractEvaluationLimits::default(),
    )
    .unwrap();
    let never = &report.contracts[0];
    assert!(never
        .outcomes
        .iter()
        .all(|position| position.outcome == INACTIVE));
    assert_eq!(never.summary.unpadded.inactive, 4);
    let breaks = &report.contracts[1];
    assert_eq!(breaks.summary.first_invalid, Some(1));
    assert_eq!(breaks.summary.first_inactive, Some(3));
    let holds = &report.contracts[2];
    assert_eq!(holds.summary.unpadded.verified, 3);
    assert_eq!(holds.summary.first_invalid, None);
}

// Trace: TC-1595, FR-059-AC-3
#[test]
fn graph_identity_mismatch_is_refused_and_report_round_trips_strictly() {
    let nodes = two_atoms();
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(2), &nodes).unwrap();
    let mut contracts = set(formula, &[(1, "c", 0, 1)]);
    let corpus = boolean(true, &[&[0], &[0, 1]]);
    let report = evaluate_contracts(
        &doc(formula),
        "f",
        &contracts,
        &PredicateTable::default(),
        &corpus,
        ContractEvaluationLimits::default(),
    )
    .unwrap();
    let bytes = report.canonical_json_bytes();
    assert_eq!(
        ContractEvaluationReport::from_json_bytes(&bytes, OwnerLimits::default()).unwrap(),
        report
    );
    let mut spaced = bytes.clone();
    spaced.insert(1, b' ');
    assert!(ContractEvaluationReport::from_json_bytes(&spaced, OwnerLimits::default()).is_err());

    contracts.graph_identity = "0".repeat(64);
    assert!(matches!(
        evaluate_contracts(
            &doc(formula),
            "f",
            &contracts,
            &PredicateTable::default(),
            &corpus,
            ContractEvaluationLimits::default(),
        ),
        Err(ContractEvaluationError::GraphIdentityMismatch { .. })
    ));
}

// Trace: TC-1603, FR-059-AC-4
#[test]
fn closed_tail_padding_is_flagged_and_excluded_from_first_times() {
    // 0 = true, 1 = p, 2 = G[0,1] p, 3 = q, 4 = F[0,1] q, 5 = root.
    let nodes = vec![
        Node::new(NodeKind::True),
        atom(0),
        Node::new(NodeKind::Globally {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(1),
        }),
        atom(1),
        Node::new(NodeKind::Future {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(3),
        }),
        Node::new(NodeKind::And {
            left: NodeId(2),
            right: NodeId(4),
        }),
    ];
    let rows: &[&[u32]] = &[&[0, 1], &[0, 1], &[0]];
    let limits = ContractEvaluationLimits::default();

    let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(5), &nodes).unwrap();
    let contracts = set(closed, &[(1, "guarantee-pads", 0, 2), (2, "assumption-pads", 4, 0)]);
    let report = evaluate_contracts(
        &doc(closed),
        "f",
        &contracts,
        &PredicateTable::default(),
        &boolean(true, rows),
        limits,
    )
    .unwrap();
    let guarantee = &report.contracts[0];
    let last = guarantee.outcomes[2];
    assert_eq!(last.outcome, INVALID);
    assert!(last.padded);
    assert_eq!(guarantee.summary.first_invalid, None);
    assert_eq!(guarantee.summary.padded.invalid, 1);
    let assumption = &report.contracts[1];
    let last = assumption.outcomes[2];
    assert_eq!(last.outcome, INACTIVE);
    assert!(last.padded);
    assert_eq!(assumption.summary.first_inactive, None);

    let prefix = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(5), &nodes).unwrap();
    let report = evaluate_contracts(
        &doc(prefix),
        "f",
        &set(prefix, &[(1, "guarantee-pads", 0, 2)]),
        &PredicateTable::default(),
        &boolean(false, rows),
        limits,
    )
    .unwrap();
    let last = report.contracts[0].outcomes[2];
    assert_eq!(last.outcome, PENDING);
    assert!(!last.padded);
}

// Trace: TC-1604, FR-059-AC-5
#[test]
fn empty_trace_gives_an_empty_series_on_every_profile() {
    let nodes = two_atoms();
    for profile in [
        SemanticProfile::ClosedTraceV1,
        SemanticProfile::OnlinePrefixV1,
        SemanticProfile::OriginCompleteHistoryV1,
    ] {
        let formula = Formula::new(profile, NodeId(2), &nodes).unwrap();
        let report = evaluate_contracts(
            &doc(formula),
            "f",
            &set(formula, &[(1, "c", 0, 1)]),
            &PredicateTable::default(),
            &boolean(profile == SemanticProfile::ClosedTraceV1, &[]),
            ContractEvaluationLimits::default(),
        )
        .unwrap();
        assert!(report.empty_trace);
        let series = &report.contracts[0];
        assert!(series.outcomes.is_empty());
        assert_eq!(series.summary, Default::default());
        assert_eq!(series.summary.first_invalid, None);
        assert_eq!(series.summary.first_inactive, None);
    }
}
