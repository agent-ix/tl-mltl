use serde::{Deserialize, Serialize};
use tl_syntax::RequirementContextDocument;

use crate::{
    context::contextual_result_sha256, ContextualEvaluationReport, EvaluationReport, TruthValue,
};

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

/// Closed schema identity for a context-aware external verdict.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContextualExternalVerdictSchemaVersion {
    /// Context-aware external verdict.
    #[serde(rename = "tl-mltl.external-verdict/v2")]
    V2,
}

/// External outcome with the exact context it claims to have consumed.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextualExternalVerdict {
    /// Closed v2 wire identity.
    pub schema_version: ContextualExternalVerdictSchemaVersion,
    /// External tool identity.
    pub tool: ToolIdentity,
    /// Formula identity used by the tool.
    pub formula_id: String,
    /// Trace identity used by the tool.
    pub trace_id: String,
    /// Claimed SHA-256 identity of the complete shared catalog.
    pub signal_catalog_sha256: String,
    /// Claimed exact caller context, or deliberate absence encoded as null.
    pub requirement_context: Option<RequirementContextDocument>,
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

/// Contextual comparison classification, including identity refusal.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContextualComparisonStatus {
    /// Truth value and verdict time agree after all identities match.
    Agreement,
    /// Two conclusive results disagree.
    Mismatch,
    /// The contextual identities do not match.
    IdentityMismatch,
    /// At least one result is pending, unsupported, malformed, or failed.
    NonConclusive,
}

/// Closed schema identity for a contextual differential report.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContextualDifferentialSchemaVersion {
    /// Contextual differential report.
    #[serde(rename = "tl-mltl.differential/v2")]
    V2,
}

/// Flat v2 differential record retaining the supplied external claim.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextualDifferentialReport {
    /// Closed v2 wire identity.
    pub schema_version: ContextualDifferentialSchemaVersion,
    /// Formula identity.
    pub formula_id: String,
    /// Trace identity.
    pub trace_id: String,
    /// Shared catalog identity.
    pub signal_catalog_sha256: String,
    /// Exact caller context, or deliberate absence encoded as null.
    pub requirement_context: Option<RequirementContextDocument>,
    /// Comparison classification.
    pub status: ContextualComparisonStatus,
    /// Reference truth value.
    pub reference_verdict: TruthValue,
    /// Reference verdict time.
    pub reference_verdict_time: u64,
    /// Complete contextual external outcome.
    pub external: ContextualExternalVerdict,
    /// Domain-separated comparison identity excluding only this field itself.
    pub comparison_sha256: String,
    /// Deterministic explanation.
    pub detail: String,
}

/// A contextual comparison record could not be serialized for its identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualComparisonError(pub String);

impl core::fmt::Display for ContextualComparisonError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            formatter,
            "contextual comparison serialization failed: {}",
            self.0
        )
    }
}

impl std::error::Error for ContextualComparisonError {}

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

/// Compares a contextual external claim without executing or impersonating its tool.
pub fn compare_external_with_context(
    reference: &ContextualEvaluationReport,
    external: ContextualExternalVerdict,
) -> Result<ContextualDifferentialReport, ContextualComparisonError> {
    let identities_match = reference.formula_id == external.formula_id
        && reference.trace_id == external.trace_id
        && reference.signal_catalog_sha256 == external.signal_catalog_sha256
        && reference.requirement_context == external.requirement_context;
    let reference_value = match reference.verdict {
        TruthValue::False => Some(false),
        TruthValue::True => Some(true),
        TruthValue::Pending => None,
    };
    let (status, detail) = if !identities_match {
        (
            ContextualComparisonStatus::IdentityMismatch,
            "formula, trace, catalog, or requirement context identity mismatch".to_owned(),
        )
    } else if external.status != ExternalStatus::Conclusive || reference_value.is_none() {
        (
            ContextualComparisonStatus::NonConclusive,
            format!("non-conclusive external status: {:?}", external.status),
        )
    } else if external.value == reference_value
        && external.verdict_time == Some(reference.verdict_time)
    {
        (
            ContextualComparisonStatus::Agreement,
            "truth value and verdict time agree".to_owned(),
        )
    } else {
        (
            ContextualComparisonStatus::Mismatch,
            "truth value or verdict time differs".to_owned(),
        )
    };
    let mut report = ContextualDifferentialReport {
        schema_version: ContextualDifferentialSchemaVersion::V2,
        formula_id: reference.formula_id.clone(),
        trace_id: reference.trace_id.clone(),
        signal_catalog_sha256: reference.signal_catalog_sha256.clone(),
        requirement_context: reference.requirement_context.clone(),
        status,
        reference_verdict: reference.verdict,
        reference_verdict_time: reference.verdict_time,
        external,
        comparison_sha256: String::new(),
        detail,
    };
    report.comparison_sha256 =
        contextual_result_sha256("tl-mltl.contextual-differential/v2/comparison", &report)
            .map_err(|error| ContextualComparisonError(error.to_string()))?;
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::{
        compare_external_with_context, ContextualComparisonStatus, ContextualExternalVerdict,
        ContextualExternalVerdictSchemaVersion, ExternalStatus, ToolIdentity,
    };
    use crate::{ContextualEvaluationReport, ContextualEvaluationSchemaVersion, TruthValue};

    fn reference() -> ContextualEvaluationReport {
        ContextualEvaluationReport {
            schema_version: ContextualEvaluationSchemaVersion::V2,
            source_revision: "source".to_owned(),
            syntax_revision: "syntax".to_owned(),
            signal_catalog_sha256: "catalog".to_owned(),
            requirement_context: None,
            request_sha256: "request".to_owned(),
            result_sha256: "result".to_owned(),
            formula_id: "formula".to_owned(),
            formula_root: 0,
            semantic_profile: "mltl.closed-trace/v1".to_owned(),
            trace_id: "trace".to_owned(),
            trace_length: 1,
            trace_closed: true,
            verdict: TruthValue::True,
            verdict_time: 0,
            observed_through: Some(0),
            horizon: 0,
            proposition_ids: vec![7],
        }
    }

    fn external(catalog: &str) -> ContextualExternalVerdict {
        ContextualExternalVerdict {
            schema_version: ContextualExternalVerdictSchemaVersion::V2,
            tool: ToolIdentity {
                name: "external".to_owned(),
                version: "1".to_owned(),
                executable_sha256: "binary".to_owned(),
                configuration_sha256: "config".to_owned(),
            },
            formula_id: "formula".to_owned(),
            trace_id: "trace".to_owned(),
            signal_catalog_sha256: catalog.to_owned(),
            requirement_context: None,
            status: ExternalStatus::Conclusive,
            value: Some(true),
            verdict_time: Some(0),
            detail: None,
        }
    }

    #[test]
    fn contextual_comparison_refuses_identity_mismatch_before_truth_comparison() {
        let report = compare_external_with_context(&reference(), external("other")).unwrap();
        assert_eq!(report.status, ContextualComparisonStatus::IdentityMismatch);
        assert!(!report.comparison_sha256.is_empty());
    }
}
