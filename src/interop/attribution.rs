//! Upstream attribution of imported or retained foreign artifacts (FR-089).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use serde::{Deserialize, Serialize};

/// The explicit marker for a revision nobody recorded. It is a value, never an
/// absent field, and no revision is ever synthesized in its place.
pub const REVISION_UNKNOWN: &str = "UNKNOWN";

/// Attribution as supplied (fields may be absent so a refusal can name them).
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Attribution {
    /// Upstream repository URL.
    pub repository: Option<String>,
    /// Path inside the upstream repository.
    pub path: Option<String>,
    /// Upstream revision, or [`REVISION_UNKNOWN`].
    pub revision: Option<String>,
    /// SPDX licence identifier.
    pub licence: Option<String>,
}

/// A missing attribution field.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AttributionError {
    /// No repository.
    MissingRepository,
    /// No path.
    MissingPath,
    /// No revision (for WEST: not even the `UNKNOWN` marker).
    MissingRevision,
    /// No licence.
    MissingLicence,
}

fn present(field: &Option<String>) -> bool {
    field.as_deref().is_some_and(|value| !value.trim().is_empty())
}

impl Attribution {
    fn common(&self) -> Result<(), AttributionError> {
        if !present(&self.repository) {
            return Err(AttributionError::MissingRepository);
        }
        if !present(&self.path) {
            return Err(AttributionError::MissingPath);
        }
        if !present(&self.licence) {
            return Err(AttributionError::MissingLicence);
        }
        Ok(())
    }

    /// FRET examples need a real revision (the first build task records the
    /// commit it retrieved, FR-087); the `UNKNOWN` marker does not satisfy it.
    pub fn validate_fret(&self) -> Result<(), AttributionError> {
        self.common()?;
        if !present(&self.revision) || self.revision.as_deref() == Some(REVISION_UNKNOWN) {
            return Err(AttributionError::MissingRevision);
        }
        Ok(())
    }

    /// WEST artifacts accept the `UNKNOWN` marker but not an absent revision.
    pub fn validate_west(&self) -> Result<(), AttributionError> {
        self.common()?;
        if !present(&self.revision) {
            return Err(AttributionError::MissingRevision);
        }
        Ok(())
    }
}
