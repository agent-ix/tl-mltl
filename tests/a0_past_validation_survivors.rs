//! TL-371: behavior tests for the past-result validators. The first full cargo-mutants run showed
//! that no test violated one operand of a validation condition at a time, so replacing a `||`
//! with `&&` (or moving a bound by one) went unnoticed. Each test below breaks exactly one operand
//! and expects the named refusal; boundary tests accept the value at the limit.

use sha2::{Digest, Sha256};
use tl_mltl::{
    evaluate_past, ClockBinding, ExactNumber, PastEvaluationLimits, PastEvaluationRelationInput,
    PastEvaluationReport, PastResultReference, PastResultValidationError, PositionHistoryDocument,
    PositionObservation,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

type Refusal = PastResultValidationError;

/// The evaluator's identity length bound (`MAX_IDENTITY_BYTES` in `src/past/mod.rs`).
const MAX_IDENTITY_BYTES: usize = 256;

fn nodes() -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Not { operand: NodeId(0) }),
        Node::new(NodeKind::Once {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(1),
        }),
    ]
}

fn history(revision: u64, positions: u64) -> PositionHistoryDocument {
    PositionHistoryDocument::new(
        "history-a",
        revision,
        0,
        positions - 1,
        Some(ClockBinding::EventPosition),
        (0..positions)
            .map(|p| PositionObservation::new(p, vec![PropositionId(0)], None))
            .collect(),
    )
    .unwrap()
}

fn report(
    history: &PositionHistoryDocument,
    revision: u64,
    relation: PastEvaluationRelationInput<'_>,
) -> PastEvaluationReport {
    let nodes = nodes();
    let formula =
        Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(2), &nodes).unwrap();
    evaluate_past(
        formula,
        "formula-a",
        history,
        1,
        "map-a",
        revision,
        relation,
        PastEvaluationLimits::default(),
    )
    .unwrap()
}

fn original() -> PastEvaluationReport {
    report(&history(1, 3), 1, PastEvaluationRelationInput::Original)
}

/// Recomputes the result digest the way the evaluator seals it.
fn reseal(report: &mut PastEvaluationReport) {
    let mut value = serde_json::to_value(&*report).unwrap();
    value.as_object_mut().unwrap().remove("resultSha256");
    let mut digest = Sha256::new();
    digest.update(b"tl-mltl.past-evaluation/v1");
    digest.update([0]);
    digest.update(serde_json::to_vec(&value).unwrap());
    report.result_sha256 = digest
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
}

