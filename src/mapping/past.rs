//! Distinct origin-complete past mapping to the C2PO expression vocabulary.
//!
//! A caller must supply operator-specific target-origin evidence. The adapter
//! records that evidence; it does not claim that rendering alone executed R2U2.

use std::collections::BTreeSet;

use serde::Serialize;
use tl_syntax::{
    Formula, FormulaDocument, Interval, NodeId, NodeKind, PastOperatorKind, PropositionId,
    SemanticProfile, SignalCatalog, SignalCatalogDocument,
};

use super::legacy::{is_c2po_identifier, sha256_hex};
use super::MappingSourceIdentity;
use crate::{context::bind_formula, ContextualBindingError, ToolIdentity};

/// Reviewed target-origin behavior, bound to one exact target version and
/// operator set.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetOriginContract {
    /// Exact external tool identity.
    pub target: ToolIdentity,
    /// Past operators with a retained target observation or a zero-interval
    /// Boolean lowering that removes target-origin behavior.
    pub admitted_operators: BTreeSet<PastOperatorKind>,
}

impl TargetOriginContract {
    /// Origin observations retained for the exact R2U2 4.2 source and C2PO
    /// compiler. This is reviewed historical evidence, not a fresh target run.
    pub fn reviewed_r2u2_4_2() -> Self {
        Self {
            target: ToolIdentity {
                name: "C2PO".to_owned(),
                version: "C2PO v4.1.0".to_owned(),
            },
            admitted_operators: [
                PastOperatorKind::Once,
                PastOperatorKind::Historically,
                PastOperatorKind::StrongPrevious,
                PastOperatorKind::Since,
                PastOperatorKind::Triggered,
            ]
            .into_iter()
            .collect(),
        }
    }

    pub(crate) fn validate(&self) -> Result<(), PastMappingError> {
        if self.target.name.is_empty() || self.target.version.is_empty() {
            return Err(PastMappingError::MissingOriginEvidence);
        }
        // These interval admissions were measured only against this exact
        // retained R2U2 4.2/C2PO target.
        if self.target.name != "C2PO" || self.target.version != "C2PO v4.1.0" {
            return Err(PastMappingError::TargetOriginMismatch);
        }
        Ok(())
    }
}

/// Typed refusal without a partial C2PO expression or manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PastMappingError {
    /// A different TL profile was selected.
    UnsupportedProfile,
    /// A future-time operator was encountered in the past-profile graph.
    UnsupportedNode(NodeId),
    /// The target-origin contract is absent or malformed.
    MissingOriginEvidence,
    /// The target or evidence differs from the reviewed origin partition.
    TargetOriginMismatch,
    /// A node uses an operator with no reviewed target-origin behavior.
    TargetOriginUnverified(PastOperatorKind),
    /// This temporal nesting shape was not covered by target-origin evidence.
    TargetOriginShapeUnverified(NodeId),
    /// The selected target does not match source semantics for this interval.
    TargetOriginIntervalMismatch {
        operator: PastOperatorKind,
        interval: Interval,
    },
    /// The formula's signal catalog does not bind every proposition.
    Binding(ContextualBindingError),
    /// A catalog document failed validation.
    InvalidCatalog(String),
    /// A signal name has no exact C2PO identifier representation.
    UnsupportedSignal(PropositionId),
    /// A node reference was absent in a supposedly validated graph.
    InvalidNode(NodeId),
    /// Work exceeded the configured renderer budget.
    ResourceIncomplete,
    /// A canonical identity could not be constructed.
    Identity(String),
}

impl core::fmt::Display for PastMappingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "past C2PO mapping refused: {self:?}")
    }
}

impl std::error::Error for PastMappingError {}

