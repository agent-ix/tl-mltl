//! Machine-readable evaluation record (`tl-mltl.evaluation-record/v1`).
//!
//! One record carries everything a consumer needs to explain one evaluation:
//! the trace and window read from the request, the verdict and position the
//! evaluator returned, and the toolchain and semantic-profile identities.

use serde::Serialize;
use sha2::{Digest, Sha256};
use tl_syntax::SemanticProfile;

use crate::{
    evaluate_closed, evaluate_prefix, CommandDocument, EvaluationLimits, EvaluationReport,
    Operation, TruthValue,
};

/// Record schema identity.
pub const RECORD_SCHEMA_VERSION: &str = "tl-mltl.evaluation-record/v1";
/// Exact checked-in JSON Schema bytes for [`RECORD_SCHEMA_VERSION`].
pub const RECORD_SCHEMA_BYTES: &[u8] =
    include_bytes!("../schemas/evaluation-record-v1.schema.json");

/// Evaluates one `evaluate` command with the default limits.
///
/// Every non-closed profile goes to `evaluate_prefix`, which refuses a profile
/// other than online-prefix with a typed error.
pub fn evaluate_command(request: &CommandDocument) -> Result<EvaluationReport, String> {
    let formula = request
        .formula
        .validate()
        .map_err(|error| format!("validate formula: {error}"))?;
    let trace = request
        .trace
        .as_ref()
        .ok_or_else(|| "evaluate operation requires trace".to_owned())?;
    let limits = EvaluationLimits::default();
    if formula.profile() == SemanticProfile::ClosedTraceV1 {
        if !trace.closed {
            return Err("closed-trace formula requires a closed trace document".to_owned());
        }
        evaluate_closed(
            formula,
            request.formula_id.clone(),
            &trace.instants,
            trace.trace_id.clone(),
            limits,
        )
    } else {
        evaluate_prefix(
            formula,
            request.formula_id.clone(),
            &trace.instants,
            trace.trace_id.clone(),
            trace.closed,
            limits,
        )
    }
    .map_err(|error| format!("evaluate formula: {error}"))
}

/// Verdict spelled as the record exposes it.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub enum RecordVerdict {
    /// Conclusively true.
    True,
    /// Conclusively false.
    False,
    /// Not yet decided by the observed prefix.
    Pending,
}

impl From<TruthValue> for RecordVerdict {
    fn from(value: TruthValue) -> Self {
        match value {
            TruthValue::True => Self::True,
            TruthValue::False => Self::False,
            TruthValue::Pending => Self::Pending,
        }
    }
}

/// Toolchain identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct RecordToolchain {
    /// Producer name.
    pub name: &'static str,
    /// Producer version.
    pub version: &'static str,
}

/// Trace exactly as read from the request.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecordTrace {
    /// Trace identity from the request.
    pub trace_id: String,
    /// Whether the request declared the trace closed.
    pub closed: bool,
    /// True proposition ids at each instant.
    pub instants: Vec<Vec<u32>>,
}

/// Inclusive evaluated window `[start, end]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct RecordWindow {
    /// First instant the formula can read (the evaluation position).
    pub start: u64,
    /// Last instant the formula can read (position plus lookahead horizon).
    pub end: u64,
}

/// One evaluation record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvaluationRecord {
    /// Record schema identity.
    pub schema_version: &'static str,
    /// Toolchain that produced the record.
    pub toolchain: RecordToolchain,
    /// Semantic profile name (`ClosedTraceV1` or `OnlinePrefixV1`).
    pub semantic_profile: &'static str,
    /// Wire identity of the semantic profile.
    pub semantic_profile_id: String,
    /// Formula identity from the request.
    pub formula_id: String,
    /// Lowercase SHA-256 of the request file bytes.
    pub input_sha256: String,
    /// Trace read from the request.
    pub trace: RecordTrace,
    /// Evaluated window.
    pub window: RecordWindow,
    /// Formula-time index the verdict applies to.
    pub evaluation_position: u64,
    /// Evaluator verdict.
    pub verdict: RecordVerdict,
    /// Last observed instant, or null for an empty trace.
    pub observed_through: Option<u64>,
    /// Static decision horizon.
    pub horizon: u64,
    /// Witness or counterexample; null because the evaluator reports none.
    pub witness: Option<serde_json::Value>,
}

/// Builds the record for the request file bytes `input`.
pub fn build_record(input: &[u8]) -> Result<EvaluationRecord, String> {
    let request: CommandDocument =
        serde_json::from_slice(input).map_err(|error| format!("parse request: {error}"))?;
    if request.operation != Operation::Evaluate {
        return Err("evaluation record requires operation evaluate".to_owned());
    }
    let report = evaluate_command(&request)?;
    let trace = request
        .trace
        .ok_or_else(|| "evaluate operation requires trace".to_owned())?;
    let semantic_profile = match report.semantic_profile.as_str() {
        "mltl.closed-trace/v1" => "ClosedTraceV1",
        "mltl.online-prefix/v1" => "OnlinePrefixV1",
        other => return Err(format!("unsupported semantic profile {other}")),
    };
    let digest = Sha256::digest(input);
    let input_sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    Ok(EvaluationRecord {
        schema_version: RECORD_SCHEMA_VERSION,
        toolchain: RecordToolchain {
            name: "tl-mltl",
            version: env!("CARGO_PKG_VERSION"),
        },
        semantic_profile,
        semantic_profile_id: report.semantic_profile,
        formula_id: report.formula_id,
        input_sha256,
        trace: RecordTrace {
            trace_id: trace.trace_id,
            closed: trace.closed,
            instants: trace
                .instants
                .iter()
                .map(|instant| instant.iter().map(|id| id.0).collect())
                .collect(),
        },
        window: RecordWindow {
            start: report.verdict_time,
            end: report.verdict_time + report.horizon,
        },
        evaluation_position: report.verdict_time,
        verdict: report.verdict.into(),
        observed_through: report.observed_through,
        horizon: report.horizon,
        witness: None,
    })
}
