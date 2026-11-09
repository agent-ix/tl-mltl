//! TL-233: NFR-001-AC-4 error-variant ledger. Each test below produces the
//! named variants from real inputs and asserts the variant itself, never its
//! message. Variants already asserted by the domain tests (for example
//! `EvaluationError::WorkLimitExceeded` in `tests/limit_register.rs`) are not
//! repeated here.

use serde::ser::{Serialize, Serializer};
use tl_mltl::wire::common::{identity, produce};
use tl_mltl::wire::trace;
use tl_mltl::wire::{
    OwnerLimits, OwnerReadErrorCode, OwnerUsage, TraceDocument, TraceSchemaVersion,
};
use tl_mltl::{
    analyze_required_history, evaluate_closed_at, evaluate_past, ClockBinding, ClockError,
    ClockSample, EvaluationError, EvaluationLimits, ExactNumber, ExactNumberError, HistoryError,
    HistoryRequirementError, HistoryRequirementValidationError, PastEvaluationError,
    PastEvaluationLimits, PastEvaluationRelationInput, PastEvaluationReport,
    PastResultRelationKind, PastResultValidationError, PositionHistoryDocument,
    PositionObservation,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

// ---------------------------------------------------------------------------
// Owner wire.
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Unserializable;

impl Serialize for Unserializable {
    fn serialize<S: Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom("unserializable on purpose"))
    }
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn owner_codes_invalid_combination_and_encoding_are_produced() {
    let trace_document = |id: &str, instants: Vec<Vec<PropositionId>>| TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: id.to_owned(),
        closed: true,
        instants,
    };
    let code = |document: &TraceDocument| {
        trace::derive(document, OwnerLimits::default())
            .unwrap_err()
            .code()
    };
    assert_eq!(
        code(&trace_document("", vec![])),
        OwnerReadErrorCode::InvalidCombination
    );
    assert_eq!(
        code(&trace_document(
            "t",
            vec![vec![PropositionId(2), PropositionId(1)]]
        )),
        OwnerReadErrorCode::InvalidCombination
    );

    // Canonical encoding fails when the value cannot serialize at all.
    assert_eq!(
        identity("tl-mltl.test/v1", &Unserializable, "digest")
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::Encoding
    );
    assert_eq!(
        produce(
            Unserializable,
            OwnerUsage::default(),
            OwnerLimits::default()
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::Encoding
    );
}

// ---------------------------------------------------------------------------
// Exact numbers, clocks, histories.
// ---------------------------------------------------------------------------

fn event_history(id: &str, revision: u64, length: u64) -> PositionHistoryDocument {
    let observations = (0..length)
        .map(|position| PositionObservation::new(position, vec![PropositionId(0)], None))
        .collect();
    PositionHistoryDocument::new(
        id,
        revision,
        0,
        length - 1,
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap()
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn exact_number_and_clock_unit_refusals_are_produced() {
    assert_eq!(
        ExactNumber::new(1, 0),
        Err(ExactNumberError::ZeroDenominator)
    );
    assert_eq!(ExactNumber::new(2, 4), Err(ExactNumberError::NotNormalized));
    let epoch = ExactNumber::new(0, 1).unwrap();
    assert_eq!(
        tl_mltl::fixed_sample_instant(epoch, ExactNumber::new(0, 1).unwrap(), 1),
        Err(ExactNumberError::NonPositivePeriod)
    );
    assert_eq!(
        tl_mltl::fixed_sample_instant(epoch, ExactNumber::new(i64::MAX, 1).unwrap(), u64::MAX),
        Err(ExactNumberError::ArithmeticOverflow)
    );

    let unit_history = |unit: &str| {
        let one = ExactNumber::new(1, 1).unwrap();
        PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            Some(ClockBinding::FixedSample {
                epoch: ExactNumber::new(0, 1).unwrap(),
                period: one,
                unit: unit.to_owned(),
            }),
            vec![PositionObservation::new(
                0,
                vec![],
                Some(ClockSample {
                    instant: ExactNumber::new(0, 1).unwrap(),
                    unit: unit.to_owned(),
                }),
            )],
        )
    };
    assert_eq!(
        unit_history("").unwrap_err(),
        HistoryError::Clock(ClockError::EmptyUnit)
    );
    assert_eq!(
        unit_history(&"u".repeat(129)).unwrap_err(),
        HistoryError::Clock(ClockError::UnitTooLong {
            actual: 129,
            limit: 128
        })
    );
}

// ---------------------------------------------------------------------------
// Required history and past evaluation.
// ---------------------------------------------------------------------------

fn once_nodes() -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Once {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(0),
        }),
    ]
}

fn past_formula(nodes: &[Node]) -> Formula<'_> {
    Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
}

