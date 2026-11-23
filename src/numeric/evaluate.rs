//! Exact evaluation of numeric predicates by reduction to the existing
//! closed, prefix and past evaluators (FR-054, FR-055).
//!
//! No floating-point type appears in this module (NFR-017).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use core::fmt;
use std::collections::BTreeSet;

use serde::Serialize;
use tl_syntax::{Formula, NodeKind, PropositionId, SemanticProfile, SignalId};

use super::model::{ExprNode, PredicateTable};
use super::trace::NumericTraceDocument;
use crate::future::evaluate_prefix_partial_at;
use crate::{
    evaluate_closed_at, evaluate_past, ClockBinding, EvaluationError, EvaluationLimits,
    EvaluationReport, HistoryError, PastEvaluationError, PastEvaluationLimits,
    PastEvaluationRelationInput, PastEvaluationReport, PositionHistoryDocument,
    PositionObservation,
};

/// Wrapper identity of the numeric evaluation report.
pub const NUMERIC_EVALUATION_V1: &str = "tl-mltl.numeric-evaluation/v1";

/// Largest scale the expression model admits.
const MAX_SCALE: u8 = 18;

/// Limits of a numeric evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NumericEvaluationLimits {
    /// Limits of the closed and prefix evaluators.
    pub evaluation: EvaluationLimits,
    /// Limits of the past evaluator.
    pub past: PastEvaluationLimits,
    /// Ceiling on `positions x predicates x expression nodes`, checked first.
    pub max_predicate_work: u64,
}

impl Default for NumericEvaluationLimits {
    fn default() -> Self {
        Self {
            evaluation: EvaluationLimits::default(),
            past: PastEvaluationLimits::default(),
            max_predicate_work: 100_000_000,
        }
    }
}

/// Truth of one predicate at one observed position.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PredicateCell {
    /// The comparison holds.
    True,
    /// The comparison does not hold.
    False,
    /// A signal the predicate reads is absent at this position.
    Unobserved,
}

/// The existing report this wrapper embeds unchanged.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(tag = "kind", content = "report", rename_all = "snake_case")]
pub enum EmbeddedReport {
    /// Closed or prefix report (`tl-mltl.evaluation/v1`).
    Future(EvaluationReport),
    /// Origin-complete report.
    Past(PastEvaluationReport),
}

/// Numeric section of the wrapper.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericSection {
    /// Predicate-defined proposition identities, in table order.
    pub predicate_proposition_ids: Vec<u32>,
    /// Per observed position, one cell per predicate in table order.
    pub truth: Vec<Vec<PredicateCell>>,
    /// Number of unobserved cells inside the observed positions.
    pub unobserved_cells: u64,
    /// Derived true-set per observed position (Boolean and predicate atoms).
    pub derived_trace: Vec<Vec<u32>>,
    /// Derived unobserved predicate atoms per position (prefix profile only;
    /// empty rows on the other profiles).
    pub derived_unobserved: Vec<Vec<u32>>,
    /// How unobserved cells were resolved under the profile.
    pub note: &'static str,
}

/// `tl-mltl.numeric-evaluation/v1`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NumericEvaluationReport {
    /// Wire identity.
    pub schema_version: &'static str,
    /// Existing report, byte-identical to a direct call on the derived trace.
    pub evaluation: EmbeddedReport,
    /// Numeric section.
    pub numeric: NumericSection,
}

