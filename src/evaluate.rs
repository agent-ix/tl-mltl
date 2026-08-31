use core::fmt;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use tl_syntax::{Formula, NodeId, NodeKind, PropositionId, SemanticProfile};

use crate::{analyze_horizon, HorizonError};

const MAX_RECURSION_DEPTH: u32 = 512;

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
}

impl Default for EvaluationLimits {
    fn default() -> Self {
        Self {
            max_node_evaluations: 1_000_000,
            max_temporal_span: 100_000,
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
    /// Formula-time index represented by this verdict (the API currently evaluates time zero).
    pub verdict_time: u64,
    /// Last observation index available when the verdict was produced.
    pub observed_through: u64,
    /// Static worst-case decision horizon.
    pub horizon: u64,
    /// Referenced proposition identities in sorted order.
    pub proposition_ids: Vec<u32>,
}

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
        if depth > MAX_RECURSION_DEPTH {
            return Err(EvaluationError::RecursionDepthExceeded {
                limit: MAX_RECURSION_DEPTH,
            });
        }
        self.consume()?;
        let child_depth = depth
            .checked_add(1)
            .ok_or(EvaluationError::RecursionDepthExceeded {
                limit: MAX_RECURSION_DEPTH,
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
            for offset in 0..witness {
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
    limits: EvaluationLimits,
) -> Result<EvaluationReport, EvaluationError> {
    validate_trace(trace)?;
    let horizon = analyze_horizon(formula, "evaluation")?.lookahead;
    let mut evaluator = Evaluator {
        formula,
        trace,
        closed,
        limits,
        evaluations: 0,
    };
    let verdict = evaluator.at(formula.root(), 0, 0)?;
    Ok(EvaluationReport {
        schema_version: "tl-mltl.evaluation/v1".to_owned(),
        formula_id: formula_id.into(),
        formula_root: formula.root().0,
        semantic_profile: formula.profile().as_str().to_owned(),
        trace_id: trace_id.into(),
        trace_length: trace.len() as u64,
        trace_closed: closed,
        verdict,
        verdict_time: 0,
        observed_through: trace.len().saturating_sub(1) as u64,
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
    if formula.profile() != SemanticProfile::ClosedTraceV1 {
        return Err(EvaluationError::UnsupportedProfile {
            expected: SemanticProfile::ClosedTraceV1.as_str(),
            actual: formula.profile().as_str(),
        });
    }
    evaluate(formula, formula_id, trace, trace_id, true, limits)
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
    if formula.profile() != SemanticProfile::OnlinePrefixV1 {
        return Err(EvaluationError::UnsupportedProfile {
            expected: SemanticProfile::OnlinePrefixV1.as_str(),
            actual: formula.profile().as_str(),
        });
    }
    evaluate(formula, formula_id, trace, trace_id, closed, limits)
}
