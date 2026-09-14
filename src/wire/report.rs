//! Canonical `tl-mltl.temporal-assessment-result/v1` owner boundary.

use std::collections::BTreeSet;

use quire_observation::authority::completeness;
use serde::{Deserialize, Serialize};

use super::common::{identity, is_sha256, produce, raw_sha256, read_expected};
use super::request::{
    ArtifactReference, AvailabilityReference, AxisReference, CompletenessReference, InputReference,
    TemporalLane, ValidatedTemporalRequest,
};
use super::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};
use crate::future::{evaluate_closed_at_with_stats, evaluate_prefix_at_with_stats};
use crate::past::evaluate_past_with_stats;
use crate::{
    EvaluationError, EvaluationLimits, OwnerHistoryState, PastEvaluationError,
    PastEvaluationLimits, PastEvaluationRelationInput, TruthValue,
};

/// Immutable temporal result contract label.
pub const CONTRACT: &str = "tl-mltl.temporal-assessment-result/v1";
/// Exact checked-in JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/temporal-assessment-result-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "830368ae792403246ff8b8d50980abdcb556481998bae91aa873cc75f71fe9d4";

/// Closed execution disposition for one admitted request.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssessmentExecution {
    /// The selected evaluator completed.
    Completed,
    /// A caller or owner resource ceiling prevented completion.
    ResourceIncomplete,
    /// The selected clock/profile/operator is unsupported.
    Unsupported,
    /// Evaluation failed after admission without a Boolean result.
    Failed,
    /// The evaluator refused an invalid runtime combination.
    Refused,
}

/// TL truth state kept independent from execution and observation facts.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporalTruth {
    /// Formula is conclusively true.
    Satisfied,
    /// Formula is conclusively false.
    Violated,
    /// An open future prefix has not decided truth.
    Pending,
    /// No truth was produced.
    Unavailable,
}

/// Why the result is or is not settled.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SettlementBasis {
    /// The exact selected scope was evaluated as closed.
    ClosedScope,
    /// An open-scope witness made satisfaction final.
    DecisiveWitness,
    /// An open-scope counterexample made violation final.
    DecisiveCounterexample,
    /// The admitted prefix remains undecided.
    Unsettled,
    /// No truth was produced.
    Unavailable,
}

/// Immutable correction relation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResultRelationKind {
    /// First result in a lineage.
    Original,
    /// Corrected input supersedes the direct predecessor.
    Superseding,
    /// Corrected input invalidates the direct predecessor.
    Invalidating,
}