/// Typed failure; never a verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NumericEvaluationError {
    /// The profile has no numeric rule (infinite-trace, LTLf).
    UnsupportedProfile {
        /// Profile identity.
        actual: &'static str,
    },
    /// An `i128` operation overflowed (a table not validated by tl-syntax).
    ArithmeticOverflow {
        /// Expression node index.
        node: u32,
    },
    /// Operands of `add`, `sub` or a comparison differ in scale, or a scale exceeds 18.
    ScaleMismatch {
        /// Expression node index (or comparison left root).
        node: u32,
    },
    /// `rescale` narrows.
    RescaleNarrowing {
        /// Expression node index.
        node: u32,
    },
    /// An operand index does not precede its user.
    MalformedExpression {
        /// Expression node index.
        node: u32,
    },
    /// A predicate proposition is also true in the Boolean `propositions` set.
    PropositionDefinedTwice {
        /// Proposition.
        proposition: PropositionId,
    },
    /// Origin-complete profile: a predicate read is unobserved inside the history.
    IncompleteHistory {
        /// Position.
        position: u64,
        /// Signal.
        signal: SignalId,
    },
    /// `positions x predicates x expression nodes` exceeds the configured ceiling.
    WorkLimitExceeded {
        /// Requested work.
        requested: u128,
        /// Limit.
        limit: u64,
    },
    /// Closed or prefix evaluation failed.
    Evaluation(EvaluationError),
    /// History construction failed.
    History(HistoryError),
    /// Origin-complete evaluation of a trace with no instant.
    EmptyHistory,
    /// Past evaluation failed.
    Past(PastEvaluationError),
}

impl fmt::Display for NumericEvaluationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { actual } => {
                write!(formatter, "numeric evaluation does not support {actual}")
            }
            Self::ArithmeticOverflow { node } => {
                write!(formatter, "exact arithmetic overflow at expression node {node}")
            }
            Self::ScaleMismatch { node } => write!(formatter, "scale mismatch at node {node}"),
            Self::RescaleNarrowing { node } => {
                write!(formatter, "rescale narrows at node {node}")
            }
            Self::MalformedExpression { node } => {
                write!(formatter, "malformed expression at node {node}")
            }
            Self::PropositionDefinedTwice { proposition } => write!(
                formatter,
                "proposition {} is both Boolean-observed and predicate-defined",
                proposition.0
            ),
            Self::IncompleteHistory { position, signal } => write!(
                formatter,
                "signal {} unobserved at history position {position}",
                signal.0
            ),
            Self::WorkLimitExceeded { requested, limit } => {
                write!(formatter, "predicate work {requested} exceeds limit {limit}")
            }
            Self::Evaluation(error) => error.fmt(formatter),
            Self::History(error) => write!(formatter, "history refused: {error:?}"),
            Self::EmptyHistory => write!(formatter, "an empty trace has no history"),
            Self::Past(error) => write!(formatter, "past evaluation failed: {error:?}"),
        }
    }
}

impl std::error::Error for NumericEvaluationError {}

/// Exact value: coefficient and scale.
type Exact = (i128, u8);

fn pow10(exponent: u8, node: u32) -> Result<i128, NumericEvaluationError> {
    10i128
        .checked_pow(u32::from(exponent))
        .ok_or(NumericEvaluationError::ArithmeticOverflow { node })
}

fn observed(trace: &NumericTraceDocument, position: usize, signal: SignalId) -> Option<Exact> {
    let instant = trace.instants.get(position)?;
    let index = instant
        .values
        .binary_search_by_key(&signal.0, |observation| observation.signal)
        .ok()?;
    let (coefficient, scale) = instant.values[index].exact()?;
    Some((i128::from(coefficient), scale))
}

