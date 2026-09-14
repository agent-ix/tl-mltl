//! Owner-selected normalized Contract-IR result handoff.
//!
//! This module deliberately owns no Contract-IR vocabulary, parser, evaluator,
//! Boolean coercion, callback, or trust flag. It derives a closed TL-owned map
//! only from a constructor-private [`ValidatedTemporalResult`].

use quire_observation::authority::{availability, completeness};
use serde::{Deserialize, Serialize};

use crate::wire::common::{identity, is_sha256, produce, raw_sha256, read_expected};
use crate::wire::report::{
    AssessmentExecution, ResultWire, TemporalTruth, ValidatedTemporalResult,
};
use crate::wire::request::ArtifactReference;
use crate::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};

/// Immutable selected mapping contract label.
pub const CONTRACT: &str = "tl-mltl.contract-ir-result-map/v1";
/// Exact checked-in JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/contract-ir-result-map-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "6a91d001c249c0fd22dc6b57f7a1ea4a2f96e10aec1522d9417978e7c1864428";

/// Closed typed reason why the TL result does not project to a Boolean value.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum NonValueKind {
    /// A valid open-prefix evaluation remains pending.
    Pending,
    /// A selected producer/result is unavailable.
    Unavailable,
    /// The exact required fact population is incomplete.
    Incomplete,
    /// The selected profile or clock is unsupported.
    Unsupported,
    /// The evaluator failed without a truth value.
    Failed,
    /// The evaluator refused the runtime combination.
    Refused,
    /// Owner completeness reports a contradiction.
    Contradicted,
    /// Evaluation exhausted an explicit resource budget.
    ResourceIncomplete,
}

/// Value-or-non-value projection owned by tl-mltl.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub enum MappedOutcome {
    /// Valid completed final Boolean.
    Value {
        /// Exact Boolean result.
        value: bool,
    },
    /// Lossless typed non-value.
    NonValue {
        /// Exact closed non-value reason.
        reason: NonValueKind,
    },
}

/// Exact selected mapping target. Construction binds it to a validated result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingSelection {
    source_identity: String,
    source_digest: String,
}

impl MappingSelection {
    /// Selects the exact supplied result; callers cannot manufacture fields.
    #[must_use]
    pub fn for_result(result: &ValidatedTemporalResult) -> Self {
        Self {
            source_identity: result.identity().to_owned(),
            source_digest: raw_sha256(result.bytes()),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct MappingWire {
    contract_version: String,
    identity: String,
    source_result: ArtifactReference,
    outcome: MappedOutcome,
    source: ResultWire,
}

/// Immutable canonical mapping bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingDocument {
    wire: MappingWire,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl MappingDocument {
    /// Mapping identity, distinct from its source result identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.wire.identity
    }

    /// Exact canonical mapping bytes.
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

/// Constructor-private mapped result view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedMappedResult {
    wire: MappingWire,
    bytes: Vec<u8>,
    usage: OwnerUsage,
    source: ValidatedTemporalResult,
}

impl ValidatedMappedResult {
    /// Mapping identity.
    #[must_use]
    pub fn identity(&self) -> &str {
        &self.wire.identity
    }

    /// Exact selected source result identity.
    #[must_use]
    pub fn source_result_identity(&self) -> &str {
        self.wire.source_result.identity()
    }

    /// Derived value or typed non-value.
    #[must_use]
    pub const fn outcome(&self) -> MappedOutcome {
        self.wire.outcome
    }

    /// Exact constructor-private temporal result from which this view was derived.
    #[must_use]
    pub const fn source(&self) -> &ValidatedTemporalResult {
        &self.source
    }

    /// Exact canonical map bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bounded strict-read work.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }
}

/// Derives a selected normalized view only from a validated temporal result.
pub fn map(
    result: &ValidatedTemporalResult,
    selection: &MappingSelection,
    limits: OwnerLimits,
) -> Result<MappingDocument, OwnerReadError> {
    let (wire, usage) = build_wire(result, selection)?;
    let document = produce(wire, usage, limits)?;
    let (wire, bytes, usage) = document.into_parts();
    Ok(MappingDocument { wire, bytes, usage })
}

/// Strict-reads a mapping by re-deriving it from the exact validated result.
pub fn read(
    bytes: &[u8],
    result: &ValidatedTemporalResult,
    selection: &MappingSelection,
    limits: OwnerLimits,
) -> Result<ValidatedMappedResult, OwnerReadError> {
    let (expected, expected_usage) = build_wire(result, selection)?;
    let (wire, usage) = read_expected(bytes, &expected, limits, |wire, _| {
        validate_wire(wire, expected_usage)
    })?;
    Ok(ValidatedMappedResult {
        wire,
        bytes: bytes.to_vec(),
        usage,
        source: result.clone(),
    })
}

fn build_wire(
    result: &ValidatedTemporalResult,
    selection: &MappingSelection,
) -> Result<(MappingWire, OwnerUsage), OwnerReadError> {
    if selection.source_identity != result.identity()
        || selection.source_digest != raw_sha256(result.bytes())
    {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ExpectedMismatch,
            "mappingSelection",
            result.usage(),
        ));
    }
    let outcome = outcome(result);
    let usage = OwnerUsage {
        formula_nodes: result.usage().formula_nodes,
        formula_depth: result.usage().formula_depth,
        positions: result.usage().positions,
        propositions: result.usage().propositions,
        support: result.usage().support,
        history_span: result.usage().history_span,
        evaluation_steps: result.usage().evaluation_steps,
        recursion_depth: result.usage().recursion_depth,
        ..OwnerUsage::default()
    };
    let mut wire = MappingWire {
        contract_version: CONTRACT.to_owned(),
        identity: String::new(),
        source_result: ArtifactReference {
            contract: crate::wire::report::CONTRACT.to_owned(),
            schema_sha256: crate::wire::report::SCHEMA_SHA256.to_owned(),
            identity: result.identity().to_owned(),
            revision: result.revision(),
            digest: raw_sha256(result.bytes()),
        },
        outcome,
        source: result.wire().clone(),
    };
    wire.identity = identity(CONTRACT, &wire, "identity")?;
    validate_wire(&wire, usage)?;
    Ok((wire, usage))
}

