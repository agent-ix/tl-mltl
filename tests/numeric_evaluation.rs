//! TL-422: numeric trace document and exact predicate evaluation
//! (FR-053, FR-054, FR-055, NFR-017; tl-mltl numeric-test-matrix).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! The tl-syntax numeric corpus (tl-syntax FR-305) is not merged, so the
//! "independently computed tables" below are hand-written in this file from
//! integer arithmetic on the stated values. UNBUILT-PIN: replace them with the
//! corpus under `tl_syntax::CORPUS_DIR` once it lands.

#[path = "support/finite_harness.rs"]
mod finite_harness;

use finite_harness::{lower as lower_oracle, oi, oracle_atom, reference, Profile};
use tl_mltl::numeric::{
    evaluate_numeric, Comparison, EmbeddedReport, ExprNode, NumericAdmissionCode,
    NumericEvaluationError, NumericEvaluationLimits, NumericInstant, NumericObservation,
    NumericTraceDocument, NumericTraceError, NumericTraceLimits, NumericTraceSchemaVersion,
    PredicateCell, PredicateDefinition, PredicateTable, ValidatedNumericTrace,
    NUMERIC_TRACE_V1, NUMERIC_TRACE_V1_SCHEMA_BYTES,
};
use tl_mltl::wire::{
    OwnerLimits, OwnerReadErrorCode, TraceDocument, TraceSchemaVersion, ValidatedTrace,
};
use tl_mltl::{evaluate_closed_at, evaluate_prefix_at, EvaluationLimits, TruthValue};
use tl_oracle::population::CaseResult;
use tl_oracle::Formula as O;
use tl_syntax::{
    Formula, Interval, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
    PropositionId, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
};

const FLAG: u32 = 0; // Boolean, proposition 0
const X: u32 = 1; // Integer [-100, 100]
const D1: u32 = 2; // scale 1
const D2: u32 = 3; // scale 2
const D9: u32 = 4; // scale 9
const D18: u32 = 5; // scale 18

fn catalog() -> SignalCatalogDocument {
    let decimal = |min, max, scale| {
        SignalDomain::FixedDecimal(
            tl_syntax::FixedDecimalSignalDomain::new(min, max, scale).unwrap(),
        )
    };
    SignalCatalogDocument::new(
        vec![
            OwnedSignalDeclaration::new(SignalId(FLAG), "flag".to_owned(), SignalDomain::Boolean),
            OwnedSignalDeclaration::new(
                SignalId(X),
                "x".to_owned(),
                SignalDomain::Integer(tl_syntax::IntegerSignalDomain::new(-100, 100).unwrap()),
            ),
            OwnedSignalDeclaration::new(SignalId(D1), "d1".to_owned(), decimal(-1000, 1000, 1)),
            OwnedSignalDeclaration::new(SignalId(D2), "d2".to_owned(), decimal(-1000, 1000, 2)),
            OwnedSignalDeclaration::new(
                SignalId(D9),
                "d9".to_owned(),
                decimal(-1_000_000_000_000, 1_000_000_000_000, 9),
            ),
            OwnedSignalDeclaration::new(
                SignalId(D18),
                "d18".to_owned(),
                decimal(-1_000_000_000_000_000_000, 1_000_000_000_000_000_000, 18),
            ),
        ],
        vec![PropositionBinding::new(PropositionId(0), SignalId(FLAG))],
    )
    .unwrap()
}

fn instant(propositions: &[u32], values: Vec<NumericObservation>) -> NumericInstant {
    NumericInstant {
        propositions: propositions.iter().copied().map(PropositionId).collect(),
        values,
    }
}

fn document(closed: bool, instants: Vec<NumericInstant>) -> NumericTraceDocument {
    NumericTraceDocument {
        schema_version: NumericTraceSchemaVersion::V1,
        trace_id: "numeric".to_owned(),
        closed,
        instants,
    }
}

fn read(bytes: &[u8]) -> Result<ValidatedNumericTrace, NumericTraceError> {
    ValidatedNumericTrace::from_json_bytes(bytes, &catalog(), NumericTraceLimits::default())
}

