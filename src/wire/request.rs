//! Canonical `tl-mltl.temporal-assessment-request/v1` owner boundary.

use quire_observation::authority::{
    availability, clock, closure, completeness, progress, OpenClosed,
};
use serde::{Deserialize, Serialize};
use tl_syntax::{
    FormulaDocument, FormulaSchemaVersion, PropositionMapDocument, SemanticProfile,
    StrictDocumentReadError, SyntaxArtifactLimits, PAST_OPERATORS_V1,
    PROPOSITION_MAP_V1_SCHEMA_SHA256,
};

use super::common::{identity, is_sha256, produce, raw_sha256, read_expected};
use super::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage, ValidatedTrace};
use crate::{
    past::history::ValidatedPositionHistory, ClockBinding, PositionHistoryDocument,
    PAST_EVALUATOR_V1, QUIRE_OBSERVATION_REVISION, TL_SYNTAX_REVISION,
};

/// Immutable request contract label.
pub const CONTRACT: &str = "tl-mltl.temporal-assessment-request/v1";
/// Exact checked-in JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/temporal-assessment-request-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "c260dcf90bdd14cf6851e686cb1c6c7a570ed11c9e7b7a2f75dae92e3a7c4be8";

/// Closed temporal evaluator lane.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TemporalLane {
    /// Formula-v1 over a finite future trace.
    Future,
    /// Formula-v2 over an origin-complete history.
    Past,
}

/// One exact immutable owner artifact reference.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ArtifactReference {
    pub(crate) contract: String,
    pub(crate) schema_sha256: String,
    pub(crate) identity: String,
    pub(crate) revision: u64,
    pub(crate) digest: String,
}

/// Exact evaluator implementation selection, distinct from a wire artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvaluatorReference {
    identity: String,
    revision: String,
    implementation_digest: String,
}

impl EvaluatorReference {
    /// Closed evaluator identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Exact source revision compiled into the evaluator.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Digest binding the exact selected implementation revision.
    #[must_use]
    pub fn implementation_digest(&self) -> &str {
        &self.implementation_digest
    }
}

impl ArtifactReference {
    /// Selected owner contract.
    #[must_use]
    pub fn contract(&self) -> &str {
        &self.contract
    }

    /// Exact schema digest.
    #[must_use]
    pub fn schema_sha256(&self) -> &str {
        &self.schema_sha256
    }

    /// Exact document/content identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.identity
    }

    /// Positive immutable revision, or one for content-addressed v1 artifacts.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Digest of exact canonical owner bytes.
    #[must_use]
    pub fn digest(&self) -> &str {
        &self.digest
    }
}

/// One independently selected progress or closure assertion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AxisReference {
    artifact: ArtifactReference,
    authority_identity: String,
    authority_revision: String,
    authority_digest: String,
    scope_identity: String,
    population_identity: String,
    state: OpenClosed,
}

impl AxisReference {
    /// Exact assertion artifact.
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactReference {
        &self.artifact
    }

    /// Exact owner authority definition identity.
    #[must_use]
    pub fn authority_identity(&self) -> &str {
        &self.authority_identity
    }

    /// Exact owner authority definition revision.
    #[must_use]
    pub fn authority_revision(&self) -> &str {
        &self.authority_revision
    }

    /// Exact owner authority definition digest.
    #[must_use]
    pub fn authority_digest(&self) -> &str {
        &self.authority_digest
    }

    /// Independent owner state.
    #[must_use]
    pub const fn state(&self) -> OpenClosed {
        self.state
    }

    /// Exact scope identity.
    #[must_use]
    pub fn scope_identity(&self) -> &str {
        &self.scope_identity
    }

    /// Exact population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }
}

/// One exact owner completeness fact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletenessFactReference {
    member_identity: String,
    observation_identity: Option<String>,
    status: completeness::FactStatus,
}

impl CompletenessFactReference {
    /// Required member identity.
    #[must_use]
    pub fn member_identity(&self) -> &str {
        &self.member_identity
    }

    /// Exact qualified observation identity, when available.
    #[must_use]
    pub fn observation_identity(&self) -> Option<&str> {
        self.observation_identity.as_deref()
    }

    /// Owner fact status.
    #[must_use]
    pub const fn status(&self) -> completeness::FactStatus {
        self.status
    }
}

/// Exact completeness owner assertion and complete fact population.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CompletenessReference {
    artifact: ArtifactReference,
    population_identity: String,
    boundary_identity: String,
    facts: Vec<CompletenessFactReference>,
    state: completeness::State,
}

