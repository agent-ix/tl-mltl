//! Strict owner boundary for `tl-mltl.history-requirement/v1`.

use crate::wire::common::{produce, read_expected};
use crate::wire::{OwnerDocument, OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};

pub use super::{
    analyze_required_history, HistoryRequirementError, HistoryRequirementReport,
    HistoryRequirementSchemaVersion, HistoryRequirementValidationError,
};

/// Exact checked-in JSON Schema bytes for [`super::HISTORY_REQUIREMENT_V1`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../../schemas/history-requirement-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "4da9ad685f502369e369ecac38c0d23bb1e094c4c8982a70774ae9ae0c356bf9";

/// Constructor-private validated required-history report.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedHistoryRequirement {
    report: HistoryRequirementReport,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl ValidatedHistoryRequirement {
    /// Exact validated report.
    #[must_use]
    pub const fn report(&self) -> &HistoryRequirementReport {
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

/// Produces canonical required-history bytes.
pub fn derive(
    report: &HistoryRequirementReport,
    limits: OwnerLimits,
) -> Result<OwnerDocument<HistoryRequirementReport>, OwnerReadError> {
    produce(
        report.clone(),
        validate(report, limits.effective())?,
        limits,
    )
}

/// Strict-reads required-history bytes against an independently supplied report.
pub fn read(
    bytes: &[u8],
    expected: &HistoryRequirementReport,
    limits: OwnerLimits,
) -> Result<ValidatedHistoryRequirement, OwnerReadError> {
    let (report, usage) = read_expected(bytes, expected, limits, validate)?;
    Ok(ValidatedHistoryRequirement {
        report,
        bytes: bytes.to_vec(),
        usage,
    })
}

fn validate(
    report: &HistoryRequirementReport,
    limits: OwnerLimits,
) -> Result<OwnerUsage, OwnerReadError> {
    report.validate().map_err(|_| invalid("requirement"))?;
    let usage = OwnerUsage {
        history_span: report.required_positions,
        ..OwnerUsage::default()
    };
    if usage.history_span > limits.max_history_span {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            "historySpan",
            usage,
        ));
    }
    Ok(usage)
}

fn invalid(field: &'static str) -> OwnerReadError {
    OwnerReadError::new(
        OwnerReadErrorCode::InvalidCombination,
        field,
        OwnerUsage::default(),
    )
}
