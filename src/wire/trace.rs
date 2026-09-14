//! Strict owner boundary for `tl-mltl.trace/v1`.

use super::common::{produce, read as read_canonical, OwnerDocument};
use super::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage, TraceDocument};

/// Immutable trace contract label.
pub const CONTRACT: &str = "tl-mltl.trace/v1";
/// Exact checked-in JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../../schemas/trace-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "9c1020bb56cbd38ebc10405a39ccffc156908bb00f032de4c518626343989946";

/// Constructor-private trace accepted through the bounded canonical reader.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedTrace {
    document: TraceDocument,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl ValidatedTrace {
    /// Produces and re-admits one existing trace document through the owner boundary.
    pub fn admit(document: &TraceDocument, limits: OwnerLimits) -> Result<Self, OwnerReadError> {
        let owner = produce(
            document.clone(),
            validate(document, limits.effective())?,
            limits,
        )?;
        let bytes = owner.bytes().to_vec();
        Self::from_json_bytes(&bytes, limits)
    }

    /// Strict-reads exactly one bounded canonical trace.
    pub fn from_json_bytes(bytes: &[u8], limits: OwnerLimits) -> Result<Self, OwnerReadError> {
        let (document, usage) = read_canonical(bytes, limits, validate)?;
        Ok(Self {
            document,
            bytes: bytes.to_vec(),
            usage,
        })
    }

    /// Validated trace value.
    #[must_use]
    pub const fn document(&self) -> &TraceDocument {
        &self.document
    }

    /// Exact canonical owner bytes.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Bounded read work.
    #[must_use]
    pub const fn usage(&self) -> OwnerUsage {
        self.usage
    }

    /// Re-emits byte-identical canonical owner bytes.
    pub fn canonical_json_bytes(&self) -> Vec<u8> {
        self.bytes.clone()
    }
}

/// Produces canonical trace bytes after complete owner validation.
pub fn derive(
    document: &TraceDocument,
    limits: OwnerLimits,
) -> Result<OwnerDocument<TraceDocument>, OwnerReadError> {
    produce(
        document.clone(),
        validate(document, limits.effective())?,
        limits,
    )
}

/// Strict-reads canonical trace bytes against an independently supplied trace.
pub fn read(
    bytes: &[u8],
    expected: &TraceDocument,
    limits: OwnerLimits,
) -> Result<ValidatedTrace, OwnerReadError> {
    let validated = ValidatedTrace::from_json_bytes(bytes, limits)?;
    if validated.document() != expected {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ExpectedMismatch,
            "trace",
            validated.usage(),
        ));
    }
    Ok(validated)
}

fn validate(document: &TraceDocument, limits: OwnerLimits) -> Result<OwnerUsage, OwnerReadError> {
    if document.trace_id.is_empty() || document.trace_id.len() > limits.max_string_bytes.min(256) {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::InvalidCombination,
            "traceId",
            OwnerUsage::default(),
        ));
    }
    let proposition_count = document
        .instants
        .iter()
        .try_fold(0usize, |total, row| total.checked_add(row.len()))
        .ok_or_else(|| {
            OwnerReadError::new(
                OwnerReadErrorCode::ResourceIncomplete,
                "propositions",
                OwnerUsage::default(),
            )
        })?;
    let usage = OwnerUsage {
        positions: document.instants.len(),
        propositions: proposition_count,
        ..OwnerUsage::default()
    };
    if usage.positions > limits.max_positions {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            "positions",
            usage,
        ));
    }
    if usage.propositions > limits.max_propositions {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            "propositions",
            usage,
        ));
    }
    for row in &document.instants {
        if row.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(OwnerReadError::new(
                OwnerReadErrorCode::InvalidCombination,
                "instants",
                usage,
            ));
        }
    }
    Ok(usage)
}