/// Versioned past-profile mapping artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PastMappingManifest {
    /// Exact manifest edition.
    pub schema_version: &'static str,
    /// tl-mltl adapter version.
    pub adapter_version: &'static str,
    /// Exact tl-mltl source revision.
    pub source_revision: String,
    /// Exact tl-mltl source tree state.
    pub source_state: String,
    /// Canonical formula-v2 graph content identity.
    pub graph_id: String,
    /// Caller formula identity.
    pub formula_id: String,
    /// Exact past TL profile.
    pub profile: &'static str,
    /// Exact event-position clock.
    pub clock: &'static str,
    /// SHA-256 of the exact supplied formula bytes.
    pub input_sha256: String,
    /// SHA-256 of the complete signal catalog.
    pub signal_catalog_sha256: String,
    /// Exact target identity.
    pub target: ToolIdentity,
    /// Canonical C2PO expression, intended for a PTSPEC section.
    pub expression: String,
    /// SHA-256 of the expression bytes.
    pub output_sha256: String,
    /// Exact mapping limitation.
    pub limitation: &'static str,
}

struct Renderer<'a, 'b> {
    formula: Formula<'a>,
    catalog: SignalCatalog<'b>,
    origin: &'b TargetOriginContract,
    visits: u64,
    limit: u64,
}

impl Renderer<'_, '_> {
    fn render(&mut self, node: NodeId) -> Result<String, PastMappingError> {
        self.visits = self
            .visits
            .checked_add(1)
            .ok_or(PastMappingError::ResourceIncomplete)?;
        if self.visits > self.limit {
            return Err(PastMappingError::ResourceIncomplete);
        }
        let kind = usize::try_from(node.0)
            .ok()
            .and_then(|index| self.formula.nodes().get(index))
            .map(|node| node.kind)
            .ok_or(PastMappingError::InvalidNode(node))?;
        let rendered = match kind {
            NodeKind::False => "false".to_owned(),
            NodeKind::True => "true".to_owned(),
            NodeKind::Proposition { proposition } => {
                let signal = self
                    .catalog
                    .signal_for_proposition(proposition)
                    .ok_or(PastMappingError::UnsupportedSignal(proposition))?;
                if !is_c2po_identifier(signal.name()) {
                    return Err(PastMappingError::UnsupportedSignal(proposition));
                }
                signal.name().to_owned()
            }
            NodeKind::Not { operand } => format!("(!{})", self.render(operand)?),
            NodeKind::And { left, right } => self.binary("&&", left, right)?,
            NodeKind::Or { left, right } => self.binary("||", left, right)?,
            NodeKind::Implies { left, right } => self.binary("->", left, right)?,
            NodeKind::Equivalent { left, right } => self.binary("<->", left, right)?,
            NodeKind::Once { interval, operand } => {
                self.require_interval(PastOperatorKind::Once, interval)?;
                let child = self.render(operand)?;
                if interval.start() == 0 && interval.end() == 0 {
                    child
                } else {
                    format!("O[{},{}]({child})", interval.start(), interval.end())
                }
            }
            NodeKind::Historically { interval, operand } => {
                self.require_interval(PastOperatorKind::Historically, interval)?;
                self.render(operand)?
            }
            NodeKind::StrongPrevious { operand } => {
                self.require(PastOperatorKind::StrongPrevious)?;
                format!("O[1,1]({})", self.render(operand)?)
            }
            NodeKind::Since {
                interval,
                left,
                right,
            } => {
                self.require_interval(PastOperatorKind::Since, interval)?;
                // S[0,0] is exactly its right operand at every position.
                // The left operand is still rendered so unsupported names or
                // nested temporal shapes cannot be hidden by lowering.
                self.render(left)?;
                self.render(right)?
            }
            NodeKind::Triggered {
                interval,
                left,
                right,
            } => {
                self.require_interval(PastOperatorKind::Triggered, interval)?;
                self.require_interval(PastOperatorKind::Since, interval)?;
                // T[0,0] = !( (!left) S[0,0] (!right) ) = !!right.
                // This preserves the explicit dual without emitting a target
                // S operator whose origin behavior lacks retained evidence.
                self.render(left)?;
                format!("(!(!{}))", self.render(right)?)
            }
            NodeKind::Future { .. }
            | NodeKind::Globally { .. }
            | NodeKind::Until { .. }
            | NodeKind::Release { .. } => {
                return Err(PastMappingError::UnsupportedNode(node));
            }
        };
        Ok(rendered)
    }

