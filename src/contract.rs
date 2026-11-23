//! Assume-guarantee contract evaluation as a per-position outcome series
//! (FR-059, tl-syntax FR-309/FR-310, ADR-017 option C as decided 2026-11-23).
//!
//! There is no trace-level contract status: the report keeps the whole series
//! and a summary, and a consumer computes any whole-trace answer from it.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{Formula, FormulaDocument, NodeId, SemanticProfile};

use crate::horizon::lookahead;
use crate::numeric::{
    evaluate_numeric, EmbeddedReport, NumericEvaluationError, NumericEvaluationLimits,
    NumericTraceDocument, PredicateTable,
};
use crate::wire::common::read as read_canonical;
use crate::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};
use crate::TruthValue;

/// Contract evaluation report identity.
pub const CONTRACT_EVALUATION_V1: &str = "tl-mltl.contract-evaluation/v1";

/// One contract: a named pair of roots in one formula graph.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractRoots {
    /// Contract identity (strictly increasing within a set).
    pub id: u32,
    /// Contract name.
    pub name: String,
    /// Assumption root.
    pub assumption: NodeId,
    /// Guarantee root.
    pub guarantee: NodeId,
}

/// Contract set bound to a formula graph by content identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractSet {
    /// Content identity of the formula graph the roots refer to.
    pub graph_identity: String,
    /// Contracts in id order.
    pub contracts: Vec<ContractRoots>,
}

impl ContractSet {
    /// Converts a validated tl-syntax contract set.
    // UNBUILT-PIN: tl-syntax FR-309/FR-311 (`ContractSetDocument`, `ContractView`)
    // is not merged; accessor names below are guesses at the spec'd shape.
    pub fn from_tl_syntax(document: &tl_syntax::ContractSetDocument) -> Self {
        Self {
            graph_identity: document.graph_identity().to_owned(),
            contracts: document
                .contracts()
                .iter()
                .map(|contract| ContractRoots {
                    id: contract.id(),
                    name: contract.name().to_owned(),
                    assumption: contract.assumption(),
                    guarantee: contract.guarantee(),
                })
                .collect(),
        }
    }
}

/// Outcome at one position (tl-syntax FR-310 vocabulary and wire spellings).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ContractOutcome {
    /// Assumption false.
    Inactive,
    /// Assumption true, guarantee false.
    Invalid,
    /// Both true.
    Verified,
    /// Not decided.
    Pending,
}

/// The FR-310 truth table.
// UNBUILT-PIN: tl-syntax FR-310 owns this table ("one definition"); replace
// this mirror with the merged tl-syntax status function once it lands.
pub const fn contract_outcome(assumption: TruthValue, guarantee: TruthValue) -> ContractOutcome {
    match (assumption, guarantee) {
        (TruthValue::False, _) => ContractOutcome::Inactive,
        (TruthValue::True, TruthValue::False) => ContractOutcome::Invalid,
        (TruthValue::True, TruthValue::True) => ContractOutcome::Verified,
        _ => ContractOutcome::Pending,
    }
}

/// One position of a contract's series.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PositionOutcome {
    /// Evaluation position.
    pub time: u64,
    /// Outcome.
    pub outcome: ContractOutcome,
    /// The outcome depends on closed-trace padding.
    pub padded: bool,
}

/// Counts of each outcome.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OutcomeCounts {
    /// `inactive` count.
    pub inactive: u64,
    /// `invalid` count.
    pub invalid: u64,
    /// `verified` count.
    pub verified: u64,
    /// `pending` count.
    pub pending: u64,
}

impl OutcomeCounts {
    fn add(&mut self, outcome: ContractOutcome) {
        match outcome {
            ContractOutcome::Inactive => self.inactive += 1,
            ContractOutcome::Invalid => self.invalid += 1,
            ContractOutcome::Verified => self.verified += 1,
            ContractOutcome::Pending => self.pending += 1,
        }
    }
}

/// Summary of one series.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractSummary {
    /// Counts over unpadded positions.
    pub unpadded: OutcomeCounts,
    /// Counts over padded positions.
    pub padded: OutcomeCounts,
    /// First unpadded `invalid` time.
    pub first_invalid: Option<u64>,
    /// First unpadded `inactive` time.
    pub first_inactive: Option<u64>,
}

/// One contract's series.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractSeries {
    /// Contract identity.
    pub id: u32,
    /// Contract name.
    pub name: String,
    /// Per-position outcomes, time 0 through the last observed position.
    pub outcomes: Vec<PositionOutcome>,
    /// Summary.
    pub summary: ContractSummary,
}

