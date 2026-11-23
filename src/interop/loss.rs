//! Typed loss records (FR-091): every construct an import or export cannot
//! carry exactly is named, never dropped, defaulted or approximated.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use serde::{Deserialize, Serialize};

use super::SourceRange;
use crate::wire::common::read as read_canonical;
use crate::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};

/// Loss record wire identity.
pub const LOSS_RECORD_V1: &str = "tl-mltl.loss-record/v1";

/// The closed set of loss kinds (10).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LossKind {
    /// An operator outside the target or source subset.
    UnsupportedOperator,
    /// A bare next whose strength the source does not state.
    AmbiguousNext,
    /// A non-Boolean signal type.
    UnsupportedSignalType,
    /// An interval the target cannot carry (open, unbounded or malformed).
    UnsupportedInterval,
    /// A variable with no map entry.
    UnmappedVariable,
    /// Two map entries for one variable or one signal.
    DuplicateVariable,
    /// A header field outside the recognized set.
    UnknownHeaderField,
    /// Carrying it would close an unbounded formula at a mission time (ADR-045).
    MissionClosureRequired,
    /// Foreign content that is not the format's formula language.
    ForeignConstruct,
    /// A reader ceiling was reached for this requirement.
    ResourceLimit,
}

impl LossKind {
    /// Every kind, in declaration order.
    pub const ALL: [Self; 10] = [
        Self::UnsupportedOperator,
        Self::AmbiguousNext,
        Self::UnsupportedSignalType,
        Self::UnsupportedInterval,
        Self::UnmappedVariable,
        Self::DuplicateVariable,
        Self::UnknownHeaderField,
        Self::MissionClosureRequired,
        Self::ForeignConstruct,
        Self::ResourceLimit,
    ];
}

/// The foreign format a record concerns.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ForeignFormat {
    /// C2PO specification text.
    C2po,
    /// FRET-exported MLTL text.
    FretMltl,
    /// FRET export bundle (variable map and trace header).
    FretBundle,
    /// WEST formula line.
    West,
}

/// `tl-mltl.loss-record/v1`. A loss record is never a temporal verdict.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LossRecord {
    /// Wire identity.
    pub schema_version: String,
    /// Kind.
    pub kind: LossKind,
    /// Format.
    pub format: ForeignFormat,
    /// Construct, as spelled in the source or named by the exporter.
    pub construct: String,
    /// Source span, when the construct has one.
    pub span: Option<SourceRange>,
    /// Why there is no span (required exactly when `span` is `None`).
    pub no_span_reason: Option<String>,
    /// Zero-based requirement index in a multi-requirement source.
    pub requirement: Option<usize>,
}

impl LossRecord {
    /// A record with a span.
    pub fn at(
        kind: LossKind,
        format: ForeignFormat,
        construct: impl Into<String>,
        span: SourceRange,
        requirement: Option<usize>,
    ) -> Self {
        Self {
            schema_version: LOSS_RECORD_V1.to_owned(),
            kind,
            format,
            construct: construct.into(),
            span: Some(span),
            no_span_reason: None,
            requirement,
        }
    }

    /// A record whose construct has no source span, with the stated reason.
    pub fn without_span(
        kind: LossKind,
        format: ForeignFormat,
        construct: impl Into<String>,
        reason: impl Into<String>,
        requirement: Option<usize>,
    ) -> Self {
        Self {
            schema_version: LOSS_RECORD_V1.to_owned(),
            kind,
            format,
            construct: construct.into(),
            span: None,
            no_span_reason: Some(reason.into()),
            requirement,
        }
    }

    /// Canonical JSON bytes.
    pub fn to_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("loss record serializes")
    }

    /// Strict reader through the owner boundary (size, depth, canonical form),
    /// plus identity and span/reason exclusivity.
    pub fn from_json_bytes(bytes: &[u8], limits: OwnerLimits) -> Result<Self, OwnerReadError> {
        let (record, _) = read_canonical(bytes, limits, |record: &Self, _| {
            let field = if record.schema_version != LOSS_RECORD_V1 {
                Some("schemaVersion")
            } else if record.span.is_some() == record.no_span_reason.is_some() {
                Some("span")
            } else {
                None
            };
            match field {
                Some(field) => Err(OwnerReadError::new(
                    OwnerReadErrorCode::InvalidCombination,
                    field,
                    OwnerUsage::default(),
                )),
                None => Ok(OwnerUsage::default()),
            }
        })?;
        Ok(record)
    }
}
