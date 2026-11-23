//! Assume-guarantee contract evaluation as a per-position outcome series
//! (FR-059, tl-syntax FR-309/FR-310, ADR-017 option C as decided 2026-11-23).
//!
//! There is no trace-level contract status: the report keeps the whole series
//! and a summary, and a consumer computes any whole-trace answer from it. The
//! status vocabulary, its truth table and its wire spellings are tl-syntax's
//! (`ContractOutcome`, `contract_outcome`, `as_str`); this module defines none.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{
    contract_outcome, ContractOutcome, ContractStatus, ContractTruth, Formula, FormulaDocument,
    NodeId, SemanticProfile,
};

use crate::horizon::lookahead;
use crate::numeric::evaluate::{evaluate_prepared, prepare};
use crate::numeric::{
    EmbeddedReport, NumericEvaluationError, NumericEvaluationLimits, NumericTraceDocument,
    PredicateTable,
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

/// Contract set bound to one formula graph by its recorded content identity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractSet {
    /// Content identity of the formula graph the roots refer to.
    pub graph_identity: String,
    /// Semantic profile the set was bound under.
    pub semantic_profile: SemanticProfile,
    /// Contracts in id order.
    pub contracts: Vec<ContractRoots>,
}

impl ContractSet {
    /// Converts a validated tl-syntax contract set (FR-309).
    pub fn from_tl_syntax(document: &tl_syntax::ContractSetDocument) -> Self {
        Self {
            graph_identity: document.graph_identity().to_owned(),
            semantic_profile: document.semantic_profile(),
            contracts: document
                .contracts()
                .iter()
                .map(|contract| ContractRoots {
                    id: contract.id,
                    name: contract.name.clone(),
                    assumption: contract.assumption,
                    guarantee: contract.guarantee,
                })
                .collect(),
        }
    }
}

/// Maps an evaluator truth value onto tl-syntax's contract input.
pub const fn contract_truth(value: TruthValue) -> ContractTruth {
    match value {
        TruthValue::True => ContractTruth::True,
        TruthValue::False => ContractTruth::False,
        TruthValue::Pending => ContractTruth::Pending,
    }
}

/// Serializes `ContractOutcome` by tl-syntax's own `as_str` spelling.
mod outcome_wire {
    use serde::{Deserialize, Deserializer, Serializer};
    use tl_syntax::{ContractOutcome, ContractStatus};

    pub(super) fn serialize<S: Serializer>(
        outcome: &ContractOutcome,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(outcome.as_str())
    }

    pub(super) fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<ContractOutcome, D::Error> {
        let spelling = String::deserialize(deserializer)?;
        let candidates = [
            ContractOutcome::Status(ContractStatus::Inactive),
            ContractOutcome::Status(ContractStatus::Invalid),
            ContractOutcome::Status(ContractStatus::Verified),
            ContractOutcome::Pending,
        ];
        candidates
            .into_iter()
            .find(|candidate| candidate.as_str() == spelling)
            .ok_or_else(|| serde::de::Error::custom(format!("unknown outcome {spelling:?}")))
    }
}

/// One position of a contract's series.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PositionOutcome {
    /// Evaluation position.
    pub time: u64,
    /// Outcome (tl-syntax FR-310 vocabulary and spelling).
    #[serde(with = "outcome_wire")]
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
            ContractOutcome::Status(ContractStatus::Inactive) => self.inactive += 1,
            ContractOutcome::Status(ContractStatus::Invalid) => self.invalid += 1,
            ContractOutcome::Status(ContractStatus::Verified) => self.verified += 1,
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
    /// Strict-reads a canonical report under the owner limits.
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
    /// The contract set's graph identity is not the supplied graph's.
    GraphIdentityMismatch {
        /// Identity carried by the set.
        expected: String,
        /// Identity of the supplied graph document.
        actual: String,
    },
    /// The set was bound under another profile than the graph's.
    ProfileMismatch {
        /// Profile recorded by the set.
        expected: &'static str,
        /// Profile of the graph.
        actual: &'static str,
    },
    /// A root is outside the graph.
    RootOutOfRange {
        /// Contract identity.
        contract: u32,
        /// Offending root.
        root: NodeId,
    },
    /// The graph document does not validate, or its identity cannot be computed.
    Identity(String),
    /// `positions x contracts x 2` root evaluations exceed the ceiling.
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
                "contract set is bound to graph {expected}, the graph is {actual}"
            ),
            Self::ProfileMismatch { expected, actual } => {
                write!(formatter, "contract set profile {expected}, graph profile {actual}")
            }
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
    /// Per-root evaluation limits. Each root evaluation is one evaluator call
    /// on rows prepared once, so its work is bounded by `numeric.evaluation`
    /// (or `numeric.past`) node limits.
    pub numeric: NumericEvaluationLimits,
    /// Ceiling on root evaluations (`positions x contracts x 2`), checked
    /// first. Total work is at most this times the per-call evaluator limit,
    /// plus one preparation bounded by `numeric.max_predicate_work`.
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

