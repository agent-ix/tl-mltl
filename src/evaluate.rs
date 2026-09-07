use core::fmt;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use tl_syntax::{
    Formula, NodeId, NodeKind, PropositionId, RequirementContextDocument, SemanticProfile,
    SignalCatalogDocument,
};

use crate::{
    context::{bind_formula, catalog_sha256, contextual_request_sha256, contextual_result_sha256},
    horizon::lookahead,
    ContextualBindingError, HorizonError, MAX_RECURSION_DEPTH, TL_SYNTAX_REVISION,
};

/// Three-valued result for closed and open-prefix evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TruthValue {
    /// Formula is conclusively false for every permitted continuation.
    False,
    /// Formula is conclusively true for every permitted continuation.
    True,
    /// Available observations do not yet decide the formula.
    Pending,
}

impl TruthValue {
    const fn not(self) -> Self {
        match self {
            Self::False => Self::True,
            Self::True => Self::False,
            Self::Pending => Self::Pending,
        }
    }

    const fn and(self, other: Self) -> Self {
        match (self, other) {
            (Self::False, _) | (_, Self::False) => Self::False,
            (Self::True, Self::True) => Self::True,
            _ => Self::Pending,
        }
    }

    const fn or(self, other: Self) -> Self {
        match (self, other) {
            (Self::True, _) | (_, Self::True) => Self::True,
            (Self::False, Self::False) => Self::False,
            _ => Self::Pending,
        }
    }
}

/// Hard limits applied before or during temporal expansion.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct EvaluationLimits {
    /// Maximum number of recursive node/time evaluations.
    pub max_node_evaluations: u64,
    /// Maximum number of offsets in one temporal interval.
    pub max_temporal_span: u64,
    /// Maximum recursive node depth for evaluation.
    pub max_recursion_depth: u32,
}

impl Default for EvaluationLimits {
    fn default() -> Self {
        Self {
            max_node_evaluations: 1_000_000,
            max_temporal_span: 100_000,
            max_recursion_depth: MAX_RECURSION_DEPTH,
        }
    }
}

/// Identity-bearing evaluation outcome.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvaluationReport {
    /// Wire identity.
    pub schema_version: String,
    /// Caller-provided formula identity.
    pub formula_id: String,
    /// Root node identity.
    pub formula_root: u32,
    /// Exact semantic-profile identity.
    pub semantic_profile: String,
    /// Caller-provided trace identity.
    pub trace_id: String,
    /// Number of observed instants.
    pub trace_length: u64,
    /// Whether the supplied trace was declared closed.
    pub trace_closed: bool,
    /// Boolean or pending result.
    pub verdict: TruthValue,
    /// Caller-selected formula-time index represented by this verdict.
    pub verdict_time: u64,
    /// Last observation index available, or `None` when the trace is empty.
    pub observed_through: Option<u64>,
    /// Static worst-case decision horizon.
    pub horizon: u64,
    /// Referenced proposition identities in sorted order.
    pub proposition_ids: Vec<u32>,
}

/// Closed schema identity for a context-bound evaluation record.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContextualEvaluationSchemaVersion {
    /// Context-bound evaluation report.
    #[serde(rename = "tl-mltl.evaluation/v2")]
    V2,
}