fn report(
    nodes: &[Node],
    history: &PositionHistoryDocument,
    revision: u64,
    relation: PastEvaluationRelationInput<'_>,
) -> Result<PastEvaluationReport, PastEvaluationError> {
    evaluate_past(
        past_formula(nodes),
        "f",
        history,
        1,
        "m",
        revision,
        relation,
        PastEvaluationLimits::default(),
    )
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn history_requirement_refusals_are_produced() {
    let nodes = once_nodes();
    assert_eq!(
        analyze_required_history(past_formula(&nodes), "").unwrap_err(),
        HistoryRequirementError::InvalidFormulaIdentity
    );

    let mut requirement = analyze_required_history(past_formula(&nodes), "f").unwrap();
    requirement.validate().unwrap();
    let mut malformed = requirement.clone();
    malformed.formula_sha256 = "NOT-A-DIGEST".to_owned();
    assert_eq!(
        malformed.validate(),
        Err(HistoryRequirementValidationError::MalformedFormulaDigest)
    );
    requirement.unit = "seconds".to_owned();
    assert_eq!(
        requirement.validate(),
        Err(HistoryRequirementValidationError::IdentityMismatch { field: "unit" })
    );
}

// The formula-identity document bounds the node table, so a table one past
// its ceiling cannot be identified. Building it is too heavy for Miri.
#[cfg(not(miri))]
// Trace: TC-176, NFR-001-AC-4
#[test]
fn an_unidentifiable_formula_is_refused_by_requirement_analysis() {
    let count = tl_syntax::MAX_FORMULA_DOCUMENT_NODES + 1;
    let mut nodes = vec![Node::new(NodeKind::True); count];
    nodes[count - 1] = Node::new(NodeKind::Not { operand: NodeId(0) });
    assert_eq!(
        analyze_required_history(past_formula(&nodes), "f").unwrap_err(),
        HistoryRequirementError::FormulaIdentityEncoding
    );
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn past_result_validation_refusals_are_produced() {
    let nodes = once_nodes();
    let history = event_history("h", 1, 4);
    let original = report(&nodes, &history, 1, PastEvaluationRelationInput::Original).unwrap();
    original.validate().unwrap();

    let refuses = |mutate: &dyn Fn(&mut PastEvaluationReport)| {
        let mut changed = original.clone();
        mutate(&mut changed);
        changed.validate().unwrap_err()
    };
    assert_eq!(
        refuses(&|r| r.result_revision = 0),
        PastResultValidationError::ResultRevisionZero
    );
    assert_eq!(
        refuses(&|r| r.operator_profile = "other".to_owned()),
        PastResultValidationError::IdentityMismatch {
            field: "operatorProfile"
        }
    );
    assert_eq!(
        refuses(&|r| r.result_sha256 = "zz".to_owned()),
        PastResultValidationError::MalformedDigest {
            field: "resultSha256"
        }
    );
    assert_eq!(
        refuses(&|r| r.limits.max_steps = u64::MAX),
        PastResultValidationError::LimitsNotClamped
    );
    assert_eq!(
        refuses(&|r| r.stats.steps = 0),
        PastResultValidationError::StatisticsOutOfRange
    );
    assert_eq!(
        refuses(&|r| r.anchor = r.history.through_position + 1),
        PastResultValidationError::AnchorOutOfRange
    );
    assert_eq!(
        refuses(&|r| r.verdict = !r.verdict),
        PastResultValidationError::StaleResultDigest
    );
    assert_eq!(
        refuses(&|r| r.relation.kind = PastResultRelationKind::Superseding),
        PastResultValidationError::RelationShape
    );

    // A genuine correction chain, then one defect at a time.
    let corrected_history = history
        .corrected(
            2,
            3,
            (0..4)
                .map(|p| PositionObservation::new(p, vec![], None))
                .collect(),
        )
        .unwrap();
    let correction = report(
        &nodes,
        &corrected_history,
        2,
        PastEvaluationRelationInput::Superseding(&original),
    )
    .unwrap();
    correction.validate().unwrap();
    correction
        .validate_with_predecessor(Some(&original))
        .unwrap();

    let refuses = |mutate: &dyn Fn(&mut PastEvaluationReport)| {
        let mut changed = correction.clone();
        mutate(&mut changed);
        changed.validate().unwrap_err()
    };
    assert_eq!(
        refuses(&|r| r.relation.corrected_history = Some(
            r.relation
                .corrected_history
                .clone()
                .map(|mut h| {
                    h.revision += 1;
                    h
                })
                .unwrap()
        )),
        PastResultValidationError::CorrectedHistoryMismatch
    );
    assert_eq!(
        refuses(&|r| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .result_revision = 2),
        PastResultValidationError::PredecessorNotEarlier
    );
    assert_eq!(
        refuses(&|r| r.relation.direct_predecessor.as_mut().unwrap().anchor += 1),
        PastResultValidationError::PredecessorContextMismatch
    );
    assert_eq!(
        refuses(&|r| {
            let own = r.result_sha256.clone();
            r.relation
                .direct_predecessor
                .as_mut()
                .unwrap()
                .result_sha256 = own;
        }),
        PastResultValidationError::SelfPredecessor
    );
    assert_eq!(
        correction
            .validate_with_predecessor(Some(&correction))
            .unwrap_err(),
        PastResultValidationError::PredecessorReferenceMismatch
    );
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn past_evaluation_refusals_are_produced() {
    let nodes = once_nodes();
    let history = event_history("h", 1, 4);
    let original = report(&nodes, &history, 1, PastEvaluationRelationInput::Original).unwrap();

    // The evaluator names the requirement analysis that refused the formula.
    let future_nodes = vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
    ];
    let wrong_profile =
        Formula::new(SemanticProfile::ClosedTraceV1, NodeId(1), &future_nodes).unwrap();
    assert!(matches!(
        evaluate_past(
            wrong_profile,
            "f",
            &history,
            1,
            "m",
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits::default(),
        ),
        Err(PastEvaluationError::Requirement(
            HistoryRequirementError::UnsupportedProfile { .. }
        ))
    ));

    assert_eq!(
        evaluate_past(
            past_formula(&nodes),
            "",
            &history,
            1,
            "m",
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits::default(),
        )
        .unwrap_err(),
        PastEvaluationError::InvalidIdentity { field: "formulaId" }
    );

    let mut broken = original.clone();
    broken.verdict = !broken.verdict;
    assert!(matches!(
        report(
            &nodes,
            &history,
            2,
            PastEvaluationRelationInput::Superseding(&broken)
        ),
        Err(PastEvaluationError::InvalidPredecessor(
            PastResultValidationError::StaleResultDigest
        ))
    ));

    let other = event_history("other", 2, 4);
    assert_eq!(
        report(
            &nodes,
            &other,
            2,
            PastEvaluationRelationInput::Superseding(&original)
        )
        .unwrap_err(),
        PastEvaluationError::CorrectionContextMismatch { field: "historyId" }
    );
    assert_eq!(
        report(
            &nodes,
            &history,
            2,
            PastEvaluationRelationInput::Superseding(&original)
        )
        .unwrap_err(),
        PastEvaluationError::HistoryRevisionNotAdvanced
    );
}

// ---------------------------------------------------------------------------
// Future evaluation.
// ---------------------------------------------------------------------------

// Trace: TC-176, NFR-001-AC-4
#[test]
fn an_unordered_trace_instant_is_refused() {
    let nodes = vec![Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    })];
    let formula = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(0), &nodes).unwrap();
    let trace = vec![vec![PropositionId(1), PropositionId(0)]];
    assert_eq!(
        evaluate_closed_at(formula, "f", &trace, "t", 0, EvaluationLimits::default()).unwrap_err(),
        EvaluationError::TraceNotStrictlyOrdered {
            instant: 0,
            previous: PropositionId(1),
            current: PropositionId(0)
        }
    );
}

