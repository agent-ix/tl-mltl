use serde::{Deserialize, Serialize};

use crate::{EvaluationReport, TruthValue};

/// Exact external-tool identity retained with adapter and differential evidence.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolIdentity {
    /// Tool or engine name.
    pub name: String,
    /// Exact version string.
    pub version: String,
    /// SHA-256 digest of the executed binary.
    pub executable_sha256: String,
    /// SHA-256 digest of material configuration.
    pub configuration_sha256: String,
}

/// External monitor execution state.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExternalStatus {
    /// External tool produced a Boolean at a named time.
    Conclusive,
    /// External tool has not reached a decision.
    Pending,
    /// Adapter or engine does not support the case.
    Unsupported,
    /// External execution failed.
    ToolError,
}

/// Identified external outcome supplied to the pure comparison layer.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ExternalVerdict {
    /// Wire identity.
    pub schema_version: String,
    /// External tool identity.
    pub tool: ToolIdentity,
    /// Formula identity used by the tool.
    pub formula_id: String,
    /// Trace identity used by the tool.
    pub trace_id: String,
    /// Typed execution state.
    pub status: ExternalStatus,
    /// Boolean value, present only for a conclusive status.
    pub value: Option<bool>,
    /// Verdict time, present only for a conclusive status.
    pub verdict_time: Option<u64>,
    /// Stable unsupported/error explanation where applicable.
    pub detail: Option<String>,
}

/// Differential comparison classification.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonStatus {
    /// Truth value and verdict time agree.
    Agreement,
    /// Two conclusive results disagree.
    Mismatch,
    /// At least one result is pending, unsupported, malformed, or failed.
    NonConclusive,
}

/// Versioned machine-readable differential record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DifferentialReport {
    /// Wire identity.
    pub schema_version: String,
    /// Formula identity.
    pub formula_id: String,
    /// Trace identity.
    pub trace_id: String,
    /// Comparison classification.
    pub status: ComparisonStatus,
    /// Reference truth value.
    pub reference_verdict: TruthValue,
    /// Reference verdict time.
    pub reference_verdict_time: u64,
    /// Complete external outcome.
    pub external: ExternalVerdict,
    /// Deterministic explanation.
    pub detail: String,
}

/// Compares an external result without executing or impersonating its engine.
///
/// Implements: FR-005
pub fn compare_external(
    reference: &EvaluationReport,
    external: ExternalVerdict,
) -> DifferentialReport {
    let identities_match =
        reference.formula_id == external.formula_id && reference.trace_id == external.trace_id;
    let reference_value = match reference.verdict {
        TruthValue::False => Some(false),
        TruthValue::True => Some(true),
        TruthValue::Pending => None,
    };
    let (status, detail) = if !identities_match {
        (
            ComparisonStatus::Mismatch,
            "formula or trace identity mismatch".to_owned(),
        )
    } else if external.status != ExternalStatus::Conclusive || reference_value.is_none() {
        (
            ComparisonStatus::NonConclusive,
            format!("non-conclusive external status: {:?}", external.status),
        )
    } else if external.value == reference_value
        && external.verdict_time == Some(reference.verdict_time)
    {
        (
            ComparisonStatus::Agreement,
            "truth value and verdict time agree".to_owned(),
        )
    } else {
        (
            ComparisonStatus::Mismatch,
            "truth value or verdict time differs".to_owned(),
        )
    };
    DifferentialReport {
        schema_version: "tl-mltl.differential/v1".to_owned(),
        formula_id: reference.formula_id.clone(),
        trace_id: reference.trace_id.clone(),
        status,
        reference_verdict: reference.verdict,
        reference_verdict_time: reference.verdict_time,
        external,
        detail,
    }
}