fn refusal(mutate: impl FnOnce(&mut PastEvaluationReport)) -> Refusal {
    let mut changed = original();
    mutate(&mut changed);
    changed.validate().unwrap_err()
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn every_identity_is_bounded_on_both_sides() {
    assert_eq!(original().validate(), Ok(()));
    let long = "x".repeat(MAX_IDENTITY_BYTES);
    let too_long = "x".repeat(MAX_IDENTITY_BYTES + 1);
    let mismatch = Refusal::IdentityMismatch { field: "formulaId" };
    assert_eq!(refusal(|r| r.formula_id.clear()), mismatch);
    assert_eq!(refusal(|r| r.formula_id = too_long.clone()), mismatch);
    // The longest admitted identity passes the identity check and fails only the stale digest.
    assert_eq!(
        refusal(|r| r.formula_id = long.clone()),
        Refusal::StaleResultDigest
    );
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn each_statistic_is_checked_on_its_own() {
    let out = Refusal::StatisticsOutOfRange;
    let base = original();
    let stats = base.stats;
    assert!(stats.max_recursion_depth >= 1 && stats.input_positions >= 2);

    assert_eq!(refusal(|r| r.stats.steps = 0), out);
    assert_eq!(refusal(|r| r.stats.node_evaluations = 0), out);
    assert_eq!(refusal(|r| r.stats.steps += 1), out);
    assert_eq!(refusal(|r| r.stats.input_positions += 1), out);
    // Limits below the observed work refuse; limits equal to it pass the statistics check.
    assert_eq!(refusal(|r| r.limits.max_steps = stats.steps - 1), out);
    assert_eq!(
        refusal(|r| r.limits.max_steps = stats.steps),
        Refusal::StaleResultDigest
    );
    assert_eq!(
        refusal(|r| r.limits.max_input_positions = stats.input_positions - 1),
        out
    );
    assert_eq!(
        refusal(|r| r.limits.max_input_positions = stats.input_positions),
        Refusal::StaleResultDigest
    );
    assert_eq!(
        refusal(|r| r.limits.max_recursion_depth = stats.max_recursion_depth - 1),
        out
    );
    assert_eq!(
        refusal(|r| r.limits.max_recursion_depth = stats.max_recursion_depth),
        Refusal::StaleResultDigest
    );
    // The recorded depth needs that many plus one distinct node evaluations.
    let nodes = stats.node_evaluations;
    assert_eq!(
        refusal(|r| r.stats.max_recursion_depth = u32::try_from(nodes).unwrap()),
        out
    );
    assert_eq!(
        refusal(|r| r.stats.max_recursion_depth = u32::try_from(nodes - 1).unwrap()),
        Refusal::StaleResultDigest
    );
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn the_anchor_and_origin_are_checked_on_their_own() {
    assert_eq!(
        refusal(|r| r.history.origin_position = 1),
        Refusal::AnchorOutOfRange
    );
    assert_eq!(
        refusal(|r| r.anchor = r.history.through_position + 1),
        Refusal::AnchorOutOfRange
    );
    assert_eq!(
        refusal(|r| r.anchor = r.history.through_position),
        Refusal::StaleResultDigest
    );
}

fn correction_chain() -> (PastEvaluationReport, PastEvaluationReport) {
    let first = original();
    let corrected = history(1, 3)
        .corrected(2, 3, history(1, 4).observations().to_vec())
        .unwrap();
    let second = report(
        &corrected,
        2,
        PastEvaluationRelationInput::Superseding(&first),
    );
    (first, second)
}

fn second_refusal(mutate: impl FnOnce(&mut PastEvaluationReport)) -> Refusal {
    let (_, mut second) = correction_chain();
    mutate(&mut second);
    second.validate().unwrap_err()
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn a_predecessor_must_be_strictly_earlier_in_each_revision() {
    let (first, second) = correction_chain();
    second.validate_with_predecessor(Some(&first)).unwrap();
    let earlier = Refusal::PredecessorNotEarlier;
    let own = second.result_revision;
    let own_history = second.history.revision;
    assert_eq!(
        second_refusal(|r| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .result_revision = own),
        earlier
    );
    assert_eq!(
        second_refusal(|r| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .result_revision = 0),
        earlier
    );
    assert_eq!(
        second_refusal(|r| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .history_revision = own_history),
        earlier
    );
    assert_eq!(
        second_refusal(|r| r
            .relation
            .direct_predecessor
            .as_mut()
            .unwrap()
            .history_revision = 0),
        earlier
    );
}

fn reference_of(report: &PastEvaluationReport) -> PastResultReference {
    PastResultReference {
        result_revision: report.result_revision,
        result_sha256: report.result_sha256.clone(),
        history_id: report.history.history_id.clone(),
        history_revision: report.history.revision,
        history_sha256: report.history.history_sha256.clone(),
        anchor: report.anchor,
    }
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn a_correction_must_share_every_context_field_with_its_predecessor() {
    type Tamper = Box<dyn Fn(&mut PastEvaluationReport)>;
    let tampers: Vec<(&str, Tamper)> = vec![
        (
            "formula id",
            Box::new(|r| r.formula_id = "formula-b".into()),
        ),
        (
            "formula digest",
            Box::new(|r| r.formula_sha256 = "1".repeat(64)),
        ),
        ("formula root", Box::new(|r| r.formula_root += 1)),
        (
            "clock",
            Box::new(|r| {
                r.clock = ClockBinding::FixedSample {
                    epoch: ExactNumber::new(0, 1).unwrap(),
                    period: ExactNumber::new(1, 1).unwrap(),
                    unit: "s".into(),
                }
            }),
        ),
        (
            "proposition map",
            Box::new(|r| r.proposition_map_id = "map-b".into()),
        ),
        (
            "evaluator revision",
            Box::new(|r| r.evaluator_revision = "other".into()),
        ),
        ("limits", Box::new(|r| r.limits.max_steps -= 1)),
    ];
    for (name, tamper) in tampers {
        let (mut first, mut second) = correction_chain();
        tamper(&mut first);
        reseal(&mut first);
        second.relation.direct_predecessor = Some(reference_of(&first));
        reseal(&mut second);
        second
            .validate()
            .unwrap_or_else(|e| panic!("{name}: own validation {e:?}"));
        assert_eq!(
            second.validate_with_predecessor(Some(&first)),
            Err(Refusal::PredecessorContextMismatch),
            "{name}"
        );
    }
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn an_original_carries_neither_a_predecessor_nor_a_corrected_history() {
    let (first, second) = correction_chain();
    let shape = Refusal::RelationShape;
    assert_eq!(
        refusal(|r| r.relation.direct_predecessor = second.relation.direct_predecessor.clone()),
        shape
    );
    assert_eq!(
        refusal(|r| r.relation.corrected_history = second.relation.corrected_history.clone()),
        shape
    );
    drop(first);
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn the_evaluator_counts_temporal_work_and_bounds_identities() {
    let stats = original().stats;
    assert_eq!(
        stats.steps,
        stats.node_evaluations + stats.temporal_iterations
    );
    assert!(stats.temporal_iterations > 0, "{stats:?}");
    let nodes = nodes();
    let formula =
        Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(2), &nodes).unwrap();
    let run = |id: &str| {
        evaluate_past(
            formula,
            id,
            &history(1, 3),
            1,
            "map-a",
            1,
            PastEvaluationRelationInput::Original,
            PastEvaluationLimits::default(),
        )
    };
    assert!(run(&"x".repeat(MAX_IDENTITY_BYTES)).is_ok());
    assert!(run(&"x".repeat(MAX_IDENTITY_BYTES + 1)).is_err());
    assert!(run("").is_err());
}

// Trace: TC-176
#[test]
fn validation_errors_render_a_nonempty_message() {
    for error in [
        Refusal::RelationShape,
        Refusal::StaleResultDigest,
        Refusal::IdentityMismatch { field: "formulaId" },
        Refusal::MalformedDigest {
            field: "resultSha256",
        },
    ] {
        assert!(!error.to_string().trim().is_empty(), "{error:?}");
    }
    assert!(Refusal::IdentityMismatch { field: "formulaId" }
        .to_string()
        .contains("formulaId"));
    assert!(
        !tl_mltl::HistoryRequirementValidationError::MalformedFormulaDigest
            .to_string()
            .trim()
            .is_empty()
    );
    assert!(
        !tl_mltl::PastEvaluationError::InvalidIdentity { field: "formulaId" }
            .to_string()
            .trim()
            .is_empty()
    );
}

fn history_named(name: &str, revision: u64, positions: u64) -> PositionHistoryDocument {
    PositionHistoryDocument::new(
        name,
        revision,
        0,
        positions - 1,
        Some(ClockBinding::EventPosition),
        (0..positions)
            .map(|p| PositionObservation::new(p, vec![PropositionId(0)], None))
            .collect(),
    )
    .unwrap()
}

// Trace: TC-176, NFR-001-AC-4
#[test]
fn a_correction_of_a_history_with_a_maximum_length_identity_is_accepted() {
    let name = "h".repeat(MAX_IDENTITY_BYTES);
    let first = report(
        &history_named(&name, 1, 3),
        1,
        PastEvaluationRelationInput::Original,
    );
    first.validate().unwrap();
    let corrected = history_named(&name, 1, 3)
        .corrected(2, 3, history(1, 4).observations().to_vec())
        .unwrap();
    let second = report(
        &corrected,
        2,
        PastEvaluationRelationInput::Superseding(&first),
    );
    second.validate().unwrap();
    second.validate_with_predecessor(Some(&first)).unwrap();
}