/// Flat v2 evaluation report carrying shared catalog and caller context identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextualEvaluationReport {
    /// Closed v2 wire identity.
    pub schema_version: ContextualEvaluationSchemaVersion,
    /// Exact tl-mltl source revision compiled into this result.
    pub source_revision: String,
    /// Exact shared tl-syntax dependency revision.
    pub syntax_revision: String,
    /// SHA-256 of the complete shared signal catalog document.
    pub signal_catalog_sha256: String,
    /// Exact caller context, or deliberate absence encoded as null.
    pub requirement_context: Option<RequirementContextDocument>,
    /// Domain-separated identity of the complete operation request.
    pub request_sha256: String,
    /// Domain-separated identity of this result excluding this field itself.
    pub result_sha256: String,
    /// Caller-provided formula identity.
    pub formula_id: String,
    /// Root node identity.
    pub formula_root: u32,
    /// Exact semantic-profile identity.
    pub semantic_profile: String,
    /// Caller-provided trace identity.
    pub trace_id: String,
    /// Number of observed instants.
    pub trace_length: u64,
    /// Whether the supplied trace was declared closed.
    pub trace_closed: bool,
    /// Boolean or pending result.
    pub verdict: TruthValue,
    /// Caller-selected formula-time index represented by this verdict.
    pub verdict_time: u64,
    /// Last observation index available, or `None` when the trace is empty.
    pub observed_through: Option<u64>,
    /// Static worst-case decision horizon.
    pub horizon: u64,
    /// Referenced proposition identities in sorted order.
    pub proposition_ids: Vec<u32>,
}

/// Failure from a context-bound evaluation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualEvaluationError {
    /// The shared catalog does not bind the formula.
    Binding(ContextualBindingError),
    /// Existing evaluation semantics refused the operation.
    Evaluation(EvaluationError),
    /// The deterministic identity could not be serialized.
    Identity(String),
}

impl fmt::Display for ContextualEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(error) => error.fmt(formatter),
            Self::Evaluation(error) => error.fmt(formatter),
            Self::Identity(error) => write!(
                formatter,
                "contextual identity serialization failed: {error}"
            ),
        }
    }
}

impl std::error::Error for ContextualEvaluationError {}

/// Evaluation failure that never contains a fallback Boolean verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvaluationError {
    /// The API was invoked with a different profile than it implements.
    UnsupportedProfile {
        /// Required wire identity.
        expected: &'static str,
        /// Actual wire identity.
        actual: &'static str,
    },
    /// An instant contains duplicate or descending proposition identities.
    TraceNotStrictlyOrdered {
        /// Instant index.
        instant: usize,
        /// Previous proposition.
        previous: PropositionId,
        /// Rejected proposition.
        current: PropositionId,
    },
    /// Checked time arithmetic failed.
    TimeOverflow,
    /// A single interval exceeded the configured expansion limit.
    TemporalSpanExceeded {
        /// Requested interval cardinality.
        requested: u64,
        /// Configured limit.
        limit: u64,
    },
    /// Recursive work exceeded the configured budget.
    WorkLimitExceeded {
        /// Configured limit.
        limit: u64,
    },
    /// Formula nesting exceeded the process-safe recursion boundary.
    RecursionDepthExceeded {
        /// Fixed nesting boundary.
        limit: u32,
    },
    /// A horizon calculation failed.
    Horizon(HorizonError),
    /// A validated formula exposed an impossible node reference.
    InvalidNodeReference(NodeId),
}

impl fmt::Display for EvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { expected, actual } => {
                write!(
                    formatter,
                    "expected semantic profile {expected}, found {actual}"
                )
            }
            Self::TraceNotStrictlyOrdered {
                instant,
                previous,
                current,
            } => write!(
                formatter,
                "trace instant {instant} proposition {} does not follow {}",
                current.0, previous.0
            ),
            Self::TimeOverflow => write!(formatter, "discrete-time arithmetic overflow"),
            Self::TemporalSpanExceeded { requested, limit } => write!(
                formatter,
                "temporal span {requested} exceeds configured limit {limit}"
            ),
            Self::WorkLimitExceeded { limit } => {
                write!(formatter, "evaluation exceeded work limit {limit}")
            }
            Self::RecursionDepthExceeded { limit } => {
                write!(
                    formatter,
                    "evaluation exceeded recursion-depth limit {limit}"
                )
            }
            Self::Horizon(error) => error.fmt(formatter),
            Self::InvalidNodeReference(node) => {
                write!(
                    formatter,
                    "invalid validated-formula node reference {}",
                    node.0
                )
            }
        }
    }
}

impl std::error::Error for EvaluationError {}

impl From<HorizonError> for EvaluationError {
    fn from(value: HorizonError) -> Self {
        Self::Horizon(value)
    }
}