/// Caller supplies complete predecessor bytes, never a free-form reference.
#[derive(Clone, Copy, Debug)]
pub enum ResultRelationInput<'a> {
    /// Original result.
    Original,
    /// Supersede this exact validated result.
    Superseding(&'a ValidatedTemporalResult),
    /// Invalidate this exact validated result.
    Invalidating(&'a ValidatedTemporalResult),
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RelationWire {
    kind: ResultRelationKind,
    direct_predecessor_identity: Option<String>,
    direct_predecessor_digest: Option<String>,
    corrected_request_identity: Option<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct UsageWire {
    wire_bytes: u64,
    depth: u64,
    string_bytes: u64,
    formula_nodes: u64,
    formula_depth: u64,
    positions: u64,
    propositions: u64,
    support: u64,
    history_span: u64,
    evaluation_steps: u64,
    recursion_depth: u32,
    visited_fields: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ResultWire {
    pub(crate) contract_version: String,
    pub(crate) identity: String,
    pub(crate) revision: u64,
    pub(crate) request: ArtifactReference,
    pub(crate) lane: TemporalLane,
    pub(crate) subject_identity: String,
    pub(crate) correspondence_identity: String,
    pub(crate) formula: ArtifactReference,
    pub(crate) input: InputReference,
    pub(crate) proposition_map: ArtifactReference,
    pub(crate) clock: ArtifactReference,
    pub(crate) evaluator: super::request::EvaluatorReference,
    pub(crate) anchor: u64,
    pub(crate) execution: AssessmentExecution,
    pub(crate) truth: TemporalTruth,
    pub(crate) final_result: bool,
    pub(crate) settlement: SettlementBasis,
    pub(crate) decision_support: Vec<String>,
    pub(crate) decision_scope_progress: AxisReference,
    pub(crate) decision_scope_closure: AxisReference,
    pub(crate) surrounding_execution_progress: AxisReference,
    pub(crate) surrounding_execution_closure: AxisReference,
    pub(crate) completeness: CompletenessReference,
    pub(crate) availability: AvailabilityReference,
    pub(crate) relation: RelationWire,
    pub(crate) usage: UsageWire,
}

/// Immutable canonical temporal result bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalResultDocument {
    wire: ResultWire,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl TemporalResultDocument {
    /// Result identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.wire.identity
    }

    /// Positive immutable result revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.wire.revision
    }

    /// Exact canonical owner bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bounded production work.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }
}

/// Constructor-private result admitted against its exact request and lineage.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedTemporalResult {
    wire: ResultWire,
    bytes: Vec<u8>,
    usage: OwnerUsage,
    request: ValidatedTemporalRequest,
}

impl ValidatedTemporalResult {
    /// Result identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.wire.identity
    }

    /// Positive immutable revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.wire.revision
    }

    /// Exact source request identity.
    #[must_use]
    pub fn request_identity(&self) -> &str {
        self.wire.request.identity()
    }

    /// Assessment execution state.
    #[must_use]
    pub const fn execution(&self) -> AssessmentExecution {
        self.wire.execution
    }

    /// Independent truth state.
    #[must_use]
    pub const fn truth(&self) -> TemporalTruth {
        self.wire.truth
    }

    /// True only for a completed, settled Boolean result.
    #[must_use]
    pub const fn is_final(&self) -> bool {
        self.wire.final_result
    }

    /// Settlement basis.
    #[must_use]
    pub const fn settlement(&self) -> SettlementBasis {
        self.wire.settlement
    }

    /// Exact sorted decision-support identities.
    #[must_use]
    pub fn decision_support(&self) -> &[String] {
        &self.wire.decision_support
    }

    /// Preserved completeness assertion.
    #[must_use]
    pub const fn completeness(&self) -> &CompletenessReference {
        &self.wire.completeness
    }

    /// Preserved availability assertion.
    #[must_use]
    pub const fn availability(&self) -> &AvailabilityReference {
        &self.wire.availability
    }

    /// Preserved decision-scope progress assertion.
    #[must_use]
    pub const fn decision_scope_progress(&self) -> &AxisReference {
        &self.wire.decision_scope_progress
    }

    /// Preserved decision-scope closure assertion.
    #[must_use]
    pub const fn decision_scope_closure(&self) -> &AxisReference {
        &self.wire.decision_scope_closure
    }

    /// Preserved surrounding-execution progress assertion.
    #[must_use]
    pub const fn surrounding_execution_progress(&self) -> &AxisReference {
        &self.wire.surrounding_execution_progress
    }

    /// Preserved surrounding-execution closure assertion.
    #[must_use]
    pub const fn surrounding_execution_closure(&self) -> &AxisReference {
        &self.wire.surrounding_execution_closure
    }

    /// Original, superseding, or invalidating lineage kind.
    #[must_use]
    pub const fn relation_kind(&self) -> ResultRelationKind {
        self.wire.relation.kind
    }

    /// Exact direct predecessor identity for a correction.
    #[must_use]
    pub fn direct_predecessor_identity(&self) -> Option<&str> {
        self.wire.relation.direct_predecessor_identity.as_deref()
    }

    /// Exact canonical bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bounded read/evaluation work.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }

    /// Exact validated request used for re-derivation.
    #[must_use]
    pub const fn request(&self) -> &ValidatedTemporalRequest {
        &self.request
    }

    pub(crate) const fn wire(&self) -> &ResultWire {
        &self.wire
    }
}

/// Evaluates one admitted request and emits one immutable canonical result.
pub fn evaluate(
    request: &ValidatedTemporalRequest,
    relation: ResultRelationInput<'_>,
    limits: OwnerLimits,
) -> Result<TemporalResultDocument, OwnerReadError> {
    let (wire, usage) = build_wire(request, relation, limits.effective())?;
    let document = produce(wire, usage, limits)?;
    let (wire, bytes, usage) = document.into_parts();
    Ok(TemporalResultDocument { wire, bytes, usage })
}

/// Strict-reads a result by re-evaluating its exact request and predecessor relation.
pub fn read(
    bytes: &[u8],
    request: &ValidatedTemporalRequest,
    relation: ResultRelationInput<'_>,
    limits: OwnerLimits,
) -> Result<ValidatedTemporalResult, OwnerReadError> {
    let (expected, expected_usage) = build_wire(request, relation, limits.effective())?;
    let (wire, usage) = read_expected(bytes, &expected, limits, |wire, effective| {
        validate_wire(wire, effective, expected_usage)
    })?;
    Ok(ValidatedTemporalResult {
        wire,
        bytes: bytes.to_vec(),
        usage,
        request: request.clone(),
    })
}