impl CompletenessReference {
    /// Exact assertion artifact.
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactReference {
        &self.artifact
    }

    /// Independent aggregate state.
    #[must_use]
    pub const fn state(&self) -> completeness::State {
        self.state
    }

    /// Exact required population identity.
    #[must_use]
    pub fn population_identity(&self) -> &str {
        &self.population_identity
    }

    /// Exact selected completeness boundary identity.
    #[must_use]
    pub fn boundary_identity(&self) -> &str {
        &self.boundary_identity
    }

    /// Complete owner fact population.
    #[must_use]
    pub fn facts(&self) -> &[CompletenessFactReference] {
        &self.facts
    }
}

/// Exact result-availability owner assertion.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AvailabilityReference {
    artifact: ArtifactReference,
    required_results: Vec<String>,
    available_results: Vec<String>,
    state: availability::State,
}

impl AvailabilityReference {
    /// Exact assertion artifact.
    #[must_use]
    pub const fn artifact(&self) -> &ArtifactReference {
        &self.artifact
    }

    /// Independent availability state.
    #[must_use]
    pub const fn state(&self) -> availability::State {
        self.state
    }

    /// Required result identities.
    #[must_use]
    pub fn required_results(&self) -> &[String] {
        &self.required_results
    }

    /// Available result identities.
    #[must_use]
    pub fn available_results(&self) -> &[String] {
        &self.available_results
    }
}

/// Four independent observation-owner scope axes plus completeness/availability.
#[derive(Clone, Copy, Debug)]
pub struct ObservationInputs<'a> {
    /// Decision-scope progress.
    pub decision_scope_progress: &'a progress::View,
    /// Decision-scope closure.
    pub decision_scope_closure: &'a closure::View,
    /// Surrounding-execution progress.
    pub surrounding_execution_progress: &'a progress::View,
    /// Surrounding-execution closure.
    pub surrounding_execution_closure: &'a closure::View,
    /// Independent completeness assertion.
    pub completeness: &'a completeness::View,
    /// Independent result-availability assertion.
    pub availability: &'a availability::View,
}

