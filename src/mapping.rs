use core::fmt;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tl_syntax::{Formula, NodeId, NodeKind, SemanticProfile};

use crate::{ToolIdentity, TL_SYNTAX_REVISION};

const MAX_RECURSION_DEPTH: u32 = 512;

/// Versioned R2U2/C2PO mapping record.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MappingManifest {
    /// Wire identity.
    pub schema_version: String,
    /// Adapter implementation identity.
    pub adapter_version: String,
    /// Exact tl-mltl source identity supplied by the build.
    pub source_revision: String,
    /// Exact tl-syntax dependency identity.
    pub syntax_revision: String,
    /// Caller-provided formula identity.
    pub formula_id: String,
    /// Online semantic profile identity.
    pub semantic_profile: String,
    /// SHA-256 of the exact formula input bytes.
    pub input_sha256: String,
    /// Deterministic C2PO expression.
    pub expression: String,
    /// SHA-256 of the exact UTF-8 expression bytes.
    pub output_sha256: String,
    /// Referenced proposition identities in stable order.
    pub proposition_ids: Vec<u32>,
    /// Optional identity of an actual external tool; absence means not executed.
    pub external_tool: Option<ToolIdentity>,
    /// Qualification boundary statement.
    pub limitation: String,
}

/// Mapping failure with no partial executable output.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MappingError {
    /// R2U2/C2PO mapping is defined only for online-prefix semantics.
    UnsupportedProfile {
        /// Actual profile.
        actual: &'static str,
    },
    /// A validated formula exposed an impossible node reference.
    InvalidNodeReference(NodeId),
    /// Formula expansion exceeded the configured node budget.
    WorkLimitExceeded {
        /// Configured node budget.
        limit: u64,
    },
    /// Formula nesting exceeded the process-safe recursion boundary.
    RecursionDepthExceeded {
        /// Fixed nesting boundary.
        limit: u32,
    },
}

impl fmt::Display for MappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { actual } => write!(
                formatter,
                "R2U2/C2PO mapping requires {}, found {actual}",
                SemanticProfile::OnlinePrefixV1.as_str()
            ),
            Self::InvalidNodeReference(node) => {
                write!(
                    formatter,
                    "invalid validated-formula node reference {}",
                    node.0
                )
            }
            Self::WorkLimitExceeded { limit } => {
                write!(formatter, "mapping exceeded work limit {limit}")
            }
            Self::RecursionDepthExceeded { limit } => {
                write!(formatter, "mapping exceeded recursion-depth limit {limit}")
            }
        }
    }
}

impl std::error::Error for MappingError {}

struct Renderer<'a> {
    formula: Formula<'a>,
    visits: u64,
    limit: u64,
}

impl Renderer<'_> {
    fn render(&mut self, node: NodeId, depth: u32) -> Result<String, MappingError> {
        if depth > MAX_RECURSION_DEPTH {
            return Err(MappingError::RecursionDepthExceeded {
                limit: MAX_RECURSION_DEPTH,
            });
        }
        self.visits = self
            .visits
            .checked_add(1)
            .ok_or(MappingError::WorkLimitExceeded { limit: self.limit })?;
        if self.visits > self.limit {
            return Err(MappingError::WorkLimitExceeded { limit: self.limit });
        }
        let child_depth = depth
            .checked_add(1)
            .ok_or(MappingError::RecursionDepthExceeded {
                limit: MAX_RECURSION_DEPTH,
            })?;
        let kind = self
            .formula
            .nodes()
            .get(node.0 as usize)
            .map(|value| value.kind)
            .ok_or(MappingError::InvalidNodeReference(node))?;
        match kind {
            NodeKind::False => Ok("false".to_owned()),
            NodeKind::True => Ok("true".to_owned()),
            NodeKind::Proposition { proposition } => Ok(format!("p{}", proposition.0)),
            NodeKind::Not { operand } => Ok(format!("(!{})", self.render(operand, child_depth)?)),
            NodeKind::And { left, right } => self.binary("&&", left, right, child_depth),
            NodeKind::Or { left, right } => self.binary("||", left, right, child_depth),
            NodeKind::Implies { left, right } => self.binary("->", left, right, child_depth),
            NodeKind::Equivalent { left, right } => self.binary("<->", left, right, child_depth),
            NodeKind::Future { interval, operand } => Ok(format!(
                "F[{},{}]({})",
                interval.start(),
                interval.end(),
                self.render(operand, child_depth)?
            )),
            NodeKind::Globally { interval, operand } => Ok(format!(
                "G[{},{}]({})",
                interval.start(),
                interval.end(),
                self.render(operand, child_depth)?
            )),
            NodeKind::Until {
                interval,
                left,
                right,
            } => self.temporal_binary(
                "U",
                interval.start(),
                interval.end(),
                left,
                right,
                child_depth,
            ),
            NodeKind::Release {
                interval,
                left,
                right,
            } => self.temporal_binary(
                "R",
                interval.start(),
                interval.end(),
                left,
                right,
                child_depth,
            ),
        }
    }

    fn binary(
        &mut self,
        operator: &str,
        left: NodeId,
        right: NodeId,
        child_depth: u32,
    ) -> Result<String, MappingError> {
        Ok(format!(
            "({} {operator} {})",
            self.render(left, child_depth)?,
            self.render(right, child_depth)?
        ))
    }

    fn temporal_binary(
        &mut self,
        operator: &str,
        start: u32,
        end: u32,
        left: NodeId,
        right: NodeId,
        child_depth: u32,
    ) -> Result<String, MappingError> {
        Ok(format!(
            "({} {operator}[{start},{end}] {})",
            self.render(left, child_depth)?,
            self.render(right, child_depth)?
        ))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Emits a deterministic mapping manifest for the supported online-prefix subset.
///
/// Implements: FR-004
pub fn map_to_c2po(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    formula_bytes: &[u8],
    source_revision: impl Into<String>,
    external_tool: Option<ToolIdentity>,
    work_limit: u64,
) -> Result<MappingManifest, MappingError> {
    if formula.profile() != SemanticProfile::OnlinePrefixV1 {
        return Err(MappingError::UnsupportedProfile {
            actual: formula.profile().as_str(),
        });
    }
    let mut renderer = Renderer {
        formula,
        visits: 0,
        limit: work_limit,
    };
    let expression = renderer.render(formula.root(), 0)?;
    let proposition_ids = formula
        .nodes()
        .iter()
        .filter_map(|node| match node.kind {
            NodeKind::Proposition { proposition } => Some(proposition.0),
            _ => None,
        })
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    Ok(MappingManifest {
        schema_version: "tl-mltl.monitor-mapping/v1".to_owned(),
        adapter_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_revision: source_revision.into(),
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        formula_id: formula_id.into(),
        semantic_profile: formula.profile().as_str().to_owned(),
        input_sha256: sha256_hex(formula_bytes),
        output_sha256: sha256_hex(expression.as_bytes()),
        expression,
        proposition_ids,
        external_tool,
        limitation:
            "mapping evidence does not establish external monitor timing, memory, or qualification"
                .to_owned(),
    })
}