/// Predicate proposition 1: `x > 0`; proposition 2: `d1 >= 0.2`.
fn table() -> PredicateTable {
    PredicateTable {
        expressions: vec![
            ExprNode::Signal { signal: SignalId(X) },
            ExprNode::Integer { value: 0 },
            ExprNode::Signal { signal: SignalId(D1) },
            ExprNode::Decimal {
                coefficient: 2,
                scale: 1,
            },
        ],
        predicates: vec![
            PredicateDefinition {
                proposition: PropositionId(1),
                comparison: Comparison::Gt,
                left: 0,
                right: 1,
            },
            PredicateDefinition {
                proposition: PropositionId(2),
                comparison: Comparison::Ge,
                left: 2,
                right: 3,
            },
        ],
    }
}

fn atom_formula(profile: SemanticProfile, proposition: u32) -> (Vec<Node>, NodeId) {
    let _ = profile;
    (
        vec![Node::new(NodeKind::Proposition {
            proposition: PropositionId(proposition),
        })],
        NodeId(0),
    )
}

// ------------------------------------------------------------------ FR-053

// Trace: TC-1570, FR-053-AC-1
#[test]
fn integer_and_decimal_observations_round_trip_at_every_scale() {
    let doc = document(
        true,
        vec![
            instant(
                &[0],
                vec![
                    NumericObservation::integer(X, -100),
                    NumericObservation::decimal(D1, 1000, 1),
                    NumericObservation::decimal(D2, -1000, 2),
                    NumericObservation::decimal(D9, 123_456_789, 9),
                    NumericObservation::decimal(D18, -1_000_000_000_000_000_000, 18),
                ],
            ),
            instant(&[], vec![NumericObservation::integer(X, 100)]),
            instant(&[], vec![]),
        ],
    );
    let bytes = serde_json::to_vec(&doc).unwrap();
    let validated = read(&bytes).unwrap();
    assert_eq!(validated.bytes(), &bytes[..]);
    assert_eq!(validated.document(), &doc);
    assert_eq!(serde_json::to_vec(validated.document()).unwrap(), bytes);

    let checked_in = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("schemas/numeric-trace-v1.schema.json"),
    )
    .unwrap();
    assert_eq!(NUMERIC_TRACE_V1_SCHEMA_BYTES, &checked_in[..]);
    let schema = String::from_utf8(checked_in).unwrap();
    assert!(schema.contains(NUMERIC_TRACE_V1));
    assert_eq!(NUMERIC_TRACE_V1, "tl-mltl.numeric-trace/v1");
}

fn admission_code(bytes: &[u8]) -> NumericAdmissionCode {
    match read(bytes) {
        Err(NumericTraceError::Admission { code, .. }) => code,
        other => panic!("expected admission refusal, got {other:?}"),
    }
}

fn owner_code(bytes: &[u8]) -> (OwnerReadErrorCode, &'static str) {
    match read(bytes) {
        Err(NumericTraceError::Owner(error)) => (error.code(), error.field()),
        other => panic!("expected owner refusal, got {other:?}"),
    }
}

fn wire(values: &str) -> Vec<u8> {
    format!(
        concat!(
            r#"{{"schemaVersion":"tl-mltl.numeric-trace/v1","traceId":"t","closed":true,"#,
            r#""instants":[{{"propositions":[],"values":[{values}]}}]}}"#
        ),
        values = values
    )
    .into_bytes()
}