/// Exact owner-read valuation input selected by a request.
#[derive(Clone, Copy, Debug)]
pub enum TemporalInput<'a> {
    /// Validated future trace.
    Future(&'a ValidatedTrace),
    /// Validated origin-complete history.
    Past(&'a ValidatedPositionHistory),
}

/// Complete input used to derive or independently strict-read a request.
#[derive(Clone, Copy, Debug)]
pub struct RequestInput<'a> {
    /// Owner-read formula document.
    pub formula: &'a FormulaDocument,
    /// Owner-read proposition map.
    pub proposition_map: &'a PropositionMapDocument,
    /// Owner-read trace or history.
    pub input: TemporalInput<'a>,
    /// Owner-read clock binding.
    pub clock: &'a clock::View,
    /// Exact native subject identity.
    pub subject_identity: &'a str,
    /// Exact native/TL correspondence identity.
    pub correspondence_identity: &'a str,
    /// Explicit evaluation anchor.
    pub anchor: u64,
    /// All independent observation-owner facts.
    pub observations: ObservationInputs<'a>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub(crate) enum InputReference {
    Future {
        trace: ArtifactReference,
        closed: bool,
    },
    Past {
        history: ArtifactReference,
        origin_position: u64,
        through_position: u64,
    },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct LimitsWire {
    max_input_bytes: u64,
    max_output_bytes: u64,
    max_depth: u64,
    max_string_bytes: u64,
    max_formula_nodes: u64,
    max_formula_depth: u64,
    max_positions: u64,
    max_propositions: u64,
    max_support: u64,
    max_history_span: u64,
    max_evaluation_steps: u64,
    max_recursion_depth: u32,
    max_visited_fields: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct RequestWire {
    pub(crate) contract_version: String,
    pub(crate) identity: String,
    pub(crate) lane: TemporalLane,
    pub(crate) semantic_profile: SemanticProfile,
    pub(crate) operator_profile: String,
    pub(crate) formula: ArtifactReference,
    pub(crate) input: InputReference,
    pub(crate) proposition_map: ArtifactReference,
    pub(crate) clock: ArtifactReference,
    pub(crate) clock_identity: String,
    pub(crate) clock_revision: String,
    pub(crate) subject_identity: String,
    pub(crate) correspondence_identity: String,
    pub(crate) anchor: u64,
    pub(crate) evaluator: EvaluatorReference,
    pub(crate) syntax_revision: String,
    pub(crate) observation_revision: String,
    pub(crate) decision_scope_progress: AxisReference,
    pub(crate) decision_scope_closure: AxisReference,
    pub(crate) surrounding_execution_progress: AxisReference,
    pub(crate) surrounding_execution_closure: AxisReference,
    pub(crate) completeness: CompletenessReference,
    pub(crate) availability: AvailabilityReference,
    pub(crate) limits: LimitsWire,
}

/// Immutable canonical request bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalRequestDocument {
    wire: RequestWire,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl TemporalRequestDocument {
    /// Request content identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.wire.identity
    }

    /// Exact canonical bytes.
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

#[derive(Clone, Debug, Eq, PartialEq)]
enum RuntimeInput {
    Future(ValidatedTrace),
    Past(ValidatedPositionHistory),
}

/// Constructor-private request admitted against all independently supplied owners.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedTemporalRequest {
    wire: RequestWire,
    bytes: Vec<u8>,
    usage: OwnerUsage,
    formula: FormulaDocument,
    proposition_map: PropositionMapDocument,
    input: RuntimeInput,
}

impl ValidatedTemporalRequest {
    /// Request identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.wire.identity
    }

    /// Selected evaluator lane.
    #[must_use]
    pub const fn lane(&self) -> TemporalLane {
        self.wire.lane
    }

    /// Exact selected semantic profile.
    #[must_use]
    pub const fn semantic_profile(&self) -> SemanticProfile {
        self.wire.semantic_profile
    }

    /// Exact selected operator profile identity.
    #[must_use]
    pub fn operator_profile(&self) -> &str {
        &self.wire.operator_profile
    }

    /// Exact formula identity.
    #[must_use]
    pub fn formula(&self) -> &ArtifactReference {
        &self.wire.formula
    }

    /// Exact proposition-map identity.
    #[must_use]
    pub fn proposition_map(&self) -> &ArtifactReference {
        &self.wire.proposition_map
    }

    /// Exact trace or history owner artifact selected by the request.
    #[must_use]
    pub fn input_artifact(&self) -> &ArtifactReference {
        match &self.wire.input {
            InputReference::Future { trace, .. } => trace,
            InputReference::Past { history, .. } => history,
        }
    }

    /// Exact selected clock owner artifact.
    #[must_use]
    pub const fn clock(&self) -> &ArtifactReference {
        &self.wire.clock
    }

    /// Exact clock identity carried independently from the artifact envelope.
    #[must_use]
    pub fn clock_identity(&self) -> &str {
        &self.wire.clock_identity
    }

    /// Exact clock revision carried independently from the artifact envelope.
    #[must_use]
    pub fn clock_revision(&self) -> &str {
        &self.wire.clock_revision
    }

    /// Exact selected evaluator implementation.
    #[must_use]
    pub const fn evaluator(&self) -> &EvaluatorReference {
        &self.wire.evaluator
    }

    /// Native subject identity evaluated by TL.
    #[must_use]
    pub fn subject_identity(&self) -> &str {
        &self.wire.subject_identity
    }

    /// Native/TL correspondence identity retained for joining.
    #[must_use]
    pub fn correspondence_identity(&self) -> &str {
        &self.wire.correspondence_identity
    }

    /// Explicit selected evaluation anchor.
    #[must_use]
    pub const fn anchor(&self) -> u64 {
        self.wire.anchor
    }

    /// Decision-scope progress assertion.
    #[must_use]
    pub fn decision_scope_progress(&self) -> &AxisReference {
        &self.wire.decision_scope_progress
    }

    /// Decision-scope closure assertion.
    #[must_use]
    pub fn decision_scope_closure(&self) -> &AxisReference {
        &self.wire.decision_scope_closure
    }

    /// Surrounding-execution progress assertion.
    #[must_use]
    pub fn surrounding_execution_progress(&self) -> &AxisReference {
        &self.wire.surrounding_execution_progress
    }

    /// Surrounding-execution closure assertion.
    #[must_use]
    pub fn surrounding_execution_closure(&self) -> &AxisReference {
        &self.wire.surrounding_execution_closure
    }

    /// Completeness assertion and fact population.
    #[must_use]
    pub fn completeness(&self) -> &CompletenessReference {
        &self.wire.completeness
    }

    /// Result-availability assertion.
    #[must_use]
    pub fn availability(&self) -> &AvailabilityReference {
        &self.wire.availability
    }

    /// Exact canonical request bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bounded strict-read work.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }

    pub(crate) const fn wire(&self) -> &RequestWire {
        &self.wire
    }

    pub(crate) const fn formula_document(&self) -> &FormulaDocument {
        &self.formula
    }

    pub(crate) fn trace(&self) -> Option<&ValidatedTrace> {
        match &self.input {
            RuntimeInput::Future(trace) => Some(trace),
            RuntimeInput::Past(_) => None,
        }
    }

    pub(crate) fn history(&self) -> Option<&PositionHistoryDocument> {
        match &self.input {
            RuntimeInput::Future(_) => None,
            RuntimeInput::Past(history) => Some(history.document()),
        }
    }
}