fn build_wire(
    request: &ValidatedTemporalRequest,
    relation: ResultRelationInput<'_>,
    limits: OwnerLimits,
) -> Result<(ResultWire, OwnerUsage), OwnerReadError> {
    if request.usage().formula_nodes > limits.max_formula_nodes
        || request.usage().positions > limits.max_positions
        || request.usage().propositions > limits.max_propositions
    {
        return Err(resource("requestPopulation", request.usage()));
    }
    let (evaluation, evaluation_steps, recursion_depth) = execute(request, limits);
    let (mut execution, mut truth, mut settlement) = match evaluation {
        Ok((truth, settlement)) => (AssessmentExecution::Completed, truth, settlement),
        Err(failure) => (
            failure,
            TemporalTruth::Unavailable,
            SettlementBasis::Unavailable,
        ),
    };
    let mut final_result = execution == AssessmentExecution::Completed
        && matches!(truth, TemporalTruth::Satisfied | TemporalTruth::Violated);
    let mut support = BTreeSet::new();
    let mut support_exceeded = false;
    if final_result {
        for identity in request
            .completeness()
            .facts()
            .iter()
            .filter(|fact| fact.status() == completeness::FactStatus::Available)
            .filter_map(|fact| fact.observation_identity())
        {
            if !support.contains(identity) && support.len() == limits.max_support {
                support_exceeded = true;
                break;
            }
            support.insert(identity.to_owned());
        }
    }
    let mut decision_support: Vec<String> = support.into_iter().collect();
    if support_exceeded {
        execution = AssessmentExecution::ResourceIncomplete;
        truth = TemporalTruth::Unavailable;
        settlement = SettlementBasis::Unavailable;
        final_result = false;
        decision_support.clear();
    }
    let (revision, relation) = relation_wire(request, relation)?;
    let source = request.wire();
    let mut usage = OwnerUsage {
        formula_nodes: request.formula_document().nodes().len(),
        formula_depth: request.usage().formula_depth,
        positions: request.usage().positions,
        propositions: request.usage().propositions,
        support: decision_support.len(),
        history_span: request.usage().history_span,
        evaluation_steps,
        recursion_depth,
        ..OwnerUsage::default()
    };
    if usage.evaluation_steps > limits.max_evaluation_steps
        || usage.recursion_depth > limits.max_recursion_depth
    {
        usage.evaluation_steps = usage.evaluation_steps.min(limits.max_evaluation_steps);
        usage.recursion_depth = usage.recursion_depth.min(limits.max_recursion_depth);
    }
    let mut wire = ResultWire {
        contract_version: CONTRACT.to_owned(),
        identity: String::new(),
        revision,
        request: ArtifactReference {
            contract: super::request::CONTRACT.to_owned(),
            schema_sha256: super::request::SCHEMA_SHA256.to_owned(),
            identity: request.identity().to_owned(),
            revision: 1,
            digest: raw_sha256(request.bytes()),
        },
        lane: source.lane,
        subject_identity: source.subject_identity.clone(),
        correspondence_identity: source.correspondence_identity.clone(),
        formula: source.formula.clone(),
        input: source.input.clone(),
        proposition_map: source.proposition_map.clone(),
        clock: source.clock.clone(),
        evaluator: source.evaluator.clone(),
        anchor: source.anchor,
        execution,
        truth,
        final_result,
        settlement,
        decision_support,
        decision_scope_progress: source.decision_scope_progress.clone(),
        decision_scope_closure: source.decision_scope_closure.clone(),
        surrounding_execution_progress: source.surrounding_execution_progress.clone(),
        surrounding_execution_closure: source.surrounding_execution_closure.clone(),
        completeness: source.completeness.clone(),
        availability: source.availability.clone(),
        relation,
        usage: usage_wire(usage)?,
    };
    wire.identity = identity(CONTRACT, &wire, "identity")?;
    validate_wire(&wire, limits, usage)?;
    Ok((wire, usage))
}