/// `tl-mltl.contract-evaluation/v1`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractEvaluationReport {
    /// Wire identity.
    pub schema_version: String,
    /// Formula identity.
    pub formula_id: String,
    /// Graph identity the contract set is bound to.
    pub graph_identity: String,
    /// Semantic profile identity.
    pub semantic_profile: String,
    /// Trace identity.
    pub trace_id: String,
    /// The trace has zero instants.
    pub empty_trace: bool,
    /// One series per contract, in set order.
    pub contracts: Vec<ContractSeries>,
}

impl ContractEvaluationReport {
    /// Strict-reads a canonical report.
    pub fn from_json_bytes(bytes: &[u8], limits: OwnerLimits) -> Result<Self, OwnerReadError> {
        let (report, _) = read_canonical(bytes, limits, |report: &Self, _| {
            if report.schema_version == CONTRACT_EVALUATION_V1 {
                Ok(OwnerUsage::default())
            } else {
                Err(OwnerReadError::new(
                    OwnerReadErrorCode::InvalidCombination,
                    "schemaVersion",
                    OwnerUsage::default(),
                ))
            }
        })?;
        Ok(report)
    }

    /// Canonical bytes.
    pub fn canonical_json_bytes(&self) -> Vec<u8> {
        serde_json::to_vec(self).expect("contract report serializes")
    }
}

/// Typed refusal; never outcomes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractEvaluationError {
    /// The contract set's graph identity is not the supplied formula's.
    GraphIdentityMismatch {
        /// Identity carried by the set.
        expected: String,
        /// Identity of the supplied formula (v2 edition).
        actual: String,
    },
    /// A root is outside the graph.
    RootOutOfRange {
        /// Contract identity.
        contract: u32,
        /// Offending root.
        root: NodeId,
    },
    /// Identity computation failed.
    Identity(String),
    /// `positions x contracts x 2` exceeds the ceiling, checked first.
    WorkLimitExceeded {
        /// Requested root evaluations.
        requested: u128,
        /// Limit.
        limit: u64,
    },
    /// Root evaluation failed.
    Evaluation(NumericEvaluationError),
}

impl fmt::Display for ContractEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GraphIdentityMismatch { expected, actual } => write!(
                formatter,
                "contract set is bound to graph {expected}, formula is {actual}"
            ),
            Self::RootOutOfRange { contract, root } => {
                write!(formatter, "contract {contract} root {} out of range", root.0)
            }
            Self::Identity(error) => write!(formatter, "graph identity failed: {error}"),
            Self::WorkLimitExceeded { requested, limit } => {
                write!(formatter, "{requested} root evaluations exceed limit {limit}")
            }
            Self::Evaluation(error) => error.fmt(formatter),
        }
    }
}

impl std::error::Error for ContractEvaluationError {}

/// Limits of a contract evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ContractEvaluationLimits {
    /// Per-root evaluation limits.
    pub numeric: NumericEvaluationLimits,
    /// Ceiling on root evaluations (`positions x contracts x 2`).
    pub max_root_evaluations: u64,
}

impl Default for ContractEvaluationLimits {
    fn default() -> Self {
        Self {
            numeric: NumericEvaluationLimits::default(),
            max_root_evaluations: 10_000_000,
        }
    }
}

fn content_identity(
    document: Result<FormulaDocument, tl_syntax::FormulaError>,
) -> Result<String, ContractEvaluationError> {
    document
        .map_err(|error| ContractEvaluationError::Identity(error.to_string()))?
        .content_identity()
        .map_err(|error| ContractEvaluationError::Identity(error.to_string()))
}

// SPEC-GAP: FR-309 binds by "the canonical formula bytes" without naming the
// edition; the v2 identity (every profile) and, where the formula fits v1, the
// v1 identity are both accepted.
fn graph_identities(formula: Formula<'_>) -> Result<Vec<String>, ContractEvaluationError> {
    let mut identities = vec![content_identity(FormulaDocument::from_formula_v2(formula))?];
    if let Ok(v1) = content_identity(FormulaDocument::from_formula(formula)) {
        identities.push(v1);
    }
    Ok(identities)
}