/// Produces one canonical temporal request from owner-read inputs.
pub fn derive(
    input: RequestInput<'_>,
    limits: OwnerLimits,
) -> Result<TemporalRequestDocument, OwnerReadError> {
    let (wire, usage) = build_wire(input, limits.effective())?;
    let document = produce(wire, usage, limits)?;
    let (wire, bytes, usage) = document.into_parts();
    Ok(TemporalRequestDocument { wire, bytes, usage })
}

/// Strict-reads a request against independently supplied exact owner views.
pub fn read(
    bytes: &[u8],
    expected: RequestInput<'_>,
    limits: OwnerLimits,
) -> Result<ValidatedTemporalRequest, OwnerReadError> {
    let (expected_wire, expected_usage) = build_wire(expected, limits.effective())?;
    let (wire, usage) = read_expected(bytes, &expected_wire, limits, |wire, effective| {
        validate_wire(wire, effective, expected_usage)
    })?;
    let runtime = match expected.input {
        TemporalInput::Future(trace) => RuntimeInput::Future(trace.clone()),
        TemporalInput::Past(history) => RuntimeInput::Past(history.clone()),
    };
    Ok(ValidatedTemporalRequest {
        wire,
        bytes: bytes.to_vec(),
        usage,
        formula: expected.formula.clone(),
        proposition_map: expected.proposition_map.clone(),
        input: runtime,
    })
}

