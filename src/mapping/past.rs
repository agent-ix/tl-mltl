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
use crate::{context::bind_formula, ContextualBindingError, ToolIdentity, TL_SYNTAX_REVISION};

/// Reviewed target-origin behavior, bound to one exact target version and
/// operator set. The evidence digest identifies the separately replayable
/// target observations used to establish false-before-origin parity.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetOriginContract {
    /// Exact source revision of the reviewed C2PO/R2U2 target.
    pub source_revision: String,
    /// Exact external tool identity.
    pub target: ToolIdentity,
    /// SHA-256 of the reviewed R2U2 monitor executable.
    pub monitor_executable_sha256: String,
    /// Lowercase SHA-256 of the reviewed target-origin evidence artifact.
    pub evidence_sha256: String,
    /// Past operators whose origin behavior was reviewed for this target.
    pub admitted_operators: BTreeSet<PastOperatorKind>,
}

impl TargetOriginContract {
    /// Origin observations retained for the exact R2U2 4.2 source and C2PO
    /// compiler. This is reviewed historical evidence, not a fresh target run.
    pub fn reviewed_r2u2_4_2() -> Self {
        Self {
            source_revision: REVIEWED_SOURCE_REVISION.to_owned(),
            target: ToolIdentity {
                name: "C2PO".to_owned(),
                version: "C2PO v4.1.0".to_owned(),
                executable_sha256: REVIEWED_COMPILER_SHA256.to_owned(),
                configuration_sha256: REVIEWED_SOURCE_SHA256.to_owned(),
            },
            monitor_executable_sha256: REVIEWED_MONITOR_SHA256.to_owned(),
            evidence_sha256: REVIEWED_OBSERVATION_SHA256.to_owned(),
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
        if !is_hex_of_length(&self.source_revision, 40)
            || self.target.name.is_empty()
            || self.target.version.is_empty()
            || !is_sha256(&self.evidence_sha256)
            || !is_sha256(&self.target.executable_sha256)
            || !is_sha256(&self.target.configuration_sha256)
            || !is_sha256(&self.monitor_executable_sha256)
        {
            return Err(PastMappingError::MissingOriginEvidence);
        }
        // These interval admissions were measured only against this exact
        // retained R2U2 4.2/C2PO target and origin-observation artifact.
        // A caller-supplied digest of the right shape is not equivalence
        // evidence for another target.
        if self.source_revision != REVIEWED_SOURCE_REVISION
            || self.target.name != "C2PO"
            || self.target.version != "C2PO v4.1.0"
            || self.target.executable_sha256 != REVIEWED_COMPILER_SHA256
            || self.target.configuration_sha256 != REVIEWED_SOURCE_SHA256
            || self.monitor_executable_sha256 != REVIEWED_MONITOR_SHA256
            || self.evidence_sha256 != REVIEWED_OBSERVATION_SHA256
        {
            return Err(PastMappingError::TargetOriginMismatch);
        }
        Ok(())
    }
}

const REVIEWED_SOURCE_REVISION: &str = "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a";
const REVIEWED_COMPILER_SHA256: &str =
    "f978a32f667a8247c387a66bce35371c97b7d8f7b730035a8ee40cdfc428ce12";
const REVIEWED_SOURCE_SHA256: &str =
    "4e0c904eccfbf7a2efdd08dfe268d1862d3a2ea473595e34afd118af4a6cb915";
const REVIEWED_MONITOR_SHA256: &str =
    "5743987dddb47cc01829a633e15623095c9c2aff2f8bb24e30d7f0e0f488f85f";
const REVIEWED_OBSERVATION_SHA256: &str =
    "378b4ba53bb5a4aa685fe26a570285df171a3a829d984afd2a682ceb60312085";

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
    /// Exact tl-syntax dependency revision.
    pub syntax_revision: &'static str,
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
    /// Exact target binary/configuration identity.
    pub target: ToolIdentity,
    /// Exact source revision of the reviewed target implementation.
    pub target_source_revision: String,
    /// Exact R2U2 monitor executable reviewed for origin parity.
    pub target_monitor_sha256: String,
    /// SHA-256 of separately reviewed target-origin evidence.
    pub target_origin_evidence_sha256: String,
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
                format!(
                    "O[{},{}]({})",
                    interval.start(),
                    interval.end(),
                    self.render(operand)?
                )
            }
            NodeKind::Historically { interval, operand } => {
                self.require_interval(PastOperatorKind::Historically, interval)?;
                format!(
                    "H[{},{}]({})",
                    interval.start(),
                    interval.end(),
                    self.render(operand)?
                )
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
                format!(
                    "({} S[{},{}] {})",
                    self.render(left)?,
                    interval.start(),
                    interval.end(),
                    self.render(right)?
                )
            }
            NodeKind::Triggered {
                interval,
                left,
                right,
            } => {
                self.require_interval(PastOperatorKind::Triggered, interval)?;
                self.require_interval(PastOperatorKind::Since, interval)?;
                format!(
                    "(!((!{}) S[{},{}] (!{})))",
                    self.render(left)?,
                    interval.start(),
                    interval.end(),
                    self.render(right)?
                )
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

/// Conservative C2PO 4.2 interval partition from the observed origin grid.
/// The larger grid found H[0,1] semantic mismatches and incomplete target
/// rows for wider O windows. Unreviewed cells are refused rather than
/// inferred from parser acceptance or a shorter retained trace.
pub(crate) fn target_equivalent_interval(operator: PastOperatorKind, interval: Interval) -> bool {
    match operator {
        PastOperatorKind::Once | PastOperatorKind::Since => {
            interval.start() == 0 && interval.end() <= 1
        }
        PastOperatorKind::Historically | PastOperatorKind::Triggered => {
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

/// The reviewed target grid covers homogeneous past chains only: up to three
/// temporal nodes, or two for strong previous. One formula may not combine
/// different temporal operators or intervals, including under a Boolean
/// node. This is a conservative evidence boundary.
fn validate_origin_shape(formula: Formula<'_>) -> Result<(), PastMappingError> {
    let mut states: Vec<OriginShape> = Vec::with_capacity(formula.nodes().len());
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
        let mut state = OriginShape {
            signature: None,
            mixed: false,
            depth: 0,
        };
        for child in children.into_iter().flatten() {
            let child = usize::try_from(child.0)
                .ok()
                .and_then(|at| states.get(at))
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
        states.push(state);
    }
    Ok(())
}

fn is_sha256(value: &str) -> bool {
    is_hex_of_length(value, 64)
}

fn is_hex_of_length(value: &str, length: usize) -> bool {
    if value.len() != length {
        return false;
    }
    for byte in value.bytes() {
        if !matches!(byte, b'0'..=b'9' | b'a'..=b'f') {
            return false;
        }
    }
    true
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
        syntax_revision: TL_SYNTAX_REVISION,
        graph_id,
        formula_id: formula_id.to_owned(),
        profile: SemanticProfile::OriginCompleteHistoryV1.as_str(),
        clock: "event_position",
        input_sha256: sha256_hex(formula_bytes),
        signal_catalog_sha256,
        target: origin.target.clone(),
        target_source_revision: origin.source_revision.clone(),
        target_monitor_sha256: origin.monitor_executable_sha256.clone(),
        target_origin_evidence_sha256: origin.evidence_sha256.clone(),
        output_sha256: sha256_hex(expression.as_bytes()),
        expression,
        limitation:
            "mapping records caller-supplied target-origin evidence; it does not execute R2U2",
    })
}
