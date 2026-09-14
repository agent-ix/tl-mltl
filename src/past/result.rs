//! Strict owner boundary for the delivered `tl-mltl.past-evaluation/v1` result.

use crate::wire::common::{produce, read_expected};
use crate::wire::{OwnerDocument, OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};

pub use super::{
    PastEvaluationReport, PastEvaluationSchemaVersion, PastResultFinality, PastResultReference,
    PastResultRelation, PastResultRelationKind, PastResultValidationError,
    PositionHistoryReference,
};

/// Exact checked-in JSON Schema bytes for [`super::PAST_EVALUATION_V1`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../../schemas/past-evaluation-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "a269301cf8150d57bb9f79e5784fbe29461ef37eb5eaeb3e571543eff2e2fa73";

/// Constructor-private validated past result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPastResult {
    report: PastEvaluationReport,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl ValidatedPastResult {
    /// Exact validated report.
    #[must_use]
    pub const fn report(&self) -> &PastEvaluationReport {
        &self.report
    }

    /// Exact canonical bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bounded read work.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }
}

/// Produces canonical past-result bytes.
pub fn derive(
    report: &PastEvaluationReport,
    limits: OwnerLimits,
) -> Result<OwnerDocument<PastEvaluationReport>, OwnerReadError> {
    produce(
        report.clone(),
        validate(report, limits.effective())?,
        limits,
    )
}

/// Strict-reads a past result against its exact independently supplied report.
pub fn read(
    bytes: &[u8],
    expected: &PastEvaluationReport,
    limits: OwnerLimits,
) -> Result<ValidatedPastResult, OwnerReadError> {
    let (report, usage) = read_expected(bytes, expected, limits, validate)?;
    Ok(ValidatedPastResult {
        report,
        bytes: bytes.to_vec(),
        usage,
    })
}

fn validate(
    report: &PastEvaluationReport,
    limits: OwnerLimits,
) -> Result<OwnerUsage, OwnerReadError> {
    report.validate().map_err(|_| {
        OwnerReadError::new(
            OwnerReadErrorCode::InvalidCombination,
            "pastResult",
            OwnerUsage::default(),
        )
    })?;
    let usage = OwnerUsage {
        positions: report.stats.input_positions,
        history_span: report.required_history,
        evaluation_steps: report.stats.steps,
        recursion_depth: report.stats.max_recursion_depth,
        ..OwnerUsage::default()
    };
    if usage.positions > limits.max_positions
        || usage.history_span > limits.max_history_span
        || usage.evaluation_steps > limits.max_evaluation_steps
        || usage.recursion_depth > limits.max_recursion_depth
    {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            "pastResultLimits",
            usage,
        ));
    }
    Ok(usage)
}
