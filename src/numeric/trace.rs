//! Strict owner boundary for `tl-mltl.numeric-trace/v1` (FR-053).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{PropositionId, SignalCatalogDocument, SignalDomain, SignalId};

use crate::wire::common::read as read_canonical;
use crate::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};

/// Immutable numeric trace contract label. No digest constant exists for it
/// (tl-syntax ADR-019 register).
pub const NUMERIC_TRACE_V1: &str = "tl-mltl.numeric-trace/v1";
/// Exact checked-in JSON Schema bytes for [`NUMERIC_TRACE_V1`].
pub const NUMERIC_TRACE_V1_SCHEMA_BYTES: &[u8] =
    include_bytes!("../../schemas/numeric-trace-v1.schema.json");
/// Default and maximum number of numeric observations per document.
pub const MAX_NUMERIC_VALUES: usize = 16_777_216;

/// Numeric trace wire identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NumericTraceSchemaVersion {
    /// Initial numeric trace schema.
    #[serde(rename = "tl-mltl.numeric-trace/v1")]
    V1,
}

/// Exact fixed-decimal observation: `coefficient * 10^-scale`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DecimalValue {
    /// Signed coefficient.
    pub coefficient: i64,
    /// Base-ten fractional digits.
    pub scale: u8,
}

/// One numeric observation. Exactly one of `integer` and `decimal` is present.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NumericObservation {
    /// Catalog signal identity.
    pub signal: u32,
    /// Integer value, for an Integer signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub integer: Option<i64>,
    /// Fixed-decimal value, for a FixedDecimal signal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decimal: Option<DecimalValue>,
}

impl NumericObservation {
    /// Integer observation.
    pub const fn integer(signal: u32, value: i64) -> Self {
        Self {
            signal,
            integer: Some(value),
            decimal: None,
        }
    }

    /// Fixed-decimal observation.
    pub const fn decimal(signal: u32, coefficient: i64, scale: u8) -> Self {
        Self {
            signal,
            integer: None,
            decimal: Some(DecimalValue { coefficient, scale }),
        }
    }

    /// Exact `(coefficient, scale)`; integers are scale 0.
    pub fn exact(self) -> Option<(i64, u8)> {
        match (self.integer, self.decimal) {
            (Some(value), None) => Some((value, 0)),
            (None, Some(decimal)) => Some((decimal.coefficient, decimal.scale)),
            _ => None,
        }
    }
}

/// One instant: Boolean true-set plus numeric observations.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NumericInstant {
    /// Strictly increasing true Boolean-bound propositions.
    pub propositions: Vec<PropositionId>,
    /// Observations strictly increasing by `signal`; an absent signal is unobserved.
    // SPEC-GAP: FR-053 says an instant "without values" is legal; the member is
    // kept required (possibly empty) so each instant has one canonical form.
    pub values: Vec<NumericObservation>,
}

/// `tl-mltl.numeric-trace/v1` document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NumericTraceDocument {
    /// Wire identity.
    pub schema_version: NumericTraceSchemaVersion,
    /// Stable trace identity.
    pub trace_id: String,
    /// Whether the trace is complete.
    pub closed: bool,
    /// Ordered instants.
    pub instants: Vec<NumericInstant>,
}

/// Owner limits plus the numeric value ceiling.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumericTraceLimits {
    /// Shared owner limits (positions, propositions, bytes, depth).
    pub owner: OwnerLimits,
    /// Maximum numeric observations per document, clamped to [`MAX_NUMERIC_VALUES`].
    pub max_values: usize,
}

impl Default for NumericTraceLimits {
    fn default() -> Self {
        Self {
            owner: OwnerLimits::default(),
            max_values: MAX_NUMERIC_VALUES,
        }
    }
}

/// Catalog admission refusal classes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumericAdmissionCode {
    /// The signal is not declared.
    UnknownSignal,
    /// The signal is Boolean.
    NonNumericSignal,
    /// Integer value for a FixedDecimal signal or the converse, or neither/both members.
    KindMismatch,
    /// Decimal scale differs from the declared scale.
    ScaleMismatch,
    /// Value outside the declared bounds.
    ValueOutOfDomain,
    /// `values` not strictly increasing by signal (includes a repeated signal).
    ValuesNotStrictlyOrdered,
    /// `propositions` not strictly increasing.
    PropositionsNotStrictlyOrdered,
}

impl NumericAdmissionCode {
    /// Stable wire label.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnknownSignal => "TL-NUMERIC-UNKNOWN-SIGNAL",
            Self::NonNumericSignal => "TL-NUMERIC-NON-NUMERIC-SIGNAL",
            Self::KindMismatch => "TL-NUMERIC-KIND-MISMATCH",
            Self::ScaleMismatch => "TL-NUMERIC-SCALE-MISMATCH",
            Self::ValueOutOfDomain => "TL-NUMERIC-VALUE-OUT-OF-DOMAIN",
            Self::ValuesNotStrictlyOrdered => "TL-NUMERIC-VALUES-NOT-ORDERED",
            Self::PropositionsNotStrictlyOrdered => "TL-NUMERIC-PROPOSITIONS-NOT-ORDERED",
        }
    }
}

/// Typed refusal; never a partial document.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NumericTraceError {
    /// Wire-level owner refusal (encoding, JSON, canonical form, limits).
    Owner(OwnerReadError),
    /// Catalog admission refusal.
    Admission {
        /// Refusal class.
        code: NumericAdmissionCode,
        /// Instant index.
        instant: usize,
        /// Signal identity, when the refusal concerns one.
        signal: Option<u32>,
    },
}