// ---------------------------------------------------------------------------
// Contextual binding refusals reach every contextual operation.
// ---------------------------------------------------------------------------

mod contextual {
    use tl_mltl::{
        analyze_horizon_with_context, evaluate_closed_with_context, ContextualBindingError,
        ContextualEvaluationError, ContextualHorizonError, EvaluationError, EvaluationLimits,
        HorizonError,
    };
    use tl_syntax::{
        Formula, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding, PropositionId,
        SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
    };

    fn nodes() -> Vec<Node> {
        vec![Node::new(NodeKind::Proposition {
            proposition: PropositionId(7),
        })]
    }

    fn catalog(bound: bool) -> SignalCatalogDocument {
        SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(1),
                "ready".to_owned(),
                SignalDomain::Boolean,
            )],
            if bound {
                vec![PropositionBinding::new(PropositionId(7), SignalId(1))]
            } else {
                vec![]
            },
        )
        .unwrap()
    }

    // Trace: TC-176, NFR-001-AC-4
    #[test]
    fn contextual_operations_surface_binding_and_inner_refusals() {
        let nodes = nodes();
        let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(0), &nodes).unwrap();
        let history =
            Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(0), &nodes).unwrap();
        let missing = ContextualBindingError::MissingProposition {
            proposition: PropositionId(7),
        };

        assert_eq!(
            analyze_horizon_with_context(closed, "f", &catalog(false), None).unwrap_err(),
            ContextualHorizonError::Binding(missing.clone())
        );
        assert_eq!(
            analyze_horizon_with_context(history, "f", &catalog(true), None).unwrap_err(),
            ContextualHorizonError::Horizon(HorizonError::UnsupportedProfile {
                actual: SemanticProfile::OriginCompleteHistoryV1.as_str()
            })
        );
        assert_eq!(
            evaluate_closed_with_context(
                closed,
                "f",
                &[vec![PropositionId(7)]],
                "t",
                EvaluationLimits::default(),
                &catalog(false),
                None,
            )
            .unwrap_err(),
            ContextualEvaluationError::Binding(missing)
        );
        assert_eq!(
            evaluate_closed_with_context(
                closed,
                "f",
                &[vec![PropositionId(8), PropositionId(7)]],
                "t",
                EvaluationLimits::default(),
                &catalog(true),
                None,
            )
            .unwrap_err(),
            ContextualEvaluationError::Evaluation(EvaluationError::TraceNotStrictlyOrdered {
                instant: 0,
                previous: PropositionId(8),
                current: PropositionId(7)
            })
        );
    }
}
