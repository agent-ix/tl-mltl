use core::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{Formula, NodeId, NodeKind};

use crate::TL_SYNTAX_CORPUS_REVISION;

/// Versioned, identity-bearing horizon and buffer result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HorizonReport {
    /// Wire identity.
    pub schema_version: String,
    /// Caller-provided stable formula identity.
    pub formula_id: String,
    /// Root node identity within the formula document.
    pub formula_root: u32,
    /// Exact semantic-profile wire identity.
    pub semantic_profile: String,
    /// Shared corpus revision used by downstream conformance.
    pub corpus_revision: String,
    /// Maximum future offset needed at evaluation time zero.
    pub lookahead: u64,
    /// Worst-case delay before a complete observation window decides the formula.
    pub propagation_delay: u64,
    /// Number of discrete observation slots in that window.
    pub required_buffer: u64,
    /// Unit shared by all three resource values.
    pub unit: String,
}

/// Checked horizon-analysis failure.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HorizonError {
    /// A nested temporal bound exceeded `u64`.
    ArithmeticOverflow {
        /// Node at which the calculation failed.
        node: NodeId,
    },
    /// A validated formula exposed an impossible node reference.
    InvalidNodeReference {
        /// Referenced node.
        node: NodeId,
    },
}

impl fmt::Display for HorizonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ArithmeticOverflow { node } => {
                write!(formatter, "horizon arithmetic overflow at node {}", node.0)
            }
            Self::InvalidNodeReference { node } => {
                write!(
                    formatter,
                    "invalid validated-formula node reference {}",
                    node.0
                )
            }
        }
    }
}

impl std::error::Error for HorizonError {}

fn prior(values: &[u64], node: NodeId) -> Result<u64, HorizonError> {
    values
        .get(node.0 as usize)
        .copied()
        .ok_or(HorizonError::InvalidNodeReference { node })
}

fn add_bound(node: NodeId, bound: u32, child: u64) -> Result<u64, HorizonError> {
    u64::from(bound)
        .checked_add(child)
        .ok_or(HorizonError::ArithmeticOverflow { node })
}

/// Computes lookahead, propagation delay, and buffer size with checked arithmetic.
///
/// Implements: FR-002
pub(crate) fn lookahead(formula: Formula<'_>) -> Result<u64, HorizonError> {
    let mut values = Vec::with_capacity(formula.nodes().len());
    for (index, node) in formula.nodes().iter().enumerate() {
        let node_id = NodeId(index as u32);
        let value = match node.kind {
            NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => 0,
            NodeKind::Not { operand } => prior(&values, operand)?,
            NodeKind::And { left, right }
            | NodeKind::Or { left, right }
            | NodeKind::Implies { left, right }
            | NodeKind::Equivalent { left, right } => {
                prior(&values, left)?.max(prior(&values, right)?)
            }
            NodeKind::Future { interval, operand } | NodeKind::Globally { interval, operand } => {
                add_bound(node_id, interval.end(), prior(&values, operand)?)?
            }
            NodeKind::Until {
                interval,
                left,
                right,
            }
            | NodeKind::Release {
                interval,
                left,
                right,
            } => add_bound(
                node_id,
                interval.end(),
                prior(&values, left)?.max(prior(&values, right)?),
            )?,
        };
        values.push(value);
    }

    prior(&values, formula.root())
}

/// Computes lookahead, propagation delay, and buffer size with checked arithmetic.
///
/// Implements: FR-002
pub fn analyze_horizon(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
) -> Result<HorizonReport, HorizonError> {
    let lookahead = lookahead(formula)?;
    let required_buffer = lookahead
        .checked_add(1)
        .ok_or(HorizonError::ArithmeticOverflow {
            node: formula.root(),
        })?;
    Ok(HorizonReport {
        schema_version: "tl-mltl.horizon/v1".to_owned(),
        formula_id: formula_id.into(),
        formula_root: formula.root().0,
        semantic_profile: formula.profile().as_str().to_owned(),
        corpus_revision: TL_SYNTAX_CORPUS_REVISION.to_owned(),
        lookahead,
        propagation_delay: lookahead,
        required_buffer,
        unit: "discrete-instants".to_owned(),
    })
}
