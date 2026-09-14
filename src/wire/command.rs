//! Strict owner boundary for the compatibility `tl-mltl.command/v1` contract.

use super::common::{produce, read as read_canonical, OwnerDocument};
use super::{CommandDocument, OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};
use crate::wire::trace::ValidatedTrace;
use tl_syntax::{StrictDocumentReadError, SyntaxArtifactLimits};

/// Immutable command contract label.
pub const CONTRACT: &str = "tl-mltl.command/v1";
/// Exact checked-in JSON Schema bytes for [`CONTRACT`].
pub const SCHEMA_BYTES: &[u8] = include_bytes!("../../schemas/command-v1.schema.json");
/// Lowercase SHA-256 digest of [`SCHEMA_BYTES`].
pub const SCHEMA_SHA256: &str = "71d10039c482a00885802611009196a1a0472d5af522fb5ff8b82ada8d68780c";

/// Constructor-private compatibility command view.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidatedCommand {
    document: CommandDocument,
    bytes: Vec<u8>,
    usage: OwnerUsage,
}

impl ValidatedCommand {
    /// Strict-reads one canonical command and every embedded owner artifact.
    pub fn from_json_bytes(bytes: &[u8], limits: OwnerLimits) -> Result<Self, OwnerReadError> {
        let (document, usage) = read_canonical(bytes, limits, validate)?;
        Ok(Self {
            document,
            bytes: bytes.to_vec(),
            usage,
        })
    }

    /// Validated compatibility command.
    #[must_use]
    pub const fn document(&self) -> &CommandDocument {
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

/// Strict-reads a canonical command against an independently supplied command.
pub fn read(
    bytes: &[u8],
    expected: &CommandDocument,
    limits: OwnerLimits,
) -> Result<ValidatedCommand, OwnerReadError> {
    let validated = ValidatedCommand::from_json_bytes(bytes, limits)?;
    if validated.document() != expected {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::ExpectedMismatch,
            "command",
            validated.usage(),
        ));
    }
    Ok(validated)
}

/// Produces a canonical compatibility command.
pub fn derive(
    document: &CommandDocument,
    limits: OwnerLimits,
) -> Result<OwnerDocument<CommandDocument>, OwnerReadError> {
    produce(
        document.clone(),
        validate(document, limits.effective())?,
        limits,
    )
}

fn validate(document: &CommandDocument, limits: OwnerLimits) -> Result<OwnerUsage, OwnerReadError> {
    if document.formula_id.is_empty()
        || document.formula_id.len() > limits.max_string_bytes.min(256)
    {
        return Err(OwnerReadError::new(
            OwnerReadErrorCode::InvalidCombination,
            "formulaId",
            OwnerUsage::default(),
        ));
    }
    let formula_bytes = document.formula.canonical_json_bytes().map_err(|_| {
        OwnerReadError::new(
            OwnerReadErrorCode::Encoding,
            "formula",
            OwnerUsage::default(),
        )
    })?;
    tl_syntax::FormulaDocument::from_json_bytes(
        &formula_bytes,
        SyntaxArtifactLimits {
            document_bytes: limits.max_input_bytes,
            json_depth: limits.max_depth,
            string_bytes: limits.max_string_bytes,
            formula_nodes: limits.max_formula_nodes,
            signals: limits.max_propositions,
            bindings: limits.max_propositions,
            propositions: limits.max_propositions,
            formula_depth: limits.max_formula_depth,
            work: limits.max_visited_fields,
        },
    )
    .map_err(|error| match error {
        StrictDocumentReadError::DocumentTooLarge { .. }
        | StrictDocumentReadError::DepthLimitExceeded { .. }
        | StrictDocumentReadError::StringTooLarge { .. }
        | StrictDocumentReadError::WorkLimitExceeded { .. }
        | StrictDocumentReadError::ResourceLimitExceeded { .. } => OwnerReadError::new(
            OwnerReadErrorCode::ResourceIncomplete,
            "formula",
            OwnerUsage::default(),
        ),
        StrictDocumentReadError::NonCanonicalDocument
        | StrictDocumentReadError::InvalidDocument(_)
        | _ => OwnerReadError::new(
            OwnerReadErrorCode::ExpectedMismatch,
            "formula",
            OwnerUsage::default(),
        ),
    })?;
    let trace_usage = if let Some(trace) = &document.trace {
        ValidatedTrace::admit(trace, limits)?.usage()
    } else {
        OwnerUsage::default()
    };
    Ok(OwnerUsage {
        formula_nodes: document.formula.nodes().len(),
        positions: trace_usage.positions,
        propositions: trace_usage.propositions,
        ..OwnerUsage::default()
    })
}