struct Evaluator<'formula, 'trace> {
    formula: Formula<'formula>,
    trace: &'trace [Vec<PropositionId>],
    closed: bool,
    limits: EvaluationLimits,
    evaluations: u64,
}

impl Evaluator<'_, '_> {
    fn consume(&mut self) -> Result<(), EvaluationError> {
        self.evaluations =
            self.evaluations
                .checked_add(1)
                .ok_or(EvaluationError::WorkLimitExceeded {
                    limit: self.limits.max_node_evaluations,
                })?;
        if self.evaluations > self.limits.max_node_evaluations {
            return Err(EvaluationError::WorkLimitExceeded {
                limit: self.limits.max_node_evaluations,
            });
        }
        Ok(())
    }

    fn node(&self, node: NodeId) -> Result<NodeKind, EvaluationError> {
        self.formula
            .nodes()
            .get(node.0 as usize)
            .map(|value| value.kind)
            .ok_or(EvaluationError::InvalidNodeReference(node))
    }

    fn temporal_endpoints(&self, start: u32, end: u32) -> Result<(u64, u64), EvaluationError> {
        let cardinality = u64::from(end) - u64::from(start) + 1;
        if cardinality > self.limits.max_temporal_span {
            return Err(EvaluationError::TemporalSpanExceeded {
                requested: cardinality,
                limit: self.limits.max_temporal_span,
            });
        }
        Ok((u64::from(start), u64::from(end)))
    }

    fn at(&mut self, node: NodeId, time: u64, depth: u32) -> Result<TruthValue, EvaluationError> {
        if depth > self.limits.max_recursion_depth {
            return Err(EvaluationError::RecursionDepthExceeded {
                limit: self.limits.max_recursion_depth,
            });
        }
        self.consume()?;
        let child_depth = depth
            .checked_add(1)
            .ok_or(EvaluationError::RecursionDepthExceeded {
                limit: self.limits.max_recursion_depth,
            })?;
        match self.node(node)? {
            NodeKind::False => Ok(TruthValue::False),
            NodeKind::True => Ok(TruthValue::True),
            NodeKind::Proposition { proposition } => self.proposition(time, proposition),
            NodeKind::Not { operand } => Ok(self.at(operand, time, child_depth)?.not()),
            NodeKind::And { left, right } => {
                let left = self.at(left, time, child_depth)?;
                if left == TruthValue::False {
                    return Ok(left);
                }
                Ok(left.and(self.at(right, time, child_depth)?))
            }
            NodeKind::Or { left, right } => {
                let left = self.at(left, time, child_depth)?;
                if left == TruthValue::True {
                    return Ok(left);
                }
                Ok(left.or(self.at(right, time, child_depth)?))
            }
            NodeKind::Implies { left, right } => {
                let left = self.at(left, time, child_depth)?.not();
                if left == TruthValue::True {
                    return Ok(left);
                }
                Ok(left.or(self.at(right, time, child_depth)?))
            }
            NodeKind::Equivalent { left, right } => {
                let left = self.at(left, time, child_depth)?;
                let right = self.at(right, time, child_depth)?;
                Ok(left.and(right).or(left.not().and(right.not())))
            }
            NodeKind::Future { interval, operand } => {
                let (start, end) = self.temporal_endpoints(interval.start(), interval.end())?;
                let mut result = TruthValue::False;
                for offset in start..=end {
                    let at = time
                        .checked_add(offset)
                        .ok_or(EvaluationError::TimeOverflow)?;
                    result = result.or(self.at(operand, at, child_depth)?);
                    if result == TruthValue::True {
                        break;
                    }
                }
                Ok(result)
            }
            NodeKind::Globally { interval, operand } => {
                let (start, end) = self.temporal_endpoints(interval.start(), interval.end())?;
                let mut result = TruthValue::True;
                for offset in start..=end {
                    let at = time
                        .checked_add(offset)
                        .ok_or(EvaluationError::TimeOverflow)?;
                    result = result.and(self.at(operand, at, child_depth)?);
                    if result == TruthValue::False {
                        break;
                    }
                }
                Ok(result)
            }
            NodeKind::Until {
                interval,
                left,
                right,
            } => self.until(
                time,
                interval.start(),
                interval.end(),
                left,
                right,
                false,
                child_depth,
            ),
            NodeKind::Release {
                interval,
                left,
                right,
            } => Ok(self
                .until(
                    time,
                    interval.start(),
                    interval.end(),
                    left,
                    right,
                    true,
                    child_depth,
                )?
                .not()),
        }
    }