fn execute(
    request: &ValidatedTemporalRequest,
    limits: OwnerLimits,
) -> (
    Result<(TemporalTruth, SettlementBasis), AssessmentExecution>,
    u64,
    u32,
) {
    let formula = match request.formula_document().validate() {
        Ok(formula) => formula,
        Err(_) => return (Err(AssessmentExecution::Refused), 0, 0),
    };
    match request.lane() {
        TemporalLane::Future => {
            let trace = match request.trace() {
                Some(trace) => trace.document(),
                None => return (Err(AssessmentExecution::Refused), 0, 0),
            };
            let evaluator_limits = EvaluationLimits {
                max_node_evaluations: limits.max_evaluation_steps,
                max_temporal_span: limits.max_history_span,
                max_recursion_depth: limits.max_recursion_depth,
            };
            let (result, stats) = match formula.profile() {
                tl_syntax::SemanticProfile::ClosedTraceV1 => evaluate_closed_at_with_stats(
                    formula,
                    request.formula().identity(),
                    &trace.instants,
                    &trace.trace_id,
                    request.wire().anchor,
                    evaluator_limits,
                ),
                tl_syntax::SemanticProfile::OnlinePrefixV1 => evaluate_prefix_at_with_stats(
                    formula,
                    request.formula().identity(),
                    &trace.instants,
                    &trace.trace_id,
                    trace.closed,
                    request.wire().anchor,
                    evaluator_limits,
                ),
                tl_syntax::SemanticProfile::OriginCompleteHistoryV1 => {
                    return (Err(AssessmentExecution::Refused), 0, 0);
                }
            };
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    return (
                        Err(classify_future(error)),
                        stats.node_evaluations,
                        stats.max_recursion_depth,
                    );
                }
            };
            let (truth, settlement) = match result.verdict {
                TruthValue::True if trace.closed => {
                    (TemporalTruth::Satisfied, SettlementBasis::ClosedScope)
                }
                TruthValue::False if trace.closed => {
                    (TemporalTruth::Violated, SettlementBasis::ClosedScope)
                }
                TruthValue::True => (TemporalTruth::Satisfied, SettlementBasis::DecisiveWitness),
                TruthValue::False => (
                    TemporalTruth::Violated,
                    SettlementBasis::DecisiveCounterexample,
                ),
                TruthValue::Pending => (TemporalTruth::Pending, SettlementBasis::Unsettled),
            };
            (
                Ok((truth, settlement)),
                stats.node_evaluations,
                stats.max_recursion_depth,
            )
        }
        TemporalLane::Past => {
            let history = match request.history() {
                Some(history) => history,
                None => return (Err(AssessmentExecution::Refused), 0, 0),
            };
            let (result, stats) = evaluate_past_with_stats(
                formula,
                request.formula().identity(),
                history,
                request.wire().anchor,
                request.proposition_map().identity(),
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits {
                    max_steps: limits.max_evaluation_steps,
                    max_temporal_span: limits.max_history_span,
                    max_recursion_depth: limits.max_recursion_depth,
                    max_input_positions: limits.max_positions,
                },
            );
            let result = match result {
                Ok(result) => result,
                Err(error) => {
                    return (
                        Err(classify_past(error)),
                        stats.steps.min(limits.max_evaluation_steps),
                        stats.max_recursion_depth.min(limits.max_recursion_depth),
                    );
                }
            };
            (
                Ok((
                    if result.verdict {
                        TemporalTruth::Satisfied
                    } else {
                        TemporalTruth::Violated
                    },
                    SettlementBasis::ClosedScope,
                )),
                result.stats.steps,
                result.stats.max_recursion_depth,
            )
        }
    }
}

fn classify_future(error: EvaluationError) -> AssessmentExecution {
    match error {
        EvaluationError::TemporalSpanExceeded { .. }
        | EvaluationError::WorkLimitExceeded { .. }
        | EvaluationError::RecursionDepthExceeded { .. } => AssessmentExecution::ResourceIncomplete,
        EvaluationError::UnsupportedProfile { .. } | EvaluationError::UnsupportedPastNode(_) => {
            AssessmentExecution::Unsupported
        }
        EvaluationError::TraceNotStrictlyOrdered { .. } => AssessmentExecution::Refused,
        EvaluationError::TimeOverflow
        | EvaluationError::Horizon(_)
        | EvaluationError::InvalidNodeReference(_) => AssessmentExecution::Failed,
    }
}