    fn binary(
        &mut self,
        token: &str,
        left: NodeId,
        right: NodeId,
    ) -> Result<String, PastMappingError> {
        Ok(format!(
            "({} {token} {})",
            self.render(left)?,
            self.render(right)?
        ))
    }

    fn require(&self, operator: PastOperatorKind) -> Result<(), PastMappingError> {
        if self.origin.admitted_operators.contains(&operator) {
            Ok(())
        } else {
            Err(PastMappingError::TargetOriginUnverified(operator))
        }
    }

    fn require_interval(
        &self,
        operator: PastOperatorKind,
        interval: Interval,
    ) -> Result<(), PastMappingError> {
        self.require(operator)?;
        if target_equivalent_interval(operator, interval) {
            Ok(())
        } else {
            Err(PastMappingError::TargetOriginIntervalMismatch { operator, interval })
        }
    }
}

/// The only nontrivial target past forms backed by retained per-step
/// observations are O[0,1] and Y as O[1,1]. Zero-interval forms lower to
/// Boolean expressions and need no target past-origin behavior. Every other
/// interval is refused rather than inferred from parser acceptance.
pub(crate) fn target_equivalent_interval(operator: PastOperatorKind, interval: Interval) -> bool {
    match operator {
        PastOperatorKind::Once => interval.start() == 0 && interval.end() <= 1,
        PastOperatorKind::Historically | PastOperatorKind::Since | PastOperatorKind::Triggered => {
            interval.start() == 0 && interval.end() == 0
        }
        PastOperatorKind::StrongPrevious => false,
    }
}

#[derive(Clone, Copy)]
struct OriginShape {
    signature: Option<(PastOperatorKind, Option<Interval>)>,
    mixed: bool,
    depth: usize,
}

/// One reviewed target-origin admission rule shared by finite past mapping
/// and infinite safety export. Callers adapt their syntax-owned node kinds
/// into child ids and an optional past operator/interval signature.
pub(crate) struct OriginShapeGuard {
    states: Vec<OriginShape>,
}

impl OriginShapeGuard {
    /// Allocate one state per validated topological formula node.
    pub(crate) fn new(capacity: usize) -> Self {
        Self {
            states: Vec::with_capacity(capacity),
        }
    }

    /// Classify the next node using the reviewed homogeneous operator,
    /// interval, and depth limit; return no target artifact on refusal.
    pub(crate) fn push(
        &mut self,
        id: NodeId,
        children: [Option<NodeId>; 2],
        temporal: Option<(PastOperatorKind, Option<Interval>)>,
    ) -> Result<(), PastMappingError> {
        let mut state = OriginShape {
            signature: None,
            mixed: false,
            depth: 0,
        };
        for child in children.into_iter().flatten() {
            let child = usize::try_from(child.0)
                .ok()
                .and_then(|at| self.states.get(at))
                .ok_or(PastMappingError::InvalidNode(child))?;
            state.depth = state.depth.max(child.depth);
            state.mixed |= child.mixed;
            if let Some(signature) = child.signature {
                if state.signature.is_some_and(|prior| prior != signature) {
                    state.mixed = true;
                }
                state.signature = Some(signature);
            }
        }
        if state.mixed {
            return Err(PastMappingError::TargetOriginShapeUnverified(id));
        }
        if let Some(signature) = temporal {
            if state.signature.is_some_and(|child| child != signature) {
                return Err(PastMappingError::TargetOriginShapeUnverified(id));
            }
            state.depth = state
                .depth
                .checked_add(1)
                .ok_or(PastMappingError::ResourceIncomplete)?;
            let limit = if signature.0 == PastOperatorKind::StrongPrevious {
                2
            } else {
                3
            };
            if state.depth > limit {
                return Err(PastMappingError::TargetOriginShapeUnverified(id));
            }
            state.signature = Some(signature);
        }
        self.states.push(state);
        Ok(())
    }
}