fn outcome(result: &ValidatedTemporalResult) -> MappedOutcome {
    let non_value = match result.execution() {
        AssessmentExecution::ResourceIncomplete => Some(NonValueKind::ResourceIncomplete),
        AssessmentExecution::Unsupported => Some(NonValueKind::Unsupported),
        AssessmentExecution::Failed => Some(NonValueKind::Failed),
        AssessmentExecution::Refused => Some(NonValueKind::Refused),
        AssessmentExecution::Completed => match result.completeness().state() {
            completeness::State::Contradicted => Some(NonValueKind::Contradicted),
            completeness::State::Incomplete => Some(NonValueKind::Incomplete),
            completeness::State::Complete => match result.availability().state() {
                availability::State::Available => None,
                availability::State::NotYetObserved
                | availability::State::ProducerUnavailable
                | availability::State::ContractUnavailable => Some(NonValueKind::Unavailable),
            },
        },
    };
    if let Some(reason) = non_value {
        return MappedOutcome::NonValue { reason };
    }
    if !result.is_final() {
        return MappedOutcome::NonValue {
            reason: match result.truth() {
                TemporalTruth::Pending => NonValueKind::Pending,
                _ => NonValueKind::Unavailable,
            },
        };
    }
    match result.truth() {
        TemporalTruth::Satisfied => MappedOutcome::Value { value: true },
        TemporalTruth::Violated => MappedOutcome::Value { value: false },
        TemporalTruth::Pending => MappedOutcome::NonValue {
            reason: NonValueKind::Pending,
        },
        TemporalTruth::Unavailable => MappedOutcome::NonValue {
            reason: NonValueKind::Unavailable,
        },
    }
}

fn validate_wire(wire: &MappingWire, usage: OwnerUsage) -> Result<OwnerUsage, OwnerReadError> {
    if wire.contract_version != CONTRACT {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ContractMismatch,
            "contractVersion",
            usage,
        ));
    }
    if !is_sha256(&wire.identity) || identity(CONTRACT, wire, "identity")? != wire.identity {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::IdentityMismatch,
            "identity",
            usage,
        ));
    }
    Ok(usage)
}