fn build_wire(
    input: RequestInput<'_>,
    limits: OwnerLimits,
) -> Result<(RequestWire, OwnerUsage), OwnerReadError> {
    validate_identity_text(input.subject_identity, "subjectIdentity")?;
    validate_identity_text(input.correspondence_identity, "correspondenceIdentity")?;
    let formula_bytes = input
        .formula
        .canonical_json_bytes()
        .map_err(|_| encoding("formula"))?;
    let formula = FormulaDocument::from_json_bytes(&formula_bytes, syntax_limits(limits))
        .map_err(|error| syntax_error(error, "formula"))?;
    let proposition_bytes = input
        .proposition_map
        .canonical_json_bytes()
        .map_err(|_| encoding("propositionMap"))?;
    let proposition_map =
        PropositionMapDocument::from_json_bytes(&proposition_bytes, syntax_limits(limits))
            .map_err(|error| syntax_error(error, "propositionMap"))?;
    validate_formula_bindings(&formula, &proposition_map)?;
    let formula_depth = formula_graph_depth(&formula)?;

    let (lane, operator_profile, input_reference, positions, propositions, history_span) =
        match input.input {
            TemporalInput::Future(trace) => {
                if formula.schema_version() != FormulaSchemaVersion::V1
                    || formula.semantic_profile() == SemanticProfile::OriginCompleteHistoryV1
                    || formula
                        .nodes()
                        .iter()
                        .any(|node| node.kind.past_operator().is_some())
                {
                    return Err(mismatch("futureFormulaProfile"));
                }
                let document = trace.document();
                if formula.semantic_profile() == SemanticProfile::ClosedTraceV1 && !document.closed
                {
                    return Err(mismatch("closedTrace"));
                }
                (
                    TemporalLane::Future,
                    formula.semantic_profile().as_str().to_owned(),
                    InputReference::Future {
                        trace: ArtifactReference {
                            contract: super::trace::CONTRACT.to_owned(),
                            schema_sha256: super::trace::SCHEMA_SHA256.to_owned(),
                            identity: document.trace_id.clone(),
                            revision: 1,
                            digest: raw_sha256(trace.bytes()),
                        },
                        closed: document.closed,
                    },
                    trace.usage().positions,
                    trace.usage().propositions,
                    0,
                )
            }
            TemporalInput::Past(history) => {
                let history_document = history.document();
                if formula.schema_version() != FormulaSchemaVersion::V2
                    || formula.semantic_profile() != SemanticProfile::OriginCompleteHistoryV1
                    || formula.nodes().iter().any(|node| {
                        node.kind.temporal_family() == Some(tl_syntax::TemporalFamily::Future)
                    })
                {
                    return Err(mismatch("pastFormulaProfile"));
                }
                if input.anchor > history_document.through_position() {
                    return Err(mismatch("anchor"));
                }
                let history_bytes = history.bytes();
                (
                    TemporalLane::Past,
                    PAST_OPERATORS_V1.to_owned(),
                    InputReference::Past {
                        history: ArtifactReference {
                            contract: crate::POSITION_HISTORY_V1.to_owned(),
                            schema_sha256: crate::past::history::SCHEMA_SHA256.to_owned(),
                            identity: history_document.history_id().to_owned(),
                            revision: history_document.revision(),
                            digest: raw_sha256(history_bytes),
                        },
                        origin_position: 0,
                        through_position: history_document.through_position(),
                    },
                    history.usage().positions,
                    history.usage().propositions,
                    history.usage().history_span,
                )
            }
        };
    if positions > limits.max_positions
        || propositions > limits.max_propositions
        || formula.nodes().len() > limits.max_formula_nodes
        || history_span > limits.max_history_span
    {
        return Err(resource("requestPopulation"));
    }
    validate_clock_scope(input.clock, input.observations, input.input, positions)?;

    let formula_contract = formula.schema_version().as_str();
    let formula_schema = match formula.schema_version() {
        FormulaSchemaVersion::V1 => tl_syntax::FORMULA_V1_SCHEMA_SHA256,
        FormulaSchemaVersion::V2 => tl_syntax::FORMULA_V2_SCHEMA_SHA256,
    };
    let formula_ref = ArtifactReference {
        contract: formula_contract.to_owned(),
        schema_sha256: formula_schema.to_owned(),
        identity: formula
            .content_identity()
            .map_err(|_| encoding("formulaIdentity"))?,
        revision: 1,
        digest: raw_sha256(&formula_bytes),
    };
    let proposition_ref = ArtifactReference {
        contract: input.proposition_map.schema_version().as_str().to_owned(),
        schema_sha256: PROPOSITION_MAP_V1_SCHEMA_SHA256.to_owned(),
        identity: proposition_map
            .content_identity()
            .map_err(|_| encoding("propositionMapIdentity"))?,
        revision: 1,
        digest: raw_sha256(&proposition_bytes),
    };
    let mut wire = RequestWire {
        contract_version: CONTRACT.to_owned(),
        identity: String::new(),
        lane,
        semantic_profile: formula.semantic_profile(),
        operator_profile,
        formula: formula_ref,
        input: input_reference,
        proposition_map: proposition_ref,
        clock: artifact_from_view(clock::CONTRACT, clock::SCHEMA_SHA256, input.clock),
        clock_identity: input.clock.payload().clock_identity().to_owned(),
        clock_revision: input.clock.payload().clock_revision().to_owned(),
        subject_identity: input.subject_identity.to_owned(),
        correspondence_identity: input.correspondence_identity.to_owned(),
        anchor: input.anchor,
        evaluator: EvaluatorReference {
            identity: if lane == TemporalLane::Past {
                PAST_EVALUATOR_V1.to_owned()
            } else {
                "tl-mltl.future-evaluator/v1".to_owned()
            },
            revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
            implementation_digest: raw_sha256(env!("TL_MLTL_SOURCE_REVISION").as_bytes()),
        },
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        observation_revision: QUIRE_OBSERVATION_REVISION.to_owned(),
        decision_scope_progress: progress_axis(input.observations.decision_scope_progress),
        decision_scope_closure: closure_axis(input.observations.decision_scope_closure),
        surrounding_execution_progress: progress_axis(
            input.observations.surrounding_execution_progress,
        ),
        surrounding_execution_closure: closure_axis(
            input.observations.surrounding_execution_closure,
        ),
        completeness: completeness_ref(input.observations.completeness),
        availability: availability_ref(input.observations.availability),
        limits: limits_wire(limits)?,
    };
    wire.identity = identity(CONTRACT, &wire, "identity")?;
    let usage = OwnerUsage {
        formula_nodes: formula.nodes().len(),
        formula_depth,
        positions,
        propositions,
        history_span,
        support: wire.completeness.facts.len(),
        ..OwnerUsage::default()
    };
    validate_wire(&wire, limits, usage)?;
    Ok((wire, usage))
}

