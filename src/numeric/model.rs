//! Owned predicate-table view consumed by numeric evaluation and export.
//!
//! This mirrors the tl-syntax FR-302/FR-303 expression arena and predicate
//! table so that the evaluator depends on one small seam. The only place that
//! names tl-syntax's predicate-table API is [`PredicateTable::from_tl_syntax`].
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

use serde::{Deserialize, Serialize};
use tl_syntax::{PropositionId, SignalId};

/// One expression node; operands are indices of earlier nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ExprNode {
    /// Catalog signal value.
    Signal {
        /// Signal identity.
        signal: SignalId,
    },
    /// Integer literal (scale 0).
    Integer {
        /// Value.
        value: i64,
    },
    /// Fixed-decimal literal.
    Decimal {
        /// Coefficient.
        coefficient: i64,
        /// Scale, at most 18.
        scale: u8,
    },
    /// Negation.
    Neg {
        /// Operand index.
        operand: u32,
    },
    /// Absolute value.
    Abs {
        /// Operand index.
        operand: u32,
    },
    /// Sum at equal scale.
    Add {
        /// Left operand index.
        left: u32,
        /// Right operand index.
        right: u32,
    },
    /// Difference at equal scale.
    Sub {
        /// Left operand index.
        left: u32,
        /// Right operand index.
        right: u32,
    },
    /// Product; scales add.
    Mul {
        /// Left operand index.
        left: u32,
        /// Right operand index.
        right: u32,
    },
    /// Widening rescale.
    Rescale {
        /// Operand index.
        operand: u32,
        /// Target scale, at least the operand's.
        scale: u8,
    },
}

/// Comparison operator of a predicate.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Comparison {
    /// `<`
    Lt,
    /// `<=`
    Le,
    /// `>`
    Gt,
    /// `>=`
    Ge,
    /// `==`
    Eq,
    /// `!=`
    Ne,
}

impl Comparison {
    /// Every operator, in wire order.
    pub const ALL: [Self; 6] = [Self::Lt, Self::Le, Self::Gt, Self::Ge, Self::Eq, Self::Ne];

    /// Exact comparison of two equal-scale coefficients.
    pub fn holds(self, left: i128, right: i128) -> bool {
        match self {
            Self::Lt => left < right,
            Self::Le => left <= right,
            Self::Gt => left > right,
            Self::Ge => left >= right,
            Self::Eq => left == right,
            Self::Ne => left != right,
        }
    }
}

/// One predicate-defined proposition.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PredicateDefinition {
    /// Proposition the predicate defines.
    pub proposition: PropositionId,
    /// Operator.
    pub comparison: Comparison,
    /// Left expression root.
    pub left: u32,
    /// Right expression root.
    pub right: u32,
}

/// A tl-syntax expression node this owned view does not carry.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PredicateAdapterError {
    /// A template slot leaf or a node kind added after this adapter.
    UnsupportedNode {
        /// Expression node index.
        node: u32,
    },
}

/// Expression arena plus predicates strictly increasing by proposition.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PredicateTable {
    /// Expression nodes; each operand precedes its user.
    pub expressions: Vec<ExprNode>,
    /// Predicates.
    pub predicates: Vec<PredicateDefinition>,
}

impl PredicateTable {
    /// Proposition identities defined by a predicate, in table order.
    pub fn proposition_ids(&self) -> Vec<PropositionId> {
        self.predicates
            .iter()
            .map(|predicate| predicate.proposition)
            .collect()
    }

    /// Signals one expression root reads, deduplicated, increasing. Each node
    /// is visited once, so a shared DAG costs its node count.
    pub fn signals_read(&self, root: u32) -> Vec<SignalId> {
        let mut found = Vec::new();
        let mut visited = vec![false; self.expressions.len()];
        let mut stack = vec![root];
        while let Some(index) = stack.pop() {
            let Some(seen) = visited.get_mut(index as usize) else {
                continue;
            };
            if *seen {
                continue;
            }
            *seen = true;
            match self.expressions[index as usize] {
                ExprNode::Signal { signal } => found.push(signal),
                ExprNode::Neg { operand }
                | ExprNode::Abs { operand }
                | ExprNode::Rescale { operand, .. } => stack.push(operand),
                ExprNode::Add { left, right }
                | ExprNode::Sub { left, right }
                | ExprNode::Mul { left, right } => {
                    stack.push(left);
                    stack.push(right);
                }
                ExprNode::Integer { .. } | ExprNode::Decimal { .. } => {}
            }
        }
        found.sort_unstable();
        found.dedup();
        found
    }

    /// Converts a validated tl-syntax predicate table (tl-syntax FR-302/FR-303).
    ///
    /// A validated table never holds a template slot leaf (`SlotSignal`), and
    /// the tl-syntax enum is `#[non_exhaustive]`; both are refused by node.
    pub fn from_tl_syntax(
        document: &tl_syntax::PredicateTableDocument,
    ) -> Result<Self, PredicateAdapterError> {
        use tl_syntax::{ComparisonOperator as C, ExprNode as E};
        let mut expressions = Vec::with_capacity(document.expressions().len());
        for (index, node) in document.expressions().iter().enumerate() {
            expressions.push(match *node {
                E::Signal { signal } => ExprNode::Signal { signal },
                E::Integer { value } => ExprNode::Integer { value },
                E::Decimal { coefficient, scale } => ExprNode::Decimal { coefficient, scale },
                E::Neg { operand } => ExprNode::Neg { operand: operand.0 },
                E::Abs { operand } => ExprNode::Abs { operand: operand.0 },
                E::Add { left, right } => ExprNode::Add {
                    left: left.0,
                    right: right.0,
                },
                E::Sub { left, right } => ExprNode::Sub {
                    left: left.0,
                    right: right.0,
                },
                E::Mul { left, right } => ExprNode::Mul {
                    left: left.0,
                    right: right.0,
                },
                E::Rescale { operand, scale } => ExprNode::Rescale {
                    operand: operand.0,
                    scale,
                },
                _ => {
                    return Err(PredicateAdapterError::UnsupportedNode {
                        node: index as u32,
                    })
                }
            });
        }
        let predicates = document
            .predicates()
            .iter()
            .map(|predicate| PredicateDefinition {
                proposition: predicate.proposition,
                comparison: match predicate.operator {
                    C::Lt => Comparison::Lt,
                    C::Le => Comparison::Le,
                    C::Gt => Comparison::Gt,
                    C::Ge => Comparison::Ge,
                    C::Eq => Comparison::Eq,
                    C::Ne => Comparison::Ne,
                },
                left: predicate.left.0,
                right: predicate.right.0,
            })
            .collect();
        Ok(Self {
            expressions,
            predicates,
        })
    }
}