/// Checks identity, profile and root range of a contract set against a graph,
/// returning the validated formula view. Shared with the contract export.
pub(crate) fn bind_contracts<'g>(
    graph: &'g FormulaDocument,
    contracts: &ContractSet,
) -> Result<Formula<'g>, ContractEvaluationError> {
    let formula = graph
        .validate()
        .map_err(|error| ContractEvaluationError::Identity(format!("{error:?}")))?;
    let actual = graph
        .content_identity()
        .map_err(|error| ContractEvaluationError::Identity(error.to_string()))?;
    if actual != contracts.graph_identity {
        return Err(ContractEvaluationError::GraphIdentityMismatch {
            expected: contracts.graph_identity.clone(),
            actual,
        });
    }
    if contracts.semantic_profile != formula.profile() {
        return Err(ContractEvaluationError::ProfileMismatch {
            expected: contracts.semantic_profile.as_str(),
            actual: formula.profile().as_str(),
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
    Ok(formula)
}

/// The formula view rooted at `root`; operands precede their user, so the
/// prefix ending at `root` is a validated graph with the same node identities.
pub(crate) fn rooted<'f>(formula: Formula<'f>, root: NodeId) -> Formula<'f> {
    let nodes = &formula.nodes()[..=root.0 as usize];
    Formula::new(formula.profile(), root, nodes).expect("a prefix of a validated graph validates")
}

fn root_lookahead(formula: Formula<'_>, root: NodeId) -> u64 {
    lookahead(rooted(formula, root)).unwrap_or(u64::MAX)
}

/// Evaluates every contract at every observed position.
///
/// The graph is the caller's `FormulaDocument` (the edition whose content
/// identity the set recorded; contracts over LTLf or unbounded graphs are
/// refused because they are not this document type). `table` may be empty: a
/// numeric trace without values is a Boolean trace (FR-053). Predicate truth
/// and derived rows are computed once; each (root, position) is one evaluator
/// call on them.
///
/// Implements: FR-059
pub fn evaluate_contracts(
    graph: &FormulaDocument,
    formula_id: &str,
    contracts: &ContractSet,
    table: &PredicateTable,
    trace: &NumericTraceDocument,
    limits: ContractEvaluationLimits,
) -> Result<ContractEvaluationReport, ContractEvaluationError> {
    let formula = bind_contracts(graph, contracts)?;
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
    let prepared = prepare(formula.profile(), table, trace, limits.numeric)
        .map_err(ContractEvaluationError::Evaluation)?;
    let truth = |root: NodeId, time: u64| -> Result<ContractTruth, ContractEvaluationError> {
        let report = evaluate_prepared(
            &prepared,
            rooted(formula, root),
            formula_id,
            table,
            trace,
            time,
            limits.numeric,
        )
        .map_err(ContractEvaluationError::Evaluation)?;
        Ok(match report {
            EmbeddedReport::Future(inner) => contract_truth(inner.verdict),
            EmbeddedReport::Past(inner) => {
                if inner.verdict {
                    ContractTruth::True
                } else {
                    ContractTruth::False
                }
            }
        })
    };
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
            let outcome =
                contract_outcome(truth(contract.assumption, time)?, truth(contract.guarantee, time)?);
            let padded = closed_padding
                && last.is_some_and(|last| time.checked_add(reach).is_none_or(|end| end > last));
            if padded {
                summary.padded.add(outcome);
            } else {
                summary.unpadded.add(outcome);
                if outcome == ContractOutcome::Status(ContractStatus::Invalid)
                    && summary.first_invalid.is_none()
                {
                    summary.first_invalid = Some(time);
                }
                if outcome == ContractOutcome::Status(ContractStatus::Inactive)
                    && summary.first_inactive.is_none()
                {
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
