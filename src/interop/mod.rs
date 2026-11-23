//! Foreign-format readers and writers (ADR-046): C2PO import (FR-086), FRET
//! MLTL and bundle import (FR-087, FR-088), WEST import and export (FR-092),
//! typed loss records (FR-091), attribution (FR-089) and the mission report and
//! closure refusal (FR-090, FR-094).
//!
//! Every reader has its own small bounded lexer and applies the same ceilings
//! (NFR-047) before retaining anything: source size, nesting and token count.
//! No reader executes R2U2, C2PO, FRET or WEST.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;

use serde::{Deserialize, Serialize};

pub mod attribution;
pub mod c2po;
pub mod fret;
pub mod loss;
pub mod mission;

pub use attribution::{Attribution, AttributionError, REVISION_UNKNOWN};
pub use loss::{ForeignFormat, LossKind, LossRecord, LOSS_RECORD_V1};

/// Byte range `[start, end)` in the source text.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SourceRange {
    /// First byte.
    pub start: usize,
    /// One past the last byte.
    pub end: usize,
}

/// Ceilings shared by every foreign reader (NFR-047).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ReaderLimits {
    /// Maximum source bytes; the exact value is admitted.
    pub max_source_bytes: usize,
    /// Maximum expression nesting depth; the exact value is admitted.
    pub max_nesting: u32,
    /// Maximum lexical tokens.
    pub max_tokens: usize,
}

impl Default for ReaderLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 1 << 20,
            max_nesting: 128,
            max_tokens: 1 << 18,
        }
    }
}

/// Hostile-input refusals shared by the readers (NFR-047's eight classes map
/// onto these; "exact" limit cases are admitted).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReaderError {
    /// The source holds no requirement at all.
    Empty,
    /// The source is not UTF-8.
    NonUtf8 {
        /// Byte offset of the first invalid sequence.
        offset: usize,
    },
    /// The source contains a NUL byte.
    NulByte {
        /// Byte offset.
        offset: usize,
    },
    /// The source exceeds `max_source_bytes`.
    SourceTooLarge {
        /// Source length.
        length: usize,
        /// Limit.
        limit: usize,
    },
    /// Nesting exceeds `max_nesting`.
    NestingTooDeep {
        /// Limit.
        limit: u32,
        /// Where the limit was passed.
        span: SourceRange,
    },
    /// Token count exceeds `max_tokens`.
    TooManyTokens {
        /// Limit.
        limit: usize,
    },
    /// A signal or variable is declared twice.
    DuplicateSignal {
        /// The name.
        name: String,
        /// The second declaration.
        span: SourceRange,
    },
}

impl fmt::Display for ReaderError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(formatter, "empty source"),
            Self::NonUtf8 { offset } => write!(formatter, "invalid UTF-8 at byte {offset}"),
            Self::NulByte { offset } => write!(formatter, "NUL byte at {offset}"),
            Self::SourceTooLarge { length, limit } => {
                write!(formatter, "source of {length} bytes exceeds {limit}")
            }
            Self::NestingTooDeep { limit, span } => {
                write!(formatter, "nesting deeper than {limit} at byte {}", span.start)
            }
            Self::TooManyTokens { limit } => write!(formatter, "more than {limit} tokens"),
            Self::DuplicateSignal { name, span } => {
                write!(formatter, "signal {name:?} declared twice at byte {}", span.start)
            }
        }
    }
}

impl std::error::Error for ReaderError {}

/// Admits raw bytes as bounded UTF-8 text without NUL bytes.
pub(crate) fn admit_source(bytes: &[u8], limits: ReaderLimits) -> Result<&str, ReaderError> {
    if bytes.len() > limits.max_source_bytes {
        return Err(ReaderError::SourceTooLarge {
            length: bytes.len(),
            limit: limits.max_source_bytes,
        });
    }
    let text = core::str::from_utf8(bytes).map_err(|error| ReaderError::NonUtf8 {
        offset: error.valid_up_to(),
    })?;
    if let Some(offset) = bytes.iter().position(|byte| *byte == 0) {
        return Err(ReaderError::NulByte { offset });
    }
    if text.trim().is_empty() {
        return Err(ReaderError::Empty);
    }
    Ok(text)
}
