use serde::{Deserialize, Serialize};
use tl_syntax::{FormulaDocument, PropositionId};

/// Trace wire schema identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum TraceSchemaVersion {
    /// Initial finite discrete trace schema.
    #[serde(rename = "tl-mltl.trace/v1")]
    V1,
}

/// Versioned trace document consumed by the CLI.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TraceDocument {
    /// Wire identity.
    pub schema_version: TraceSchemaVersion,
    /// Stable trace identity.
    pub trace_id: String,
    /// Whether the trace is complete.
    pub closed: bool,
    /// Ordered true proposition identities at each discrete instant.
    pub instants: Vec<Vec<PropositionId>>,
}

/// Versioned CLI request document.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CommandDocument {
    /// Wire identity.
    pub schema_version: CommandSchemaVersion,
    /// Requested operation.
    pub operation: Operation,
    /// Stable formula identity.
    pub formula_id: String,
    /// Validated formula document.
    pub formula: FormulaDocument,
    /// Optional trace; required for evaluation.
    pub trace: Option<TraceDocument>,
}

/// CLI request schema identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum CommandSchemaVersion {
    /// Initial command schema.
    #[serde(rename = "tl-mltl.command/v1")]
    V1,
}

/// CLI operation vocabulary.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Evaluate a closed trace or open prefix according to formula profile.
    Evaluate,
    /// Compute static horizon resources.
    Analyze,
    /// Emit a C2PO mapping manifest.
    MapC2po,
}