    fn proposition(
        &self,
        time: u64,
        proposition: PropositionId,
    ) -> Result<TruthValue, EvaluationError> {
        let Some(instant) = usize::try_from(time)
            .ok()
            .and_then(|index| self.trace.get(index))
        else {
            return Ok(if self.closed {
                TruthValue::False
            } else {
                TruthValue::Pending
            });
        };
        Ok(if instant.binary_search(&proposition).is_ok() {
            TruthValue::True
        } else {
            TruthValue::False
        })
    }

    fn until(
        &mut self,
        time: u64,
        start: u32,
        end: u32,
        left: NodeId,
        right: NodeId,
        negate_operands: bool,
        child_depth: u32,
    ) -> Result<TruthValue, EvaluationError> {
        let (start, end) = self.temporal_endpoints(start, end)?;
        let mut result = TruthValue::False;
        for witness in start..=end {
            let witness_time = time
                .checked_add(witness)
                .ok_or(EvaluationError::TimeOverflow)?;
            let mut candidate = self.at(right, witness_time, child_depth)?;
            if negate_operands {
                candidate = candidate.not();
            }
            for offset in start..witness {
                if candidate == TruthValue::False {
                    break;
                }
                let at = time
                    .checked_add(offset)
                    .ok_or(EvaluationError::TimeOverflow)?;
                let mut value = self.at(left, at, child_depth)?;
                if negate_operands {
                    value = value.not();
                }
                candidate = candidate.and(value);
            }
            result = result.or(candidate);
            if result == TruthValue::True {
                break;
            }
        }
        Ok(result)
    }
}

fn validate_trace(trace: &[Vec<PropositionId>]) -> Result<(), EvaluationError> {
    for (instant_index, instant) in trace.iter().enumerate() {
        for pair in instant.windows(2) {
            if pair[0] >= pair[1] {
                return Err(EvaluationError::TraceNotStrictlyOrdered {
                    instant: instant_index,
                    previous: pair[0],
                    current: pair[1],
                });
            }
        }
    }
    Ok(())
}