// Trace: TC-1571, FR-053-AC-2
#[test]
fn every_admission_and_wire_error_class_has_its_own_code() {
    use NumericAdmissionCode as A;
    assert_eq!(admission_code(&wire(r#"{"signal":9,"integer":1}"#)), A::UnknownSignal);
    assert_eq!(admission_code(&wire(r#"{"signal":0,"integer":1}"#)), A::NonNumericSignal);
    assert_eq!(
        admission_code(&wire(r#"{"signal":1,"decimal":{"coefficient":1,"scale":0}}"#)),
        A::KindMismatch
    );
    assert_eq!(admission_code(&wire(r#"{"signal":2,"integer":1}"#)), A::KindMismatch);
    assert_eq!(admission_code(&wire(r#"{"signal":1}"#)), A::KindMismatch);
    assert_eq!(
        admission_code(&wire(r#"{"signal":2,"decimal":{"coefficient":1,"scale":2}}"#)),
        A::ScaleMismatch
    );
    assert_eq!(admission_code(&wire(r#"{"signal":1,"integer":101}"#)), A::ValueOutOfDomain);
    assert_eq!(
        admission_code(&wire(r#"{"signal":2,"integer":1},{"signal":1,"integer":1}"#)),
        A::ValuesNotStrictlyOrdered
    );
    assert_eq!(
        admission_code(&wire(r#"{"signal":1,"integer":1},{"signal":1,"integer":2}"#)),
        A::ValuesNotStrictlyOrdered
    );

    // Wire classes. SPEC-GAP: the shared owner reader reports float and
    // exponent alike (`InvalidJson`); they are distinct inputs, one code.
    use OwnerReadErrorCode as W;
    assert_eq!(owner_code(&wire(r#"{"signal":1,"integer":1.5}"#)).0, W::InvalidJson);
    assert_eq!(owner_code(&wire(r#"{"signal":1,"integer":1e2}"#)).0, W::InvalidJson);
    assert_eq!(owner_code(&wire(r#"{"signal":1,"integer":-0}"#)).0, W::NonCanonical);
    assert_eq!(owner_code(&wire(r#"{"signal":1,"integer":01}"#)).0, W::InvalidJson);
    assert_eq!(
        owner_code(&wire(r#"{"signal":1,"signal":1,"integer":1}"#)).0,
        W::InvalidJson
    );
    assert_eq!(
        owner_code(&wire(r#"{"signal":1,"integer":1,"extra":0}"#)).0,
        W::InvalidJson
    );
    let mut trailing = wire(r#"{"signal":1,"integer":1}"#);
    trailing.extend_from_slice(b" {}");
    assert_eq!(owner_code(&trailing), (W::InvalidJson, "trailingData"));
    let small = NumericTraceLimits {
        owner: OwnerLimits {
            max_input_bytes: 16,
            ..OwnerLimits::default()
        },
        ..NumericTraceLimits::default()
    };
    assert!(matches!(
        ValidatedNumericTrace::from_json_bytes(&wire(""), &catalog(), small),
        Err(NumericTraceError::Owner(error)) if error.code() == W::ResourceIncomplete
    ));
    let few_values = NumericTraceLimits {
        max_values: 0,
        ..NumericTraceLimits::default()
    };
    assert!(matches!(
        ValidatedNumericTrace::from_json_bytes(
            &wire(r#"{"signal":1,"integer":1}"#),
            &catalog(),
            few_values
        ),
        Err(NumericTraceError::Owner(error)) if error.field() == "values"
    ));
}

// Trace: TC-1572, FR-053-AC-3
#[test]
fn boolean_trace_v1_is_unchanged_and_refused_by_the_numeric_reader() {
    let v1 = TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: "t".to_owned(),
        closed: true,
        instants: vec![vec![PropositionId(0)], vec![]],
    };
    let bytes = serde_json::to_vec(&v1).unwrap();
    let validated = ValidatedTrace::from_json_bytes(&bytes, OwnerLimits::default()).unwrap();
    assert_eq!(validated.document(), &v1);
    assert_eq!(
        tl_mltl::wire::trace::SCHEMA_SHA256,
        "9c1020bb56cbd38ebc10405a39ccffc156908bb00f032de4c518626343989946"
    );
    assert!(matches!(
        read(&bytes),
        Err(NumericTraceError::Owner(error)) if error.code() == OwnerReadErrorCode::InvalidJson
    ));
}

// ------------------------------------------------------------------ FR-054

fn corpus_trace() -> NumericTraceDocument {
    // x: -1, 0, 1, (absent); d1: 0.3, 0.1, 0.2, 0.2
    document(
        true,
        vec![
            instant(
                &[0],
                vec![
                    NumericObservation::integer(X, -1),
                    NumericObservation::decimal(D1, 3, 1),
                ],
            ),
            instant(
                &[],
                vec![
                    NumericObservation::integer(X, 0),
                    NumericObservation::decimal(D1, 1, 1),
                ],
            ),
            instant(
                &[0],
                vec![
                    NumericObservation::integer(X, 1),
                    NumericObservation::decimal(D1, 2, 1),
                ],
            ),
            instant(&[], vec![NumericObservation::decimal(D1, 2, 1)]),
        ],
    )
}

// Trace: TC-1573, FR-054-AC-1
#[test]
fn predicate_truth_equals_the_independent_table() {
    use PredicateCell::{False as F, True as T, Unobserved as U};
    // Independent table: x > 0 <=> x >= 1; d1 >= 0.2 <=> 10*d1 >= 2.
    let expected = vec![vec![F, T], vec![F, F], vec![T, T], vec![U, T]];
    let (nodes, root) = atom_formula(SemanticProfile::ClosedTraceV1, 1);
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
    let report = evaluate_numeric(
        formula,
        "f",
        &table(),
        &corpus_trace(),
        0,
        NumericEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(report.numeric.truth, expected);
    assert_eq!(report.numeric.unobserved_cells, 1);
    assert_eq!(report.numeric.predicate_proposition_ids, vec![1, 2]);
}

fn oracle_formulas() -> Vec<O> {
    let p = || Box::new(oracle_atom(0));
    let q = || Box::new(oracle_atom(1));
    vec![
        O::Future(oi((0, 2)), q()),
        O::Globally(oi((0, 3)), q()),
        O::Until(oi((0, 2)), p(), q()),
        O::Release(oi((1, 2)), q(), p()),
        O::And(p(), Box::new(O::Future(oi((1, 1)), q()))),
        O::Not(Box::new(O::Globally(oi((0, 1)), q()))),
    ]
}

// Trace: TC-1574, FR-054-AC-1
#[test]
fn verdicts_equal_tl_oracle_on_the_induced_trace() {
    // Atom 1 is predicate `x > 0` here; atom 0 is the Boolean flag.
    let table = PredicateTable {
        predicates: vec![table().predicates[0]],
        ..table()
    };
    let mut trace = corpus_trace();
    trace.instants[3].values.insert(0, NumericObservation::integer(X, 5));
    for f in oracle_formulas() {
        let mut nodes = Vec::new();
        let root = lower_oracle(&f, &mut nodes);
        for time in 0..4u64 {
            let closed = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
            let report = evaluate_numeric(
                closed,
                "f",
                &table,
                &trace,
                time,
                NumericEvaluationLimits::default(),
            )
            .unwrap();
            let word: Vec<[bool; 2]> = report
                .numeric
                .derived_trace
                .iter()
                .map(|row| [row.contains(&0), row.contains(&1)])
                .collect();
            let expected = reference(Profile::ClosedFuture, &f, &word, time as usize);
            let EmbeddedReport::Future(inner) = &report.evaluation else {
                panic!("closed profile embeds a future report");
            };
            let actual = match inner.verdict {
                TruthValue::True => CaseResult::Admitted(true),
                TruthValue::False => CaseResult::Admitted(false),
                TruthValue::Pending => panic!("closed verdict pending"),
            };
            assert_eq!(actual, expected);

            // Prefix: a decided verdict equals the closed reading (padding is
            // one admissible continuation).
            let prefix = Formula::new(SemanticProfile::OnlinePrefixV1, root, &nodes).unwrap();
            let open = NumericTraceDocument {
                closed: false,
                ..trace.clone()
            };
            let report = evaluate_numeric(
                prefix,
                "f",
                &table,
                &open,
                time,
                NumericEvaluationLimits::default(),
            )
            .unwrap();
            let EmbeddedReport::Future(inner) = &report.evaluation else {
                panic!("prefix profile embeds a future report");
            };
            match inner.verdict {
                TruthValue::True => assert_eq!(expected, CaseResult::Admitted(true)),
                TruthValue::False => assert_eq!(expected, CaseResult::Admitted(false)),
                TruthValue::Pending => {}
            }
        }
    }
}

// Trace: TC-1575, FR-054-AC-2
#[test]
fn numeric_free_traces_match_the_existing_evaluators_byte_for_byte() {
    let trace = document(
        true,
        vec![instant(&[0], vec![]), instant(&[], vec![]), instant(&[0], vec![])],
    );
    let boolean: Vec<Vec<PropositionId>> =
        vec![vec![PropositionId(0)], vec![], vec![PropositionId(0)]];
    let empty = PredicateTable::default();
    for f in oracle_formulas() {
        let mut nodes = Vec::new();
        let root = lower_oracle(&f, &mut nodes);
        let closed = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
        let report =
            evaluate_numeric(closed, "f", &empty, &trace, 0, NumericEvaluationLimits::default())
                .unwrap();
        let direct =
            evaluate_closed_at(closed, "f", &boolean, "numeric", 0, EvaluationLimits::default())
                .unwrap();
        assert_eq!(
            serde_json::to_vec(&report.evaluation).unwrap(),
            serde_json::to_vec(&EmbeddedReport::Future(direct)).unwrap()
        );
        let prefix = Formula::new(SemanticProfile::OnlinePrefixV1, root, &nodes).unwrap();
        let open = NumericTraceDocument {
            closed: false,
            ..trace.clone()
        };
        let report =
            evaluate_numeric(prefix, "f", &empty, &open, 0, NumericEvaluationLimits::default())
                .unwrap();
        let direct = evaluate_prefix_at(
            prefix,
            "f",
            &boolean,
            "numeric",
            false,
            0,
            EvaluationLimits::default(),
        )
        .unwrap();
        assert_eq!(report.evaluation, EmbeddedReport::Future(direct));
    }
}

/// Test-only exact reference: rationals as `numerator / 10^scale`, compared by
/// cross-multiplication in `i128`, sharing no code with `src/`.
fn reference_compare(op: Comparison, (a, sa): (i64, u8), (b, sb): (i64, u8)) -> bool {
    let left = i128::from(a) * 10i128.pow(u32::from(sb));
    let right = i128::from(b) * 10i128.pow(u32::from(sa));
    match op {
        Comparison::Lt => left < right,
        Comparison::Le => left <= right,
        Comparison::Gt => left > right,
        Comparison::Ge => left >= right,
        Comparison::Eq => left == right,
        Comparison::Ne => left != right,
    }
}

// Trace: TC-1576, FR-054-AC-3
#[test]
fn generated_comparisons_equal_the_exact_reference_including_bounds() {
    // `rescale(d1, 2) + d2 <op> c` at every boundary and interior grid value.
    let samples = [-1000i64, -999, -1, 0, 1, 999, 1000];
    for op in Comparison::ALL {
        for constant in [-1999i64, -2, 0, 3, 1999] {
            let table = PredicateTable {
                expressions: vec![
                    ExprNode::Signal { signal: SignalId(D1) },
                    ExprNode::Rescale {
                        operand: 0,
                        scale: 2,
                    },
                    ExprNode::Signal { signal: SignalId(D2) },
                    ExprNode::Add { left: 1, right: 2 },
                    ExprNode::Decimal {
                        coefficient: constant,
                        scale: 2,
                    },
                ],
                predicates: vec![PredicateDefinition {
                    proposition: PropositionId(1),
                    comparison: op,
                    left: 3,
                    right: 4,
                }],
            };
            let mut instants = Vec::new();
            let mut expected = Vec::new();
            for &a in &samples {
                for &b in &samples {
                    instants.push(instant(
                        &[],
                        vec![
                            NumericObservation::decimal(D1, a, 1),
                            NumericObservation::decimal(D2, b, 2),
                        ],
                    ));
                    // Independent: 10*a + b at scale 2.
                    let sum = a * 10 + b;
                    expected.push(reference_compare(op, (sum, 2), (constant, 2)));
                }
            }
            let trace = document(true, instants);
            let (nodes, root) = atom_formula(SemanticProfile::ClosedTraceV1, 1);
            let formula = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
            let report =
                evaluate_numeric(
                    formula,
                    "f",
                    &table,
                    &trace,
                    0,
                    NumericEvaluationLimits::default(),
                )
                    .unwrap();
            let actual: Vec<bool> = report
                .numeric
                .truth
                .iter()
                .map(|row| row[0] == PredicateCell::True)
                .collect();
            assert_eq!(actual, expected, "{op:?} {constant}");
        }
    }
    // An unvalidated table that overflows refuses with no verdict.
    let overflow = PredicateTable {
        expressions: vec![
            ExprNode::Integer { value: i64::MAX },
            ExprNode::Mul { left: 0, right: 0 },
            ExprNode::Mul { left: 1, right: 1 },
            ExprNode::Integer { value: 0 },
        ],
        predicates: vec![PredicateDefinition {
            proposition: PropositionId(1),
            comparison: Comparison::Gt,
            left: 2,
            right: 3,
        }],
    };
    let (nodes, root) = atom_formula(SemanticProfile::ClosedTraceV1, 1);
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
    assert!(matches!(
        evaluate_numeric(
            formula,
            "f",
            &overflow,
            &document(true, vec![instant(&[], vec![])]),
            0,
            NumericEvaluationLimits::default()
        ),
        Err(NumericEvaluationError::ArithmeticOverflow { node: 2 })
    ));
}

// ------------------------------------------------------------------ FR-055

fn not_atom(proposition: u32) -> (Vec<Node>, NodeId) {
    (
        vec![
            Node::new(NodeKind::Proposition {
                proposition: PropositionId(proposition),
            }),
            Node::new(NodeKind::Not {
                operand: NodeId(0),
            }),
        ],
        NodeId(1),
    )
}

fn verdict(report: &tl_mltl::numeric::NumericEvaluationReport) -> TruthValue {
    match &report.evaluation {
        EmbeddedReport::Future(inner) => inner.verdict,
        EmbeddedReport::Past(_) => panic!("future report expected"),
    }
}

// Trace: TC-1578, FR-055-AC-1
#[test]
fn closed_profile_pads_unobserved_predicates_false_and_counts_them() {
    let trace = corpus_trace(); // x absent at position 3
    let (nodes, root) = atom_formula(SemanticProfile::ClosedTraceV1, 1);
    let atom = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
    let (not_nodes, not_root) = not_atom(1);
    let negated = Formula::new(SemanticProfile::ClosedTraceV1, not_root, &not_nodes).unwrap();
    let limits = NumericEvaluationLimits::default();
    // Omission at an observed position.
    let report = evaluate_numeric(atom, "f", &table(), &trace, 3, limits).unwrap();
    assert_eq!(verdict(&report), TruthValue::False);
    assert_eq!(report.numeric.unobserved_cells, 1);
    let report = evaluate_numeric(negated, "f", &table(), &trace, 3, limits).unwrap();
    assert_eq!(verdict(&report), TruthValue::True);
    // Absence after the end.
    let report = evaluate_numeric(atom, "f", &table(), &trace, 9, limits).unwrap();
    assert_eq!(verdict(&report), TruthValue::False);
    let report = evaluate_numeric(negated, "f", &table(), &trace, 9, limits).unwrap();
    assert_eq!(verdict(&report), TruthValue::True);
}

// Trace: TC-1579, FR-055-AC-2
#[test]
fn prefix_profile_reads_unobserved_predicates_as_pending_under_kleene() {
    // Position 0: flag (prop 0) true; x>0 (prop 1) false; prop 2 (d1>=0.2)
    // unobserved because d1 is absent. Three operand values True/False/Pending
    // come from props 0, 1 and 2; all nine (and, or) pairs with prop 2 on one side.
    let trace = document(
        false,
        vec![instant(&[0], vec![NumericObservation::integer(X, -1)])],
    );
    let limits = NumericEvaluationLimits::default();
    let and_or = |op: &str, left: u32, right: u32| -> TruthValue {
        let l = Node::new(NodeKind::Proposition {
            proposition: PropositionId(left),
        });
        let r = Node::new(NodeKind::Proposition {
            proposition: PropositionId(right),
        });
        let kind = if op == "and" {
            NodeKind::And {
                left: NodeId(0),
                right: NodeId(1),
            }
        } else {
            NodeKind::Or {
                left: NodeId(0),
                right: NodeId(1),
            }
        };
        let nodes = vec![l, r, Node::new(kind)];
        let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(2), &nodes).unwrap();
        verdict(&evaluate_numeric(formula, "f", &table(), &trace, 0, limits).unwrap())
    };
    use TruthValue::{False, Pending, True};
    let value = |prop: u32| match prop {
        0 => True,
        1 => False,
        _ => Pending,
    };
    let kleene_and = |a: TruthValue, b: TruthValue| match (a, b) {
        (False, _) | (_, False) => False,
        (True, True) => True,
        _ => Pending,
    };
    let kleene_or = |a: TruthValue, b: TruthValue| match (a, b) {
        (True, _) | (_, True) => True,
        (False, False) => False,
        _ => Pending,
    };
    for left in 0..3 {
        for right in 0..3 {
            if left != 2 && right != 2 {
                continue;
            }
            assert_eq!(and_or("and", left, right), kleene_and(value(left), value(right)));
            assert_eq!(and_or("or", left, right), kleene_or(value(left), value(right)));
        }
    }
    // With no unobserved cells the result equals the Boolean prefix evaluator.
    let full = document(
        false,
        vec![instant(
            &[0],
            vec![
                NumericObservation::integer(X, -1),
                NumericObservation::decimal(D1, 5, 1),
            ],
        )],
    );
    let (nodes, root) = atom_formula(SemanticProfile::OnlinePrefixV1, 2);
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, root, &nodes).unwrap();
    let report = evaluate_numeric(formula, "f", &table(), &full, 0, limits).unwrap();
    assert_eq!(report.numeric.unobserved_cells, 0);
    assert_eq!(verdict(&report), TruthValue::True);
}

// Trace: TC-1580, FR-055-AC-3
#[test]
fn origin_complete_profile_refuses_a_gap_and_keeps_pre_origin_false() {
    let past_formula = |interval: (u32, u32)| -> (Vec<Node>, NodeId) {
        (
            vec![
                Node::new(NodeKind::Proposition {
                    proposition: PropositionId(1),
                }),
                Node::new(NodeKind::Historically {
                    interval: Interval::new(interval.0, interval.1).unwrap(),
                    operand: NodeId(0),
                }),
            ],
            NodeId(1),
        )
    };
    let limits = NumericEvaluationLimits::default();
    let (nodes, root) = past_formula((0, 3));
    let formula = Formula::new(SemanticProfile::OriginCompleteHistoryV1, root, &nodes).unwrap();
    // x absent at position 3 (inside the history to anchor 3).
    assert!(matches!(
        evaluate_numeric(formula, "f", &table(), &corpus_trace(), 3, limits),
        Err(NumericEvaluationError::IncompleteHistory { position: 3, signal })
            if signal == SignalId(X)
    ));
    // Anchor 2: H[0,3] reaches before the origin, where the atom is false.
    let positive = document(
        true,
        vec![
            instant(&[], vec![NumericObservation::integer(X, 1)]),
            instant(&[], vec![NumericObservation::integer(X, 2)]),
            instant(&[], vec![NumericObservation::integer(X, 3)]),
        ],
    );
    let report = evaluate_numeric(formula, "f", &table(), &positive, 2, limits).unwrap();
    let EmbeddedReport::Past(inner) = &report.evaluation else {
        panic!("past report expected");
    };
    assert!(!inner.verdict);
    let (nodes, root) = past_formula((0, 2));
    let inside = Formula::new(SemanticProfile::OriginCompleteHistoryV1, root, &nodes).unwrap();
    let report = evaluate_numeric(inside, "f", &table(), &positive, 2, limits).unwrap();
    let EmbeddedReport::Past(inner) = &report.evaluation else {
        panic!("past report expected");
    };
    assert!(inner.verdict);
}

// ------------------------------------------------------------------ NFR-017

// Trace: TC-1599, NFR-017-AC-1
#[test]
fn reports_repeat_byte_for_byte_and_numeric_modules_hold_no_float() {
    let (nodes, root) = atom_formula(SemanticProfile::ClosedTraceV1, 1);
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
    let run = || {
        serde_json::to_vec(
            &evaluate_numeric(
                formula,
                "f",
                &table(),
                &corpus_trace(),
                0,
                NumericEvaluationLimits::default(),
            )
            .unwrap(),
        )
        .unwrap()
    };
    assert_eq!(run(), run());
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/numeric");
    for entry in std::fs::read_dir(dir).unwrap() {
        let text = std::fs::read_to_string(entry.unwrap().path()).unwrap();
        for token in ["f32", "f64"] {
            assert!(
                !text
                    .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                    .any(|word| word == token),
                "{token} in numeric module"
            );
        }
    }
}

// Trace: TC-1600, StR-014-VC-1
#[test]
fn two_runs_on_one_numeric_trace_give_identical_reports() {
    let bytes = serde_json::to_vec(&corpus_trace()).unwrap();
    let first = read(&bytes).unwrap();
    let second = read(&bytes).unwrap();
    assert_eq!(first, second);
    let (nodes, root) = atom_formula(SemanticProfile::ClosedTraceV1, 2);
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, root, &nodes).unwrap();
    let limits = NumericEvaluationLimits::default();
    assert_eq!(
        evaluate_numeric(formula, "f", &table(), first.document(), 0, limits),
        evaluate_numeric(formula, "f", &table(), second.document(), 0, limits)
    );
}