fn classify_past(error: PastEvaluationError) -> AssessmentExecution {
    match error {
        PastEvaluationError::InputPositionLimitExceeded { .. }
        | PastEvaluationError::TemporalSpanExceeded { .. }
        | PastEvaluationError::StepLimitExceeded { .. }
        | PastEvaluationError::RecursionDepthExceeded { .. } => {
            AssessmentExecution::ResourceIncomplete
        }
        PastEvaluationError::OwnerStatePreserved {
            state: OwnerHistoryState::Unsupported,
        }
        | PastEvaluationError::FutureNodeUnsupported { .. } => AssessmentExecution::Unsupported,
        PastEvaluationError::OwnerStatePreserved { .. }
        | PastEvaluationError::History(_)
        | PastEvaluationError::Requirement(_)
        | PastEvaluationError::AnchorOutOfRange { .. }
        | PastEvaluationError::InvalidIdentity { .. }
        | PastEvaluationError::ResultRevisionInvalid
        | PastEvaluationError::InvalidPredecessor(_)
        | PastEvaluationError::CorrectionContextMismatch { .. }
        | PastEvaluationError::HistoryRevisionNotAdvanced => AssessmentExecution::Refused,
        PastEvaluationError::InvalidNodeReference { .. }
        | PastEvaluationError::PositionArithmeticOverflow
        | PastEvaluationError::HistoryPositionAbsent { .. }
        | PastEvaluationError::Result(_) => AssessmentExecution::Failed,
    }
}

fn relation_wire(
    request: &ValidatedTemporalRequest,
    input: ResultRelationInput<'_>,
) -> Result<(u64, RelationWire), OwnerReadError> {
    let (predecessor, kind) = match input {
        ResultRelationInput::Original => {
            return Ok((
                1,
                RelationWire {
                    kind: ResultRelationKind::Original,
                    direct_predecessor_identity: None,
                    direct_predecessor_digest: None,
                    corrected_request_identity: None,
                },
            ));
        }
        ResultRelationInput::Superseding(predecessor) => {
            (predecessor, ResultRelationKind::Superseding)
        }
        ResultRelationInput::Invalidating(predecessor) => {
            (predecessor, ResultRelationKind::Invalidating)
        }
    };
    let previous = predecessor.wire();
    let current = request.wire();
    if previous.subject_identity != current.subject_identity
        || previous.correspondence_identity != current.correspondence_identity
        || previous.formula != current.formula
        || previous.proposition_map != current.proposition_map
        || previous.clock != current.clock
        || previous.evaluator != current.evaluator
        || previous.anchor != current.anchor
        || previous.lane != current.lane
    {
        return Err(expected("correctionContext", predecessor.usage()));
    }
    let revision = predecessor
        .revision()
        .checked_add(1)
        .ok_or_else(|| resource("resultRevision", predecessor.usage()))?;
    Ok((
        revision,
        RelationWire {
            kind,
            direct_predecessor_identity: Some(predecessor.identity().to_owned()),
            direct_predecessor_digest: Some(raw_sha256(predecessor.bytes())),
            corrected_request_identity: Some(request.identity().to_owned()),
        },
    ))
}

