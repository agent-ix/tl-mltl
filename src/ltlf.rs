//! LTLf evaluation on nonempty finite traces with the explicit end-of-trace
//! rules of tl-syntax ADR-044 (FR-083), and the LTL3 verdict mapping (FR-085).
//!
//! There is no padding: positions are `0..n-1` and every unbounded operator
//! reads from the current position to the last. Evaluation is bottom-up over
//! the node table and from the last position to the first, so stack depth does
//! not grow with formula depth or trace length; its cost `nodes x n` is charged
//! against the work limit before any node is evaluated.
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).
//! UNBUILT-PIN: the six LTLf node kinds and `SemanticProfile::LtlfFiniteTraceV1`
//! are tl-syntax ADR-044/FR-332/FR-333 (PROPOSED type shape, not merged).

use core::fmt;

use tl_syntax::{Formula, NodeId, NodeKind, PropositionId, SemanticProfile, SignalCatalogDocument};

use crate::context::bind_formula;
use crate::ContextualBindingError;

/// Work ceiling of one LTLf evaluation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LtlfLimits {
    /// Maximum `nodes x trace length` cell evaluations.
    pub max_work: u64,
}

impl Default for LtlfLimits {
    fn default() -> Self {
        Self {
            max_work: 16_000_000,
        }
    }
}

/// Typed LTLf refusal; never a verdict.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LtlfError {
    /// The trace has no position. LTLf traces are nonempty.
    EmptyTrace,
    /// The formula is not under `mltl.ltlf-finite-trace/v1`.
    ProfileMismatch {
        /// Its profile identity.
        actual: &'static str,
    },
    /// A proposition is not bound through the signal catalog.
    UnboundSignal(ContextualBindingError),
    /// `nodes x n` exceeds the work limit.
    WorkLimitExceeded {
        /// The charge.
        charge: u128,
        /// The limit.
        limit: u64,
    },
    /// A trace instant is not strictly increasing.
    TraceNotStrictlyOrdered {
        /// Instant index.
        instant: usize,
    },
    /// A node kind outside the LTLf vocabulary (a bounded or past node).
    NodeNotInProfile {
        /// Offending node.
        node: NodeId,
    },
}

impl fmt::Display for LtlfError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTrace => write!(formatter, "LTLf traces are nonempty"),
            Self::ProfileMismatch { actual } => {
                write!(formatter, "LTLf evaluation requires mltl.ltlf-finite-trace/v1, found {actual}")
            }
            Self::UnboundSignal(error) => error.fmt(formatter),
            Self::WorkLimitExceeded { charge, limit } => {
                write!(formatter, "LTLf charge {charge} exceeds work limit {limit}")
            }
            Self::TraceNotStrictlyOrdered { instant } => {
                write!(formatter, "trace instant {instant} is not strictly increasing")
            }
            Self::NodeNotInProfile { node } => {
                write!(formatter, "node {} is not an LTLf operator", node.0)
            }
        }
    }
}

impl std::error::Error for LtlfError {}

/// The work an evaluation charges: `nodes x n`.
pub fn ltlf_charge(formula: Formula<'_>, trace_length: usize) -> u128 {
    (formula.nodes().len() as u128).saturating_mul(trace_length as u128)
}