fn validate_wire(
    wire: &RequestWire,
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
    if wire.syntax_revision != TL_SYNTAX_REVISION
        || wire.observation_revision != QUIRE_OBSERVATION_REVISION
        || wire.limits != limits_wire(limits)?
    {
        return Err(mismatch_with_usage("dependencyOrLimits", usage));
    }
    if !is_sha256(&wire.identity) || identity(CONTRACT, wire, "identity")? != wire.identity {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::IdentityMismatch,
            "identity",
            usage,
        ));
    }
    for artifact in [
        &wire.formula,
        &wire.proposition_map,
        &wire.clock,
        &wire.decision_scope_progress.artifact,
        &wire.decision_scope_closure.artifact,
        &wire.surrounding_execution_progress.artifact,
        &wire.surrounding_execution_closure.artifact,
        &wire.completeness.artifact,
        &wire.availability.artifact,
    ] {
        validate_artifact(artifact, usage)?;
    }
    if wire.evaluator.identity.is_empty()
        || wire.evaluator.revision.is_empty()
        || !is_sha256(&wire.evaluator.implementation_digest)
    {
        return Err(mismatch_with_usage("evaluator", usage));
    }
    Ok(usage)
}

fn validate_formula_bindings(
    formula: &FormulaDocument,
    propositions: &PropositionMapDocument,
) -> Result<(), OwnerReadError> {
    let known: std::collections::BTreeSet<_> = propositions
        .propositions()
        .iter()
        .map(|entry| entry.id)
        .collect();
    if formula
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            tl_syntax::NodeKind::Proposition { proposition } => Some(proposition),
            _ => None,
        })
        .any(|id| !known.contains(&id))
    {
        return Err(mismatch("propositionMap"));
    }
    Ok(())
}

fn formula_graph_depth(formula: &FormulaDocument) -> Result<usize, OwnerReadError> {
    let mut depths = Vec::with_capacity(formula.nodes().len());
    for node in formula.nodes() {
        let operands = match node.kind {
            tl_syntax::NodeKind::False
            | tl_syntax::NodeKind::True
            | tl_syntax::NodeKind::Proposition { .. } => [None, None],
            tl_syntax::NodeKind::Not { operand }
            | tl_syntax::NodeKind::Future { operand, .. }
            | tl_syntax::NodeKind::Globally { operand, .. }
            | tl_syntax::NodeKind::Once { operand, .. }
            | tl_syntax::NodeKind::Historically { operand, .. }
            | tl_syntax::NodeKind::StrongPrevious { operand } => [Some(operand), None],
            tl_syntax::NodeKind::And { left, right }
            | tl_syntax::NodeKind::Or { left, right }
            | tl_syntax::NodeKind::Implies { left, right }
            | tl_syntax::NodeKind::Equivalent { left, right }
            | tl_syntax::NodeKind::Until { left, right, .. }
            | tl_syntax::NodeKind::Release { left, right, .. }
            | tl_syntax::NodeKind::Since { left, right, .. }
            | tl_syntax::NodeKind::Triggered { left, right, .. } => [Some(left), Some(right)],
        };
        let mut parent_depth = 0usize;
        for operand in operands.into_iter().flatten() {
            let depth = usize::try_from(operand.0)
                .ok()
                .and_then(|index| depths.get(index))
                .copied()
                .ok_or_else(|| mismatch("formulaTopology"))?;
            parent_depth = parent_depth.max(depth);
        }
        depths.push(
            parent_depth
                .checked_add(1)
                .ok_or_else(|| resource("formulaDepth"))?,
        );
    }
    usize::try_from(formula.root().0)
        .ok()
        .and_then(|index| depths.get(index))
        .copied()
        .ok_or_else(|| mismatch("formulaRoot"))
}