/// Value of one node from its already-computed operands (`None` = unobserved).
fn node_value(
    kind: ExprNode,
    node: u32,
    values: &[Option<Exact>],
    position: usize,
    trace: &NumericTraceDocument,
) -> Result<Option<Exact>, NumericEvaluationError> {
    let operand = |index: u32| -> Result<Option<Exact>, NumericEvaluationError> {
        // Operands precede their user, so a later or equal index is malformed.
        if index >= node {
            return Err(NumericEvaluationError::MalformedExpression { node });
        }
        Ok(values[index as usize])
    };
    let overflow = || NumericEvaluationError::ArithmeticOverflow { node };
    let value = match kind {
        ExprNode::Signal { signal } => observed(trace, position, signal),
        ExprNode::Integer { value } => Some((i128::from(value), 0)),
        ExprNode::Decimal { coefficient, scale } => {
            if scale > MAX_SCALE {
                return Err(NumericEvaluationError::ScaleMismatch { node });
            }
            Some((i128::from(coefficient), scale))
        }
        ExprNode::Neg { operand: index } => match operand(index)? {
            Some((value, scale)) => Some((value.checked_neg().ok_or_else(overflow)?, scale)),
            None => None,
        },
        ExprNode::Abs { operand: index } => match operand(index)? {
            Some((value, scale)) => Some((value.checked_abs().ok_or_else(overflow)?, scale)),
            None => None,
        },
        ExprNode::Add { left, right } | ExprNode::Sub { left, right } => {
            match (operand(left)?, operand(right)?) {
                (Some((l, ls)), Some((r, rs))) => {
                    if ls != rs {
                        return Err(NumericEvaluationError::ScaleMismatch { node });
                    }
                    let value = if matches!(kind, ExprNode::Add { .. }) {
                        l.checked_add(r)
                    } else {
                        l.checked_sub(r)
                    };
                    Some((value.ok_or_else(overflow)?, ls))
                }
                _ => None,
            }
        }
        ExprNode::Mul { left, right } => match (operand(left)?, operand(right)?) {
            (Some((l, ls)), Some((r, rs))) => {
                let scale = ls.checked_add(rs).filter(|scale| *scale <= MAX_SCALE);
                let scale = scale.ok_or(NumericEvaluationError::ScaleMismatch { node })?;
                Some((l.checked_mul(r).ok_or_else(overflow)?, scale))
            }
            _ => None,
        },
        ExprNode::Rescale {
            operand: index,
            scale,
        } => match operand(index)? {
            Some((value, from)) => {
                if scale < from {
                    return Err(NumericEvaluationError::RescaleNarrowing { node });
                }
                if scale > MAX_SCALE {
                    return Err(NumericEvaluationError::ScaleMismatch { node });
                }
                let factor = pow10(scale - from, node)?;
                Some((value.checked_mul(factor).ok_or_else(overflow)?, scale))
            }
            None => None,
        },
    };
    Ok(value)
}

/// Every arena node's value at `position`, bottom-up, each node once: shared
/// operands are never re-evaluated (cost `nodes`, not exponential).
pub(crate) fn arena_values(
    table: &PredicateTable,
    position: usize,
    trace: &NumericTraceDocument,
) -> Result<Vec<Option<Exact>>, NumericEvaluationError> {
    let mut values = Vec::with_capacity(table.expressions.len());
    for (index, kind) in table.expressions.iter().enumerate() {
        let value = node_value(*kind, index as u32, &values, position, trace)?;
        values.push(value);
    }
    Ok(values)
}

/// Truth of every predicate at every observed position.
pub(crate) fn predicate_cells(
    table: &PredicateTable,
    trace: &NumericTraceDocument,
) -> Result<Vec<Vec<PredicateCell>>, NumericEvaluationError> {
    let mut rows = Vec::with_capacity(trace.instants.len());
    for position in 0..trace.instants.len() {
        let values = arena_values(table, position, trace)?;
        let mut row = Vec::with_capacity(table.predicates.len());
        for predicate in &table.predicates {
            let root = |index: u32| -> Result<Option<Exact>, NumericEvaluationError> {
                values
                    .get(index as usize)
                    .copied()
                    .ok_or(NumericEvaluationError::MalformedExpression { node: index })
            };
            let cell = match (root(predicate.left)?, root(predicate.right)?) {
                (Some((l, ls)), Some((r, rs))) => {
                    if ls != rs {
                        return Err(NumericEvaluationError::ScaleMismatch {
                            node: predicate.left,
                        });
                    }
                    if predicate.comparison.holds(l, r) {
                        PredicateCell::True
                    } else {
                        PredicateCell::False
                    }
                }
                _ => PredicateCell::Unobserved,
            };
            row.push(cell);
        }
        rows.push(row);
    }
    Ok(rows)
}

fn check_work(
    table: &PredicateTable,
    trace: &NumericTraceDocument,
    limit: u64,
) -> Result<(), NumericEvaluationError> {
    let requested = (trace.instants.len() as u128)
        .saturating_mul(table.expressions.len().max(1) as u128)
        .saturating_add(
            (trace.instants.len() as u128).saturating_mul(table.predicates.len() as u128),
        );
    if requested > u128::from(limit) {
        return Err(NumericEvaluationError::WorkLimitExceeded { requested, limit });
    }
    Ok(())
}