impl From<OwnerReadError> for NumericTraceError {
    fn from(error: OwnerReadError) -> Self {
        Self::Owner(error)
    }
}

impl fmt::Display for NumericTraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Owner(error) => error.fmt(formatter),
            Self::Admission {
                code,
                instant,
                signal,
            } => write!(
                formatter,
                "{} at instant {instant} signal {signal:?}",
                code.as_str()
            ),
        }
    }
}

impl std::error::Error for NumericTraceError {}

/// A numeric trace admitted against one catalog.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedNumericTrace {
    document: NumericTraceDocument,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl ValidatedNumericTrace {
    /// Strict-reads exactly one bounded canonical numeric trace and admits every
    /// observation against `catalog`.
    ///
    /// Implements: FR-053
    pub fn from_json_bytes(
        bytes: &[u8],
        catalog: &SignalCatalogDocument,
        limits: NumericTraceLimits,
    ) -> Result<Self, NumericTraceError> {
        let (document, usage) = read_canonical(bytes, limits.owner, |document, owner| {
            validate(document, catalog, owner, limits.max_values)
        })?;
        Ok(Self {
            document,
            bytes: bytes.to_vec(),
            usage,
        })
    }

    /// Admits an in-memory document by producing and re-reading its canonical bytes.
    pub fn admit(
        document: &NumericTraceDocument,
        catalog: &SignalCatalogDocument,
        limits: NumericTraceLimits,
    ) -> Result<Self, NumericTraceError> {
        let bytes = serde_json::to_vec(document).map_err(|_| {
            NumericTraceError::Owner(OwnerReadError::new(
                OwnerReadErrorCode::Encoding,
                "document",
                OwnerUsage::default(),
            ))
        })?;
        Self::from_json_bytes(&bytes, catalog, limits)
    }

    /// Validated document.
    #[must_use]
    pub const fn document(&self) -> &NumericTraceDocument {
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

fn admission(code: NumericAdmissionCode, instant: usize, signal: Option<u32>) -> NumericTraceError {
    NumericTraceError::Admission {
        code,
        instant,
        signal,
    }
}

fn validate(
    document: &NumericTraceDocument,
    catalog_document: &SignalCatalogDocument,
    limits: OwnerLimits,
    max_values: usize,
) -> Result<OwnerUsage, NumericTraceError> {
    if document.trace_id.is_empty() || document.trace_id.len() > limits.max_string_bytes.min(256) {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::InvalidCombination,
            "traceId",
            OwnerUsage::default(),
        )
        .into());
    }
    let mut propositions = 0usize;
    let mut values = 0usize;
    for instant in &document.instants {
        propositions = propositions.saturating_add(instant.propositions.len());
        values = values.saturating_add(instant.values.len());
    }
    let usage = OwnerUsage {
        positions: document.instants.len(),
        propositions,
        ..OwnerUsage::default()
    };
    if usage.positions > limits.max_positions {
        return Err(
            OwnerReadError::new(OwnerReadErrorCode::ResourceIncomplete, "positions", usage).into(),
        );
    }
    if usage.propositions > limits.max_propositions {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            "propositions",
            usage,
        )
        .into());
    }
    if values > max_values.min(MAX_NUMERIC_VALUES) {
        return Err(
            OwnerReadError::new(OwnerReadErrorCode::ResourceIncomplete, "values", usage).into(),
        );
    }
    let catalog = catalog_document
        .validate()
        .expect("a constructed catalog document validates");
    for (index, instant) in document.instants.iter().enumerate() {
        if instant.propositions.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(admission(
                NumericAdmissionCode::PropositionsNotStrictlyOrdered,
                index,
                None,
            ));
        }
        if let Some(pair) = instant
            .values
            .windows(2)
            .find(|pair| pair[0].signal >= pair[1].signal)
        {
            return Err(admission(
                NumericAdmissionCode::ValuesNotStrictlyOrdered,
                index,
                Some(pair[1].signal),
            ));
        }
        for observation in &instant.values {
            admit_observation(&catalog, index, *observation)?;
        }
    }
    Ok(usage)
}

fn admit_observation(
    catalog: &tl_syntax::SignalCatalog<'_>,
    instant: usize,
    observation: NumericObservation,
) -> Result<(), NumericTraceError> {
    let signal = Some(observation.signal);
    let Some(declaration) = catalog.signal(SignalId(observation.signal)) else {
        return Err(admission(NumericAdmissionCode::UnknownSignal, instant, signal));
    };
    match (declaration.domain(), observation.integer, observation.decimal) {
        (SignalDomain::Boolean, _, _) => Err(admission(
            NumericAdmissionCode::NonNumericSignal,
            instant,
            signal,
        )),
        (SignalDomain::Integer(domain), Some(value), None) => {
            if value < domain.minimum() || value > domain.maximum() {
                Err(admission(NumericAdmissionCode::ValueOutOfDomain, instant, signal))
            } else {
                Ok(())
            }
        }
        (SignalDomain::FixedDecimal(domain), None, Some(decimal)) => {
            if decimal.scale != domain.scale() {
                Err(admission(NumericAdmissionCode::ScaleMismatch, instant, signal))
            } else if decimal.coefficient < domain.minimum_coefficient()
                || decimal.coefficient > domain.maximum_coefficient()
            {
                Err(admission(NumericAdmissionCode::ValueOutOfDomain, instant, signal))
            } else {
                Ok(())
            }
        }
        _ => Err(admission(NumericAdmissionCode::KindMismatch, instant, signal)),
    }
}