fn validate_clock_scope(
    clock: &clock::View,
    observations: ObservationInputs<'_>,
    input: TemporalInput<'_>,
    positions: usize,
) -> Result<(), OwnerReadError> {
    validate_axis_pair(
        observations.decision_scope_progress,
        observations.decision_scope_closure,
        clock,
        "decisionScope",
    )?;
    let decision_subject = observations.decision_scope_progress.subject();
    let decision_authority = observations.decision_scope_progress.authority();
    if clock.subject() != decision_subject
        || clock.authority() != decision_authority
        || observations.completeness.subject() != decision_subject
        || observations.completeness.authority() != decision_authority
        || observations.availability.subject() != decision_subject
        || observations.availability.authority() != decision_authority
        || observations.completeness.payload().population_identity()
            != decision_subject.population_identity.as_str()
    {
        return Err(mismatch("decisionEvidenceContext"));
    }
    validate_axis_pair(
        observations.surrounding_execution_progress,
        observations.surrounding_execution_closure,
        clock,
        "surroundingExecution",
    )?;
    let selected_positions = match clock.payload().selection() {
        clock::RangeRef::EventPosition {
            start,
            end_exclusive,
        }
        | clock::RangeRef::FixedSample {
            start,
            end_exclusive,
            ..
        } => {
            let start = start.parse::<u64>().map_err(|_| mismatch("clockRange"))?;
            let end = end_exclusive
                .parse::<u64>()
                .map_err(|_| mismatch("clockRange"))?;
            end.checked_sub(start)
                .ok_or_else(|| mismatch("clockRange"))?
        }
        clock::RangeRef::TimestampedEvent { .. } => return Err(mismatch("clockFamily")),
    };
    if usize::try_from(selected_positions).ok() != Some(positions) {
        return Err(mismatch("clockRange"));
    }
    if let TemporalInput::Past(history) = input {
        match (clock.payload().selection(), history.document().clock()) {
            (clock::RangeRef::EventPosition { .. }, Some(ClockBinding::EventPosition)) => {}
            (
                clock::RangeRef::FixedSample {
                    epoch_nanos,
                    period_nanos,
                    ..
                },
                Some(ClockBinding::FixedSample {
                    epoch,
                    period,
                    unit,
                }),
            ) if unit == "nanoseconds"
                && epoch.denominator() == 1
                && period.denominator() == 1
                && epoch_nanos.parse::<i64>().ok() == Some(epoch.numerator())
                && period_nanos.parse::<i64>().ok() == Some(period.numerator()) => {}
            _ => return Err(mismatch("clockBinding")),
        }
    }
    Ok(())
}

fn validate_axis_pair(
    progress: &progress::View,
    closure: &closure::View,
    clock: &clock::View,
    field: &'static str,
) -> Result<(), OwnerReadError> {
    let progress_payload = progress.payload();
    let closure_payload = closure.payload();
    if progress.subject() != closure.subject()
        || progress.authority() != closure.authority()
        || progress_payload.scope_identity() != closure_payload.scope_identity()
        || progress_payload.clock_identity() != clock.payload().clock_identity()
        || closure_payload.clock_identity() != clock.payload().clock_identity()
        || progress_payload.clock_revision() != clock.payload().clock_revision()
        || closure_payload.clock_revision() != clock.payload().clock_revision()
    {
        return Err(mismatch(field));
    }
    Ok(())
}

fn progress_axis(view: &progress::View) -> AxisReference {
    AxisReference {
        artifact: artifact_from_view(progress::CONTRACT, progress::SCHEMA_SHA256, view),
        authority_identity: view.authority().definition_identity.as_str().to_owned(),
        authority_revision: view.authority().definition_revision.as_str().to_owned(),
        authority_digest: digest_hex(view.authority().definition_digest.as_bytes()),
        scope_identity: view.subject().scope_identity.as_str().to_owned(),
        population_identity: view.subject().population_identity.as_str().to_owned(),
        state: view.payload().state(),
    }
}

fn closure_axis(view: &closure::View) -> AxisReference {
    AxisReference {
        artifact: artifact_from_view(closure::CONTRACT, closure::SCHEMA_SHA256, view),
        authority_identity: view.authority().definition_identity.as_str().to_owned(),
        authority_revision: view.authority().definition_revision.as_str().to_owned(),
        authority_digest: digest_hex(view.authority().definition_digest.as_bytes()),
        scope_identity: view.subject().scope_identity.as_str().to_owned(),
        population_identity: view.subject().population_identity.as_str().to_owned(),
        state: view.payload().state(),
    }
}

fn completeness_ref(view: &completeness::View) -> CompletenessReference {
    CompletenessReference {
        artifact: artifact_from_view(completeness::CONTRACT, completeness::SCHEMA_SHA256, view),
        population_identity: view.payload().population_identity().to_owned(),
        boundary_identity: view.payload().boundary_identity().to_owned(),
        facts: view
            .payload()
            .facts()
            .map(|fact| CompletenessFactReference {
                member_identity: fact.member_identity.to_owned(),
                observation_identity: fact.observation_identity.map(str::to_owned),
                status: fact.status,
            })
            .collect(),
        state: view.payload().state(),
    }
}

fn availability_ref(view: &availability::View) -> AvailabilityReference {
    AvailabilityReference {
        artifact: artifact_from_view(availability::CONTRACT, availability::SCHEMA_SHA256, view),
        required_results: view.payload().required_results().to_vec(),
        available_results: view.payload().available_results().to_vec(),
        state: view.payload().state(),
    }
}

trait ObservationView {
    fn document_identity(&self) -> &str;
    fn document_revision(&self) -> u64;
    fn document_bytes(&self) -> &[u8];
}