/// First signal a predicate reads that is absent at `position`.
fn first_unobserved_signal(
    table: &PredicateTable,
    predicate: usize,
    position: usize,
    trace: &NumericTraceDocument,
) -> SignalId {
    let definition = table.predicates[predicate];
    let mut signals = table.signals_read(definition.left);
    signals.extend(table.signals_read(definition.right));
    signals.sort_unstable();
    signals
        .into_iter()
        .find(|signal| observed(trace, position, *signal).is_none())
        .unwrap_or(SignalId(u32::MAX))
}

/// Predicate truth and the derived Boolean (and unobserved) rows of one trace,
/// computed once and shared by every evaluation time (FR-054, FR-055).
pub(crate) struct Prepared {
    profile: SemanticProfile,
    predicate_ids: Vec<PropositionId>,
    cells: Vec<Vec<PredicateCell>>,
    derived_trace: Vec<Vec<PropositionId>>,
    derived_unobserved: Vec<Vec<PropositionId>>,
    unobserved_cells: u64,
    /// First position holding an unobserved cell, with its predicate index.
    first_gap: Option<(usize, usize)>,
    /// Origin-complete history over the derived trace, built once.
    history: Option<PositionHistoryDocument>,
}

/// Prepares a trace for evaluation under `profile`.
pub(crate) fn prepare(
    profile: SemanticProfile,
    table: &PredicateTable,
    trace: &NumericTraceDocument,
    limits: NumericEvaluationLimits,
) -> Result<Prepared, NumericEvaluationError> {
    if !matches!(
        profile,
        SemanticProfile::ClosedTraceV1
            | SemanticProfile::OnlinePrefixV1
            | SemanticProfile::OriginCompleteHistoryV1
    ) {
        return Err(NumericEvaluationError::UnsupportedProfile {
            actual: profile.as_str(),
        });
    }
    check_work(table, trace, limits.max_predicate_work)?;
    let predicate_ids = table.proposition_ids();
    for instant in &trace.instants {
        if let Some(proposition) = instant
            .propositions
            .iter()
            .find(|proposition| predicate_ids.contains(proposition))
        {
            return Err(NumericEvaluationError::PropositionDefinedTwice {
                proposition: *proposition,
            });
        }
    }
    let cells = predicate_cells(table, trace)?;
    let mut derived_trace = Vec::with_capacity(cells.len());
    let mut derived_unobserved = Vec::with_capacity(cells.len());
    let mut unobserved_cells = 0u64;
    let mut first_gap = None;
    for (position, row) in cells.iter().enumerate() {
        let mut truth: BTreeSet<PropositionId> =
            trace.instants[position].propositions.iter().copied().collect();
        let mut unknown = Vec::new();
        for (index, cell) in row.iter().enumerate() {
            match cell {
                PredicateCell::True => {
                    truth.insert(predicate_ids[index]);
                }
                PredicateCell::False => {}
                PredicateCell::Unobserved => {
                    unobserved_cells += 1;
                    if first_gap.is_none() {
                        first_gap = Some((position, index));
                    }
                    if profile == SemanticProfile::OnlinePrefixV1 {
                        unknown.push(predicate_ids[index]);
                    }
                }
            }
        }
        unknown.sort_unstable();
        derived_trace.push(truth.into_iter().collect::<Vec<_>>());
        derived_unobserved.push(unknown);
    }
    let history = if profile == SemanticProfile::OriginCompleteHistoryV1 && !derived_trace.is_empty()
    {
        let observations = derived_trace
            .iter()
            .enumerate()
            .map(|(position, row)| PositionObservation::new(position as u64, row.clone(), None))
            .collect();
        Some(
            PositionHistoryDocument::new(
                trace.trace_id.clone(),
                1,
                0,
                derived_trace.len().saturating_sub(1) as u64,
                Some(ClockBinding::EventPosition),
                observations,
            )
            .map_err(NumericEvaluationError::History)?,
        )
    } else {
        None
    };
    Ok(Prepared {
        profile,
        predicate_ids,
        cells,
        derived_trace,
        derived_unobserved,
        unobserved_cells,
        first_gap,
        history,
    })
}

const CLOSED_NOTE: &str =
    "closed-trace: an unobserved predicate is false (padding), so its negation is true";