fn validate_wire(
    wire: &ResultWire,
    limits: OwnerLimits,
    usage: OwnerUsage,
) -> Result<OwnerUsage, OwnerReadError> {
    if wire.contract_version != CONTRACT {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ContractMismatch,
            "contractVersion",
            usage,
        ));
    }
    if wire.revision == 0
        || !is_sha256(&wire.identity)
        || identity(CONTRACT, wire, "identity")? != wire.identity
    {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::IdentityMismatch,
            "identity",
            usage,
        ));
    }
    let valid_execution = match wire.execution {
        AssessmentExecution::Completed => {
            matches!(
                wire.truth,
                TemporalTruth::Satisfied | TemporalTruth::Violated | TemporalTruth::Pending
            ) && wire.settlement != SettlementBasis::Unavailable
        }
        _ => {
            wire.truth == TemporalTruth::Unavailable
                && wire.settlement == SettlementBasis::Unavailable
                && !wire.final_result
                && wire.decision_support.is_empty()
        }
    };
    let valid_final = wire.final_result
        == (wire.execution == AssessmentExecution::Completed
            && matches!(
                wire.truth,
                TemporalTruth::Satisfied | TemporalTruth::Violated
            ));
    let lane_matches_input = matches!(
        (wire.lane, &wire.input),
        (TemporalLane::Future, InputReference::Future { .. })
            | (TemporalLane::Past, InputReference::Past { .. })
    );
    let valid_settlement = match (wire.execution, wire.lane, &wire.input, wire.truth) {
        (
            AssessmentExecution::Completed,
            TemporalLane::Past,
            InputReference::Past { .. },
            TemporalTruth::Satisfied | TemporalTruth::Violated,
        ) => wire.settlement == SettlementBasis::ClosedScope,
        (
            AssessmentExecution::Completed,
            TemporalLane::Future,
            InputReference::Future { closed: true, .. },
            TemporalTruth::Satisfied | TemporalTruth::Violated,
        ) => wire.settlement == SettlementBasis::ClosedScope,
        (
            AssessmentExecution::Completed,
            TemporalLane::Future,
            InputReference::Future { closed: false, .. },
            TemporalTruth::Satisfied,
        ) => wire.settlement == SettlementBasis::DecisiveWitness,
        (
            AssessmentExecution::Completed,
            TemporalLane::Future,
            InputReference::Future { closed: false, .. },
            TemporalTruth::Violated,
        ) => wire.settlement == SettlementBasis::DecisiveCounterexample,
        (
            AssessmentExecution::Completed,
            TemporalLane::Future,
            InputReference::Future { .. },
            TemporalTruth::Pending,
        ) => wire.settlement == SettlementBasis::Unsettled,
        (execution, _, _, TemporalTruth::Unavailable)
            if execution != AssessmentExecution::Completed =>
        {
            wire.settlement == SettlementBasis::Unavailable
        }
        _ => false,
    };
    let sorted_unique = wire
        .decision_support
        .windows(2)
        .all(|pair| pair[0] < pair[1]);
    let support_matches_finality = wire.final_result || wire.decision_support.is_empty();
    let relation_valid = match wire.relation.kind {
        ResultRelationKind::Original => {
            wire.revision == 1
                && wire.relation.direct_predecessor_identity.is_none()
                && wire.relation.direct_predecessor_digest.is_none()
                && wire.relation.corrected_request_identity.is_none()
        }
        ResultRelationKind::Superseding | ResultRelationKind::Invalidating => {
            wire.revision > 1
                && wire.relation.direct_predecessor_identity.is_some()
                && wire
                    .relation
                    .direct_predecessor_identity
                    .as_deref()
                    .is_some_and(is_sha256)
                && wire
                    .relation
                    .direct_predecessor_digest
                    .as_deref()
                    .is_some_and(is_sha256)
                && wire.relation.corrected_request_identity.as_deref()
                    == Some(wire.request.identity())
        }
    };
    if !valid_execution
        || !valid_final
        || !lane_matches_input
        || !valid_settlement
        || !sorted_unique
        || !support_matches_finality
        || !relation_valid
    {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::InvalidCombination,
            "resultState",
            usage,
        ));
    }
    if wire.usage != usage_wire(usage)?
        || usage.support > limits.max_support
        || usage.evaluation_steps > limits.max_evaluation_steps
        || usage.recursion_depth > limits.max_recursion_depth
    {
        return Err(resource("resultUsage", usage));
    }
    Ok(usage)
}

fn usage_wire(usage: OwnerUsage) -> Result<UsageWire, OwnerReadError> {
    Ok(UsageWire {
        wire_bytes: persistent_usize(usage.wire_bytes, usage)?,
        depth: persistent_usize(usage.depth, usage)?,
        string_bytes: persistent_usize(usage.string_bytes, usage)?,
        formula_nodes: persistent_usize(usage.formula_nodes, usage)?,
        formula_depth: persistent_usize(usage.formula_depth, usage)?,
        positions: persistent_usize(usage.positions, usage)?,
        propositions: persistent_usize(usage.propositions, usage)?,
        support: persistent_usize(usage.support, usage)?,
        history_span: usage.history_span,
        evaluation_steps: usage.evaluation_steps,
        recursion_depth: usage.recursion_depth,
        visited_fields: persistent_usize(usage.visited_fields, usage)?,
    })
}

fn persistent_usize(value: usize, usage: OwnerUsage) -> Result<u64, OwnerReadError> {
    u64::try_from(value).map_err(|_| resource("usageConversion", usage))
}

fn resource(field: &'static str, usage: OwnerUsage) -> OwnerReadError {
    OwnerReadError::new(OwnerReadErrorCode::ResourceIncomplete, field, usage)
}

fn expected(field: &'static str, usage: OwnerUsage) -> OwnerReadError {
    OwnerReadError::new(OwnerReadErrorCode::ExpectedMismatch, field, usage)
}
