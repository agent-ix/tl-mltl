//! Strict owner boundary for `tl-mltl.position-history/v1`.

use crate::wire::common::{produce, read_expected};
use crate::wire::{OwnerDocument, OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};

use super::PositionHistoryDocument;

/// Exact checked-in JSON Schema bytes for [`super::POSITION_HISTORY_V1`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../../schemas/position-history-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "55cc048b6196436af5f85cf0f29539297ad68f246f81a5ea76fba3512ca61511";

/// Constructor-private validated history.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedPositionHistory {
    document: PositionHistoryDocument,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl ValidatedPositionHistory {
    /// Exact validated history.
    #[must_use]
    pub const fn document(&self) -> &PositionHistoryDocument {
        &self.document
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

/// Produces canonical bytes for an already valid history.
pub fn derive(
    history: &PositionHistoryDocument,
    limits: OwnerLimits,
) -> Result<OwnerDocument<PositionHistoryDocument>, OwnerReadError> {
    produce(
        history.clone(),
        validate(history, limits.effective())?,
        limits,
    )
}

/// Strict-reads a history against independently supplied exact owner state.
pub fn read(
    bytes: &[u8],
    expected: &PositionHistoryDocument,
    limits: OwnerLimits,
) -> Result<ValidatedPositionHistory, OwnerReadError> {
    let (document, usage) = read_expected(bytes, expected, limits, validate)?;
    Ok(ValidatedPositionHistory {
        document,
        bytes: bytes.to_vec(),
        usage,
    })
}

fn validate(
    history: &PositionHistoryDocument,
    limits: OwnerLimits,
) -> Result<OwnerUsage, OwnerReadError> {
    history.validate().map_err(|_| {
        OwnerReadError::new(
            OwnerReadErrorCode::InvalidCombination,
            "history",
            OwnerUsage::default(),
        )
    })?;
    let propositions = history
        .observations()
        .iter()
        .try_fold(0usize, |sum, row| {
            sum.checked_add(row.true_propositions.len())
        })
        .ok_or_else(|| resource("propositions", OwnerUsage::default()))?;
    let usage = OwnerUsage {
        positions: history.observations().len(),
        propositions,
        history_span: history.through_position(),
        ..OwnerUsage::default()
    };
    if usage.positions > limits.max_positions {
        return Err(resource("positions", usage));
    }
    if usage.propositions > limits.max_propositions {
        return Err(resource("propositions", usage));
    }
    if usage.history_span > limits.max_history_span {
        return Err(resource("historySpan", usage));
    }
    Ok(usage)
}

fn resource(field: &'static str, usage: OwnerUsage) -> OwnerReadError {
    OwnerReadError::new(OwnerReadErrorCode::ResourceIncomplete, field, usage)
}