const PREFIX_NOTE: &str =
    "online-prefix: an unobserved predicate is Pending under the Kleene operators";
const PAST_NOTE: &str = "origin-complete: an unobserved predicate inside the history refuses; \
                         positions before the origin keep the proposition-false extension";

/// Evaluates `formula` (whose profile must be the prepared one) at `time` on
/// prepared rows: one evaluator call, no recomputation of predicate truth.
pub(crate) fn evaluate_prepared(
    prepared: &Prepared,
    formula: Formula<'_>,
    formula_id: &str,
    table: &PredicateTable,
    trace: &NumericTraceDocument,
    time: u64,
    limits: NumericEvaluationLimits,
) -> Result<EmbeddedReport, NumericEvaluationError> {
    if formula.profile() != prepared.profile {
        return Err(NumericEvaluationError::UnsupportedProfile {
            actual: formula.profile().as_str(),
        });
    }
    match prepared.profile {
        SemanticProfile::ClosedTraceV1 => evaluate_closed_at(
            formula,
            formula_id,
            &prepared.derived_trace,
            trace.trace_id.clone(),
            time,
            limits.evaluation,
        )
        .map(EmbeddedReport::Future)
        .map_err(NumericEvaluationError::Evaluation),
        SemanticProfile::OnlinePrefixV1 => evaluate_prefix_partial_at(
            formula,
            formula_id,
            &prepared.derived_trace,
            &prepared.derived_unobserved,
            trace.trace_id.clone(),
            trace.closed,
            time,
            limits.evaluation,
        )
        .map(EmbeddedReport::Future)
        .map_err(NumericEvaluationError::Evaluation),
        _ => {
            // The history runs from origin 0 through the anchor.
            if let Some((position, index)) = prepared.first_gap {
                if position as u64 <= time {
                    return Err(NumericEvaluationError::IncompleteHistory {
                        position: position as u64,
                        signal: first_unobserved_signal(table, index, position, trace),
                    });
                }
            }
            let Some(history) = &prepared.history else {
                return Err(NumericEvaluationError::EmptyHistory);
            };
            evaluate_past(
                formula,
                formula_id,
                history,
                time,
                "numeric-predicates",
                1,
                PastEvaluationRelationInput::Original,
                limits.past,
            )
            .map(EmbeddedReport::Past)
            .map_err(NumericEvaluationError::Past)
        }
    }
}

/// Evaluates a formula whose propositions may be predicate-defined, against a
/// numeric trace, at `time` (verdict time, or the anchor on the past profile).
///
/// Implements: FR-054, FR-055
pub fn evaluate_numeric(
    formula: Formula<'_>,
    formula_id: &str,
    table: &PredicateTable,
    trace: &NumericTraceDocument,
    time: u64,
    limits: NumericEvaluationLimits,
) -> Result<NumericEvaluationReport, NumericEvaluationError> {
    let prepared = prepare(formula.profile(), table, trace, limits)?;
    let evaluation =
        evaluate_prepared(&prepared, formula, formula_id, table, trace, time, limits)?;
    let note = match prepared.profile {
        SemanticProfile::ClosedTraceV1 => CLOSED_NOTE,
        SemanticProfile::OnlinePrefixV1 => PREFIX_NOTE,
        _ => PAST_NOTE,
    };
    let ids = |rows: &[Vec<PropositionId>]| -> Vec<Vec<u32>> {
        rows.iter()
            .map(|row| row.iter().map(|id| id.0).collect())
            .collect()
    };
    Ok(NumericEvaluationReport {
        schema_version: NUMERIC_EVALUATION_V1,
        evaluation,
        numeric: NumericSection {
            predicate_proposition_ids: prepared.predicate_ids.iter().map(|id| id.0).collect(),
            derived_trace: ids(&prepared.derived_trace),
            derived_unobserved: ids(&prepared.derived_unobserved),
            unobserved_cells: prepared.unobserved_cells,
            truth: prepared.cells,
            note,
        },
    })
}


/// Proposition identities referenced by a formula (helper for callers that
/// split Boolean and predicate atoms).
pub fn referenced_propositions(formula: Formula<'_>) -> Vec<PropositionId> {
    formula
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            NodeKind::Proposition { proposition } => Some(proposition),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