macro_rules! observation_view {
    ($type:path) => {
        impl ObservationView for $type {
            fn document_identity(&self) -> &str {
                self.identity().as_str()
            }

            fn document_revision(&self) -> u64 {
                self.revision()
            }

            fn document_bytes(&self) -> &[u8] {
                self.bytes()
            }
        }
    };
}

observation_view!(clock::View);
observation_view!(progress::View);
observation_view!(closure::View);
observation_view!(completeness::View);
observation_view!(availability::View);

fn artifact_from_view<V: ObservationView>(
    contract: &str,
    schema_sha256: &str,
    view: &V,
) -> ArtifactReference {
    ArtifactReference {
        contract: contract.to_owned(),
        schema_sha256: schema_sha256.to_owned(),
        identity: view.document_identity().to_owned(),
        revision: view.document_revision(),
        digest: raw_sha256(view.document_bytes()),
    }
}

fn validate_artifact(
    artifact: &ArtifactReference,
    usage: OwnerUsage,
) -> Result<(), OwnerReadError> {
    if artifact.contract.is_empty()
        || artifact.identity.is_empty()
        || artifact.revision == 0
        || !is_sha256(&artifact.schema_sha256)
        || !is_sha256(&artifact.digest)
    {
        return Err(mismatch_with_usage("artifactReference", usage));
    }
    Ok(())
}

fn limits_wire(limits: OwnerLimits) -> Result<LimitsWire, OwnerReadError> {
    Ok(LimitsWire {
        max_input_bytes: checked_u64(limits.max_input_bytes)?,
        max_output_bytes: checked_u64(limits.max_output_bytes)?,
        max_depth: checked_u64(limits.max_depth)?,
        max_string_bytes: checked_u64(limits.max_string_bytes)?,
        max_formula_nodes: checked_u64(limits.max_formula_nodes)?,
        max_formula_depth: checked_u64(limits.max_formula_depth)?,
        max_positions: checked_u64(limits.max_positions)?,
        max_propositions: checked_u64(limits.max_propositions)?,
        max_support: checked_u64(limits.max_support)?,
        max_history_span: limits.max_history_span,
        max_evaluation_steps: limits.max_evaluation_steps,
        max_recursion_depth: limits.max_recursion_depth,
        max_visited_fields: checked_u64(limits.max_visited_fields)?,
    })
}

fn checked_u64(value: usize) -> Result<u64, OwnerReadError> {
    u64::try_from(value).map_err(|_| resource("limitConversion"))
}

fn syntax_error(error: StrictDocumentReadError, field: &'static str) -> OwnerReadError {
    match error {
        StrictDocumentReadError::DocumentTooLarge { .. }
        | StrictDocumentReadError::DepthLimitExceeded { .. }
        | StrictDocumentReadError::StringTooLarge { .. }
        | StrictDocumentReadError::WorkLimitExceeded { .. }
        | StrictDocumentReadError::ResourceLimitExceeded { .. } => resource(field),
        StrictDocumentReadError::NonCanonicalDocument
        | StrictDocumentReadError::InvalidDocument(_)
        | _ => mismatch(field),
    }
}

fn syntax_limits(limits: OwnerLimits) -> SyntaxArtifactLimits {
    SyntaxArtifactLimits {
        document_bytes: limits.max_input_bytes,
        json_depth: limits.max_depth,
        string_bytes: limits.max_string_bytes,
        formula_nodes: limits.max_formula_nodes,
        formula_depth: limits.max_formula_depth,
        signals: limits.max_propositions,
        bindings: limits.max_propositions,
        propositions: limits.max_propositions,
        work: limits.max_visited_fields,
    }
}

fn digest_hex(bytes: &[u8; 32]) -> String {
    let mut value = String::with_capacity(64);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(value, "{byte:02x}");
    }
    value
}

fn validate_identity_text(value: &str, field: &'static str) -> Result<(), OwnerReadError> {
    if value.is_empty() || value.len() > 256 {
        Err(mismatch(field))
    } else {
        Ok(())
    }
}

fn mismatch(field: &'static str) -> OwnerReadError {
    mismatch_with_usage(field, OwnerUsage::default())
}

fn mismatch_with_usage(field: &'static str, usage: OwnerUsage) -> OwnerReadError {
    OwnerReadError::new(OwnerReadErrorCode::ExpectedMismatch, field, usage)
}

fn encoding(field: &'static str) -> OwnerReadError {
    OwnerReadError::new(OwnerReadErrorCode::Encoding, field, OwnerUsage::default())
}

fn resource(field: &'static str) -> OwnerReadError {
    OwnerReadError::new(
        OwnerReadErrorCode::ResourceIncomplete,
        field,
        OwnerUsage::default(),
    )
}