fn referenced_propositions(formula: Formula<'_>) -> Vec<u32> {
    formula
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            NodeKind::Proposition { proposition } => Some(proposition.0),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn evaluate(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    trace: &[Vec<PropositionId>],
    trace_id: impl Into<String>,
    closed: bool,
    verdict_time: u64,
    limits: EvaluationLimits,
) -> Result<EvaluationReport, EvaluationError> {
    validate_trace(trace)?;
    let horizon = lookahead(formula)?;
    let mut evaluator = Evaluator {
        formula,
        trace,
        closed,
        limits,
        evaluations: 0,
    };
    let verdict = evaluator.at(formula.root(), verdict_time, 0)?;
    Ok(EvaluationReport {
        schema_version: "tl-mltl.evaluation/v1".to_owned(),
        formula_id: formula_id.into(),
        formula_root: formula.root().0,
        semantic_profile: formula.profile().as_str().to_owned(),
        trace_id: trace_id.into(),
        trace_length: trace.len() as u64,
        trace_closed: closed,
        verdict,
        verdict_time,
        observed_through: trace
            .len()
            .checked_sub(1)
            .and_then(|index| u64::try_from(index).ok()),
        horizon,
        proposition_ids: referenced_propositions(formula),
    })
}

/// Evaluates a complete trace under `mltl.closed-trace/v1`.
///
/// Implements: FR-001
pub fn evaluate_closed(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    trace: &[Vec<PropositionId>],
    trace_id: impl Into<String>,
    limits: EvaluationLimits,
) -> Result<EvaluationReport, EvaluationError> {
    evaluate_closed_at(formula, formula_id, trace, trace_id, 0, limits)
}

/// Evaluates a complete trace after binding every formula proposition through
/// the exact caller-supplied shared catalog.
pub fn evaluate_closed_with_context(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    trace: &[Vec<PropositionId>],
    trace_id: impl Into<String>,
    limits: EvaluationLimits,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<ContextualEvaluationReport, ContextualEvaluationError> {
    bind_formula(formula, signal_catalog).map_err(ContextualEvaluationError::Binding)?;
    let formula_id = formula_id.into();
    let trace_id = trace_id.into();
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Request<'a> {
        formula_id: &'a str,
        formula_root: u32,
        semantic_profile: &'a str,
        formula_nodes: &'a [tl_syntax::Node],
        trace: &'a [Vec<PropositionId>],
        trace_id: &'a str,
        limits: [u64; 3],
        source_revision: &'a str,
        syntax_revision: &'a str,
    }
    let request = Request {
        formula_id: &formula_id,
        formula_root: formula.root().0,
        semantic_profile: formula.profile().as_str(),
        formula_nodes: formula.nodes(),
        trace,
        trace_id: &trace_id,
        limits: [
            limits.max_node_evaluations,
            limits.max_temporal_span,
            u64::from(limits.max_recursion_depth),
        ],
        source_revision: env!("TL_MLTL_SOURCE_REVISION"),
        syntax_revision: TL_SYNTAX_REVISION,
    };
    let request_sha256 = contextual_request_sha256(
        "tl-mltl.contextual-evaluation/v2/request",
        &request,
        signal_catalog,
        requirement_context,
    )
    .map_err(|error| ContextualEvaluationError::Identity(error.to_string()))?;
    let report = evaluate_closed(formula, formula_id.clone(), trace, trace_id.clone(), limits)
        .map_err(ContextualEvaluationError::Evaluation)?;
    let mut contextual = ContextualEvaluationReport {
        schema_version: ContextualEvaluationSchemaVersion::V2,
        source_revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        signal_catalog_sha256: catalog_sha256(signal_catalog)
            .map_err(|error| ContextualEvaluationError::Identity(error.to_string()))?,
        requirement_context: requirement_context.cloned(),
        request_sha256,
        result_sha256: String::new(),
        formula_id: report.formula_id,
        formula_root: report.formula_root,
        semantic_profile: report.semantic_profile,
        trace_id: report.trace_id,
        trace_length: report.trace_length,
        trace_closed: report.trace_closed,
        verdict: report.verdict,
        verdict_time: report.verdict_time,
        observed_through: report.observed_through,
        horizon: report.horizon,
        proposition_ids: report.proposition_ids,
    };
    contextual.result_sha256 =
        contextual_result_sha256("tl-mltl.contextual-evaluation/v2/result", &contextual)
            .map_err(|error| ContextualEvaluationError::Identity(error.to_string()))?;
    Ok(contextual)
}

/// Evaluates a complete trace at `verdict_time` under `mltl.closed-trace/v1`.
pub fn evaluate_closed_at(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    trace: &[Vec<PropositionId>],
    trace_id: impl Into<String>,
    verdict_time: u64,
    limits: EvaluationLimits,
) -> Result<EvaluationReport, EvaluationError> {
    if formula.profile() != SemanticProfile::ClosedTraceV1 {
        return Err(EvaluationError::UnsupportedProfile {
            expected: SemanticProfile::ClosedTraceV1.as_str(),
            actual: formula.profile().as_str(),
        });
    }
    evaluate(
        formula,
        formula_id,
        trace,
        trace_id,
        true,
        verdict_time,
        limits,
    )
}

/// Evaluates an open or explicitly closed prefix under `mltl.online-prefix/v1`.
///
/// Implements: FR-003
pub fn evaluate_prefix(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    trace: &[Vec<PropositionId>],
    trace_id: impl Into<String>,
    closed: bool,
    limits: EvaluationLimits,
) -> Result<EvaluationReport, EvaluationError> {
    evaluate_prefix_at(formula, formula_id, trace, trace_id, closed, 0, limits)
}

/// Evaluates a prefix after binding every formula proposition through the exact
/// caller-supplied shared catalog.
pub fn evaluate_prefix_with_context(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    trace: &[Vec<PropositionId>],
    trace_id: impl Into<String>,
    closed: bool,
    limits: EvaluationLimits,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<ContextualEvaluationReport, ContextualEvaluationError> {
    bind_formula(formula, signal_catalog).map_err(ContextualEvaluationError::Binding)?;
    let formula_id = formula_id.into();
    let trace_id = trace_id.into();
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Request<'a> {
        formula_id: &'a str,
        formula_root: u32,
        semantic_profile: &'a str,
        formula_nodes: &'a [tl_syntax::Node],
        trace: &'a [Vec<PropositionId>],
        trace_id: &'a str,
        closed: bool,
        limits: [u64; 3],
        source_revision: &'a str,
        syntax_revision: &'a str,
    }
    let request = Request {
        formula_id: &formula_id,
        formula_root: formula.root().0,
        semantic_profile: formula.profile().as_str(),
        formula_nodes: formula.nodes(),
        trace,
        trace_id: &trace_id,
        closed,
        limits: [
            limits.max_node_evaluations,
            limits.max_temporal_span,
            u64::from(limits.max_recursion_depth),
        ],
        source_revision: env!("TL_MLTL_SOURCE_REVISION"),
        syntax_revision: TL_SYNTAX_REVISION,
    };
    let request_sha256 = contextual_request_sha256(
        "tl-mltl.contextual-prefix-evaluation/v2/request",
        &request,
        signal_catalog,
        requirement_context,
    )
    .map_err(|error| ContextualEvaluationError::Identity(error.to_string()))?;
    let report = evaluate_prefix(
        formula,
        formula_id.clone(),
        trace,
        trace_id.clone(),
        closed,
        limits,
    )
    .map_err(ContextualEvaluationError::Evaluation)?;
    let mut contextual = ContextualEvaluationReport {
        schema_version: ContextualEvaluationSchemaVersion::V2,
        source_revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        signal_catalog_sha256: catalog_sha256(signal_catalog)
            .map_err(|error| ContextualEvaluationError::Identity(error.to_string()))?,
        requirement_context: requirement_context.cloned(),
        request_sha256,
        result_sha256: String::new(),
        formula_id: report.formula_id,
        formula_root: report.formula_root,
        semantic_profile: report.semantic_profile,
        trace_id: report.trace_id,
        trace_length: report.trace_length,
        trace_closed: report.trace_closed,
        verdict: report.verdict,
        verdict_time: report.verdict_time,
        observed_through: report.observed_through,
        horizon: report.horizon,
        proposition_ids: report.proposition_ids,
    };
    contextual.result_sha256 = contextual_result_sha256(
        "tl-mltl.contextual-prefix-evaluation/v2/result",
        &contextual,
    )
    .map_err(|error| ContextualEvaluationError::Identity(error.to_string()))?;
    Ok(contextual)
}

/// Evaluates an open or closed prefix at `verdict_time`.
pub fn evaluate_prefix_at(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    trace: &[Vec<PropositionId>],
    trace_id: impl Into<String>,
    closed: bool,
    verdict_time: u64,
    limits: EvaluationLimits,
) -> Result<EvaluationReport, EvaluationError> {
    if formula.profile() != SemanticProfile::OnlinePrefixV1 {
        return Err(EvaluationError::UnsupportedProfile {
            expected: SemanticProfile::OnlinePrefixV1.as_str(),
            actual: formula.profile().as_str(),
        });
    }
    evaluate(
        formula,
        formula_id,
        trace,
        trace_id,
        closed,
        verdict_time,
        limits,
    )
}

#[cfg(test)]
mod tests {
    use tl_syntax::{
        FormulaDocument, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
        RequirementContextDocument, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
        SourceSpan,
    };

    use super::{
        evaluate_closed_with_context, evaluate_prefix_with_context,
        ContextualEvaluationSchemaVersion, EvaluationLimits,
    };

    fn formula() -> FormulaDocument {
        FormulaDocument::new(
            SemanticProfile::ClosedTraceV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition {
                proposition: tl_syntax::PropositionId(7),
            })],
        )
        .unwrap()
    }

    fn catalog() -> SignalCatalogDocument {
        SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(1),
                "request_ready".to_owned(),
                SignalDomain::Boolean,
            )],
            vec![PropositionBinding::new(
                tl_syntax::PropositionId(7),
                SignalId(1),
            )],
        )
        .unwrap()
    }

    fn context(revision: &str) -> RequirementContextDocument {
        RequirementContextDocument::new(
            "agent-ix/tl-mltl/FR-007".to_owned(),
            revision.to_owned(),
            "AC-1".to_owned(),
            "contextual-evaluation".to_owned(),
            SourceSpan::new(0, 1).unwrap(),
        )
        .unwrap()
    }

    // Trace: TC-025, TC-028, FR-007-AC-1, FR-007-AC-4
    #[test]
    fn contextual_evaluation_preserves_shared_context_and_binds_its_identity() {
        let document = formula();
        let first = evaluate_closed_with_context(
            document.validate().unwrap(),
            "formula",
            &[vec![tl_syntax::PropositionId(7)]],
            "trace",
            EvaluationLimits::default(),
            &catalog(),
            Some(&context("1")),
        )
        .unwrap();
        let second = evaluate_closed_with_context(
            document.validate().unwrap(),
            "formula",
            &[vec![tl_syntax::PropositionId(7)]],
            "trace",
            EvaluationLimits::default(),
            &catalog(),
            Some(&context("2")),
        )
        .unwrap();
        assert_eq!(first.schema_version, ContextualEvaluationSchemaVersion::V2);
        assert_eq!(first.requirement_context, Some(context("1")));
        assert_ne!(first.request_sha256, second.request_sha256);
        assert_ne!(first.result_sha256, second.result_sha256);
    }

    // Trace: TC-029, FR-007-AC-5
    #[test]
    fn contextual_evaluation_wire_is_closed_and_requires_contextual_fields() {
        let document = formula();
        let report = evaluate_closed_with_context(
            document.validate().unwrap(),
            "formula",
            &[vec![tl_syntax::PropositionId(7)]],
            "trace",
            EvaluationLimits::default(),
            &catalog(),
            None,
        )
        .unwrap();
        let mut value = serde_json::to_value(report).unwrap();
        value.as_object_mut().unwrap().remove("signalCatalogSha256");
        assert!(serde_json::from_value::<super::ContextualEvaluationReport>(value).is_err());
    }

    // Trace: TC-025, TC-028, FR-007-AC-1, FR-007-AC-4
    #[test]
    fn contextual_prefix_binds_closedness_and_retains_explicit_absence() {
        let document = FormulaDocument::new(
            SemanticProfile::OnlinePrefixV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition {
                proposition: tl_syntax::PropositionId(7),
            })],
        )
        .unwrap();
        let open = evaluate_prefix_with_context(
            document.validate().unwrap(),
            "formula",
            &[],
            "trace",
            false,
            EvaluationLimits::default(),
            &catalog(),
            None,
        )
        .unwrap();
        let closed = evaluate_prefix_with_context(
            document.validate().unwrap(),
            "formula",
            &[],
            "trace",
            true,
            EvaluationLimits::default(),
            &catalog(),
            None,
        )
        .unwrap();
        assert_eq!(open.requirement_context, None);
        assert_ne!(open.request_sha256, closed.request_sha256);
    }
}