/// Evaluates `formula` at position 0 of a nonempty finite trace.
///
/// Implements: FR-083
pub fn evaluate_ltlf(
    formula: Formula<'_>,
    trace: &[Vec<PropositionId>],
    limits: LtlfLimits,
) -> Result<bool, LtlfError> {
    if formula.profile() != SemanticProfile::LtlfFiniteTraceV1 {
        return Err(LtlfError::ProfileMismatch {
            actual: formula.profile().as_str(),
        });
    }
    if trace.is_empty() {
        return Err(LtlfError::EmptyTrace);
    }
    for (instant, row) in trace.iter().enumerate() {
        if row.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(LtlfError::TraceNotStrictlyOrdered { instant });
        }
    }
    let charge = ltlf_charge(formula, trace.len());
    if charge > u128::from(limits.max_work) {
        return Err(LtlfError::WorkLimitExceeded {
            charge,
            limit: limits.max_work,
        });
    }
    let n = trace.len();
    let last = n - 1;
    let mut values: Vec<Vec<bool>> = Vec::with_capacity(formula.nodes().len());
    for (index, node) in formula.nodes().iter().enumerate() {
        // `Formula::new` guarantees operands precede their user.
        let at = |id: NodeId| -> &Vec<bool> { &values[id.0 as usize] };
        let row: Vec<bool> = match node.kind {
            NodeKind::False => vec![false; n],
            NodeKind::True => vec![true; n],
            NodeKind::Proposition { proposition } => trace
                .iter()
                .map(|instant| instant.binary_search(&proposition).is_ok())
                .collect(),
            NodeKind::Not { operand } => at(operand).iter().map(|value| !value).collect(),
            NodeKind::And { left, right } => pointwise(at(left), at(right), |l, r| l && r),
            NodeKind::Or { left, right } => pointwise(at(left), at(right), |l, r| l || r),
            NodeKind::Implies { left, right } => pointwise(at(left), at(right), |l, r| !l || r),
            NodeKind::Equivalent { left, right } => pointwise(at(left), at(right), |l, r| l == r),
            NodeKind::LtlfFuture { operand } => {
                let operand = at(operand);
                backward(n, |i, next| operand[i] || next.unwrap_or(false))
            }
            NodeKind::LtlfGlobally { operand } => {
                let operand = at(operand);
                backward(n, |i, next| operand[i] && next.unwrap_or(true))
            }
            NodeKind::LtlfUntil { left, right } => {
                let (l, r) = (at(left), at(right));
                backward(n, |i, next| r[i] || (l[i] && next.unwrap_or(false)))
            }
            NodeKind::LtlfRelease { left, right } => {
                let (l, r) = (at(left), at(right));
                backward(n, |i, next| r[i] && (l[i] || next.unwrap_or(true)))
            }
            NodeKind::StrongNext { operand } => {
                let operand = at(operand);
                (0..n).map(|i| i < last && operand[i + 1]).collect()
            }
            NodeKind::WeakNext { operand } => {
                let operand = at(operand);
                (0..n).map(|i| i == last || operand[i + 1]).collect()
            }
            NodeKind::Future { .. }
            | NodeKind::Globally { .. }
            | NodeKind::Until { .. }
            | NodeKind::Release { .. }
            | NodeKind::Once { .. }
            | NodeKind::Historically { .. }
            | NodeKind::StrongPrevious { .. }
            | NodeKind::Since { .. }
            | NodeKind::Triggered { .. } => {
                return Err(LtlfError::NodeNotInProfile {
                    node: NodeId(index as u32),
                })
            }
        };
        values.push(row);
    }
    Ok(values[formula.root().0 as usize][0])
}

/// Binds every proposition through the signal catalog, then evaluates.
///
/// Implements: FR-083 (unbound-signal refusal)
pub fn evaluate_ltlf_with_catalog(
    formula: Formula<'_>,
    trace: &[Vec<PropositionId>],
    catalog: &SignalCatalogDocument,
    limits: LtlfLimits,
) -> Result<bool, LtlfError> {
    if formula.profile() != SemanticProfile::LtlfFiniteTraceV1 {
        return Err(LtlfError::ProfileMismatch {
            actual: formula.profile().as_str(),
        });
    }
    bind_formula(formula, catalog).map_err(LtlfError::UnboundSignal)?;
    evaluate_ltlf(formula, trace, limits)
}

fn pointwise(left: &[bool], right: &[bool], op: impl Fn(bool, bool) -> bool) -> Vec<bool> {
    left.iter()
        .zip(right)
        .map(|(l, r)| op(*l, *r))
        .collect()
}

/// Fills positions from the last to the first; `next` is the value at `i + 1`,
/// or `None` at the last position.
fn backward(n: usize, step: impl Fn(usize, Option<bool>) -> bool) -> Vec<bool> {
    let mut row = vec![false; n];
    let mut next = None;
    for i in (0..n).rev() {
        row[i] = step(i, next);
        next = Some(row[i]);
    }
    row
}

/// The three LTL3 verdicts (ADR-047).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Ltl3Verdict {
    /// Every continuation satisfies.
    True,
    /// Every continuation violates.
    False,
    /// Some continuations satisfy and some violate (nearest meaning).
    Inconclusive,
}

impl crate::TruthValue {
    /// LTL3 verdict of this truth value (ADR-047). `Pending` maps to the
    /// nearest LTL3 verdict, inconclusive. Total: every variant has a verdict.
    ///
    /// Implements: FR-085
    pub const fn ltl3(self) -> Option<Ltl3Verdict> {
        match self {
            Self::True => Some(Ltl3Verdict::True),
            Self::False => Some(Ltl3Verdict::False),
            Self::Pending => Some(Ltl3Verdict::Inconclusive),
        }
    }
}

impl crate::record::RecordVerdict {
    /// LTL3 verdict of this record verdict (ADR-047).
    ///
    /// Implements: FR-085
    pub const fn ltl3(self) -> Option<Ltl3Verdict> {
        match self {
            Self::True => Some(Ltl3Verdict::True),
            Self::False => Some(Ltl3Verdict::False),
            Self::Pending => Some(Ltl3Verdict::Inconclusive),
        }
    }
}

#[cfg(feature = "infinite-trace")]
impl crate::infinite::Disposition {
    /// LTL3 verdict of this disposition (ADR-047); `Unsupported` and `Failed`
    /// are not verdicts and map to `None`.
    ///
    /// Implements: FR-085
    pub const fn ltl3(self) -> Option<Ltl3Verdict> {
        match self {
            Self::Proved => Some(Ltl3Verdict::True),
            Self::Refuted => Some(Ltl3Verdict::False),
            Self::Inconclusive => Some(Ltl3Verdict::Inconclusive),
            Self::Unsupported | Self::Failed => None,
        }
    }
}