/// Keep the admitted shape within a conservative homogeneous partition: up
/// to three temporal nodes, or two for strong previous. One formula may not
/// combine different temporal operators or intervals, including under a
/// Boolean node. This is a refusal boundary, not a broader target claim.
fn validate_origin_shape(formula: Formula<'_>) -> Result<(), PastMappingError> {
    let mut guard = OriginShapeGuard::new(formula.nodes().len());
    for (index, node) in formula.nodes().iter().enumerate() {
        let id = NodeId(u32::try_from(index).map_err(|_| PastMappingError::ResourceIncomplete)?);
        let (children, temporal) = match node.kind {
            NodeKind::False | NodeKind::True | NodeKind::Proposition { .. } => ([None, None], None),
            NodeKind::Not { operand } => ([Some(operand), None], None),
            NodeKind::And { left, right }
            | NodeKind::Or { left, right }
            | NodeKind::Implies { left, right }
            | NodeKind::Equivalent { left, right } => ([Some(left), Some(right)], None),
            NodeKind::Once { interval, operand } => (
                [Some(operand), None],
                Some((PastOperatorKind::Once, Some(interval))),
            ),
            NodeKind::Historically { interval, operand } => (
                [Some(operand), None],
                Some((PastOperatorKind::Historically, Some(interval))),
            ),
            NodeKind::StrongPrevious { operand } => (
                [Some(operand), None],
                Some((PastOperatorKind::StrongPrevious, None)),
            ),
            NodeKind::Since {
                interval,
                left,
                right,
            } => (
                [Some(left), Some(right)],
                Some((PastOperatorKind::Since, Some(interval))),
            ),
            NodeKind::Triggered {
                interval,
                left,
                right,
            } => (
                [Some(left), Some(right)],
                Some((PastOperatorKind::Triggered, Some(interval))),
            ),
            NodeKind::Future { .. }
            | NodeKind::Globally { .. }
            | NodeKind::Until { .. }
            | NodeKind::Release { .. } => return Err(PastMappingError::UnsupportedNode(id)),
        };
        guard.push(id, children, temporal)?;
    }
    Ok(())
}

/// Maps a validated formula-v2 past graph under an explicit reviewed target
/// origin contract. Every refusal returns no executable artifact.
#[allow(clippy::too_many_arguments)]
pub fn map_past_to_c2po(
    formula: Formula<'_>,
    formula_id: &str,
    formula_bytes: &[u8],
    source: MappingSourceIdentity,
    catalog_document: &SignalCatalogDocument,
    origin: &TargetOriginContract,
    work_limit: u64,
) -> Result<PastMappingManifest, PastMappingError> {
    if formula.profile() != SemanticProfile::OriginCompleteHistoryV1 {
        return Err(PastMappingError::UnsupportedProfile);
    }
    origin.validate()?;
    // The borrowed graph has structural validation but no document depth
    // ceiling. Admit its v2 owner document before recursive rendering.
    let graph_id = FormulaDocument::from_formula_v2(formula)
        .map_err(|error| PastMappingError::Identity(error.to_string()))?
        .content_identity()
        .map_err(|error| PastMappingError::Identity(error.to_string()))?;
    validate_origin_shape(formula)?;
    bind_formula(formula, catalog_document).map_err(PastMappingError::Binding)?;
    let catalog = catalog_document
        .validate()
        .map_err(|error| PastMappingError::InvalidCatalog(error.to_string()))?;
    let expression = Renderer {
        formula,
        catalog,
        origin,
        visits: 0,
        limit: work_limit,
    }
    .render(formula.root())?;
    let signal_catalog_sha256 = sha256_hex(
        &serde_json::to_vec(catalog_document)
            .map_err(|error| PastMappingError::Identity(error.to_string()))?,
    );
    Ok(PastMappingManifest {
        schema_version: "tl-mltl.past-c2po-mapping/v1",
        adapter_version: env!("CARGO_PKG_VERSION"),
        source_revision: source.revision,
        source_state: source.state.as_str().to_owned(),
        graph_id,
        formula_id: formula_id.to_owned(),
        profile: SemanticProfile::OriginCompleteHistoryV1.as_str(),
        clock: "event_position",
        input_sha256: sha256_hex(formula_bytes),
        signal_catalog_sha256,
        target: origin.target.clone(),
        output_sha256: sha256_hex(expression.as_bytes()),
        expression,
        limitation:
            "mapping records caller-supplied target-origin evidence; it does not execute R2U2",
    })
}