fn root_truth(
    formula: Formula<'_>,
    root: NodeId,
    formula_id: &str,
    table: &PredicateTable,
    trace: &NumericTraceDocument,
    time: u64,
    limits: NumericEvaluationLimits,
) -> Result<TruthValue, ContractEvaluationError> {
    // Operands precede their users, so the prefix ending at `root` is a
    // validated graph with the same node identities.
    let nodes = &formula.nodes()[..=root.0 as usize];
    let rooted = Formula::new(formula.profile(), root, nodes).map_err(|_| {
        ContractEvaluationError::RootOutOfRange { contract: u32::MAX, root }
    })?;
    let report = evaluate_numeric(rooted, formula_id, table, trace, time, limits)
        .map_err(ContractEvaluationError::Evaluation)?;
    Ok(match report.evaluation {
        EmbeddedReport::Future(inner) => inner.verdict,
        EmbeddedReport::Past(inner) => {
            if inner.verdict {
                TruthValue::True
            } else {
                TruthValue::False
            }
        }
    })
}

fn root_lookahead(formula: Formula<'_>, root: NodeId) -> u64 {
    let nodes = &formula.nodes()[..=root.0 as usize];
    Formula::new(formula.profile(), root, nodes)
        .ok()
        .and_then(|rooted| lookahead(rooted).ok())
        .unwrap_or(u64::MAX)
}

/// Evaluates every contract at every observed position.
///
/// `table` may be empty: a numeric trace without values is a Boolean trace
/// (FR-053). The profile of `formula` selects the evaluator; the origin-complete
/// profile uses the position as anchor.
///
/// Implements: FR-059
pub fn evaluate_contracts(
    formula: Formula<'_>,
    formula_id: &str,
    contracts: &ContractSet,
    table: &PredicateTable,
    trace: &NumericTraceDocument,
    limits: ContractEvaluationLimits,
) -> Result<ContractEvaluationReport, ContractEvaluationError> {
    let identities = graph_identities(formula)?;
    if !identities.contains(&contracts.graph_identity) {
        return Err(ContractEvaluationError::GraphIdentityMismatch {
            expected: contracts.graph_identity.clone(),
            actual: identities[0].clone(),
        });
    }
    for contract in &contracts.contracts {
        for root in [contract.assumption, contract.guarantee] {
            if root.0 as usize >= formula.nodes().len() {
                return Err(ContractEvaluationError::RootOutOfRange {
                    contract: contract.id,
                    root,
                });
            }
        }
    }
    let positions = trace.instants.len();
    let requested = (positions as u128)
        .saturating_mul(contracts.contracts.len() as u128)
        .saturating_mul(2);
    if requested > u128::from(limits.max_root_evaluations) {
        return Err(ContractEvaluationError::WorkLimitExceeded {
            requested,
            limit: limits.max_root_evaluations,
        });
    }
    let closed_padding = formula.profile() == SemanticProfile::ClosedTraceV1;
    let last = positions.checked_sub(1).map(|last| last as u64);
    let mut series = Vec::with_capacity(contracts.contracts.len());
    for contract in &contracts.contracts {
        // Lookahead is defined only for future profiles; padding only exists
        // on the closed profile.
        let reach = if closed_padding {
            root_lookahead(formula, contract.assumption)
                .max(root_lookahead(formula, contract.guarantee))
        } else {
            0
        };
        let mut outcomes = Vec::with_capacity(positions);
        let mut summary = ContractSummary::default();
        for time in 0..positions as u64 {
            let assumption = root_truth(
                formula,
                contract.assumption,
                formula_id,
                table,
                trace,
                time,
                limits.numeric,
            )?;
            let guarantee = root_truth(
                formula,
                contract.guarantee,
                formula_id,
                table,
                trace,
                time,
                limits.numeric,
            )?;
            let outcome = contract_outcome(assumption, guarantee);
            let padded = closed_padding
                && last.is_some_and(|last| time.checked_add(reach).is_none_or(|end| end > last));
            if padded {
                summary.padded.add(outcome);
            } else {
                summary.unpadded.add(outcome);
                if outcome == ContractOutcome::Invalid && summary.first_invalid.is_none() {
                    summary.first_invalid = Some(time);
                }
                if outcome == ContractOutcome::Inactive && summary.first_inactive.is_none() {
                    summary.first_inactive = Some(time);
                }
            }
            outcomes.push(PositionOutcome {
                time,
                outcome,
                padded,
            });
        }
        series.push(ContractSeries {
            id: contract.id,
            name: contract.name.clone(),
            outcomes,
            summary,
        });
    }
    Ok(ContractEvaluationReport {
        schema_version: CONTRACT_EVALUATION_V1.to_owned(),
        formula_id: formula_id.to_owned(),
        graph_identity: contracts.graph_identity.clone(),
        semantic_profile: formula.profile().as_str().to_owned(),
        trace_id: trace.trace_id.clone(),
        empty_trace: positions == 0,
        contracts: series,
    })
}
