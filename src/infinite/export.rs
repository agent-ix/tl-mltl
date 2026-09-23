//! Refutation-only C2PO expression export for a finite-horizon safety body.

use serde::Serialize;
use tl_syntax::{
    InfiniteNodeKind as K, NodeId, PartialValue, PastOperatorKind, SignalCatalog,
    SignalCatalogDocument, TemporalInterval,
};

use crate::{
    mapping::legacy::{is_c2po_identifier, sha256_hex},
    TargetOriginContract, ToolIdentity,
};

use super::{
    evaluate_prefix_safety_at, finite_horizons, Disposition, InfiniteError, PrefixRequest, PROFILE,
};

/// Typed refusal before a C2PO expression or manifest is emitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SafetyExportError {
    /// Graph identity or proposition-map identity differs from the request.
    Identity,
    /// The graph is outside exact `G[0,)ψ` with finite future horizon.
    UnsupportedShape,
    /// A target past-origin contract is absent or does not admit an operator.
    TargetOrigin,
    /// A past interval has no reviewed target representation.
    UnboundedPast,
    /// Mixed future/past nesting has no reviewed target parser contract.
    MixedTargetContext,
    /// Partial or conflicting observations cannot be exported to this target.
    PartialValuation,
    /// The catalog is invalid or a proposition is not bound.
    Catalog,
    /// A signal name is not an exact C2PO identifier.
    Signal,
    /// The work limit or checked arithmetic prevented complete rendering.
    ResourceIncomplete,
    /// A target step or manifest does not match the exported input or tool.
    TargetMismatch,
}

impl core::fmt::Display for SafetyExportError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "safety C2PO export refused: {self:?}")
    }
}

impl std::error::Error for SafetyExportError {}

/// A C2PO monitor expression that can supply refutation evidence only.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafetyMappingManifest {
    /// Exact manifest edition.
    pub schema_version: &'static str,
    /// Exact infinite TL profile.
    pub profile: &'static str,
    /// Exact event-position clock.
    pub clock: &'static str,
    /// Provider revision used to export this expression.
    pub provider_revision: &'static str,
    /// Canonical graph identity.
    pub graph_id: String,
    /// Bound finite-prefix input digest.
    pub input_sha256: String,
    /// Target executable and configuration identity.
    pub target: ToolIdentity,
    /// Reviewed origin evidence digest.
    pub target_origin_evidence_sha256: String,
    /// C2PO section required by the expression.
    pub section: &'static str,
    /// Exact target expression for the safety body ψ.
    pub expression: String,
    /// SHA-256 of the target expression bytes.
    pub output_sha256: String,
    /// Finite future decision horizon of ψ at each position.
    pub decision_horizon: u64,
    /// This artifact cannot prove the infinite claim from target passes.
    pub refutation_only: bool,
}

/// One observed C2PO verdict for the exported expression at an exact position.
pub struct TargetStepObservation<'a> {
    /// Exact tool executable and configuration used for the observation.
    pub target: &'a ToolIdentity,
    /// Digest of the expression the target executed.
    pub expression_sha256: &'a str,
    /// Origin-based event position of the target observation.
    pub position: u64,
    /// Boolean verdict emitted by the target for the safety body.
    pub verdict: bool,
}

/// Result of replaying one target step under the provider's infinite semantics.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SafetyReplayDisposition {
    /// The target violation is a decisive bad prefix at this exact position.
    Refuted,
    /// A target pass provides no infinite-trace proof.
    Inconclusive,
    /// The target and provider disagree at this position.
    Mismatch,
    /// The provider could not finish within its work budget.
    ResourceIncomplete,
}

/// Replays one target verdict against the exact finite prefix and position.
/// A pass is never upgraded to proof of the infinite safety claim.
pub fn replay_target_step(
    manifest: &SafetyMappingManifest,
    request: &PrefixRequest<'_>,
    observation: TargetStepObservation<'_>,
) -> Result<SafetyReplayDisposition, SafetyExportError> {
    request
        .validate_shape()
        .map_err(|_| SafetyExportError::Identity)?;
    if !manifest.refutation_only
        || manifest.profile != PROFILE
        || manifest.provider_revision != crate::TL_MLTL_SOURCE_REVISION
        || manifest.graph_id != request.graph_id
        || manifest.input_sha256
            != request
                .content_identity()
                .map_err(|_| SafetyExportError::Identity)?
        || manifest.clock != request.formula.clock().as_str()
        || manifest.target != *observation.target
        || manifest.output_sha256 != observation.expression_sha256
        || manifest.output_sha256 != sha256_hex(manifest.expression.as_bytes())
    {
        return Err(SafetyExportError::TargetMismatch);
    }
    let report = evaluate_prefix_safety_at(request, Some(observation.position)).map_err(
        |error| match error {
            InfiniteError::ResourceIncomplete => SafetyExportError::ResourceIncomplete,
            _ => SafetyExportError::Identity,
        },
    )?;
    if report.disposition == Disposition::Failed {
        return Ok(SafetyReplayDisposition::ResourceIncomplete);
    }
    let provider_refuted = report.disposition == Disposition::Refuted
        && report.identity.selected_position == observation.position;
    Ok(match (observation.verdict, provider_refuted) {
        (false, true) => SafetyReplayDisposition::Refuted,
        (true, false) => SafetyReplayDisposition::Inconclusive,
        _ => SafetyReplayDisposition::Mismatch,
    })
}

struct Renderer<'a, 'b> {
    formula: tl_syntax::InfiniteFormula<'a>,
    catalog: SignalCatalog<'b>,
    origin: &'b TargetOriginContract,
    visits: u64,
    limit: u64,
    past: bool,
    future: bool,
}

impl Renderer<'_, '_> {
    fn render(&mut self, node: NodeId) -> Result<String, SafetyExportError> {
        self.visits = self
            .visits
            .checked_add(1)
            .ok_or(SafetyExportError::ResourceIncomplete)?;
        if self.visits > self.limit {
            return Err(SafetyExportError::ResourceIncomplete);
        }
        let kind = self
            .formula
            .node(node)
            .ok_or(SafetyExportError::UnsupportedShape)?
            .kind;
        let expression = match kind {
            K::False => "false".to_owned(),
            K::True => "true".to_owned(),
            K::Proposition { proposition } => {
                let signal = self
                    .catalog
                    .signal_for_proposition(proposition)
                    .ok_or(SafetyExportError::Catalog)?;
                if !is_c2po_identifier(signal.name()) {
                    return Err(SafetyExportError::Signal);
                }
                signal.name().to_owned()
            }
            K::Not { operand } => format!("(!{})", self.render(operand)?),
            K::And { left, right } => self.binary("&&", left, right)?,
            K::Or { left, right } => self.binary("||", left, right)?,
            K::Implies { left, right } => self.binary("->", left, right)?,
            K::Equivalent { left, right } => self.binary("<->", left, right)?,
            K::Future { interval, operand } | K::Globally { interval, operand } => {
                self.future = true;
                let TemporalInterval::Closed(bounds) = interval else {
                    return Err(SafetyExportError::UnsupportedShape);
                };
                let symbol = if matches!(kind, K::Future { .. }) {
                    "F"
                } else {
                    "G"
                };
                format!(
                    "{symbol}[{},{}]({})",
                    bounds.start(),
                    bounds.end(),
                    self.render(operand)?
                )
            }
            K::Until {
                interval,
                left,
                right,
            }
            | K::Release {
                interval,
                left,
                right,
            } => {
                self.future = true;
                let TemporalInterval::Closed(bounds) = interval else {
                    return Err(SafetyExportError::UnsupportedShape);
                };
                let symbol = if matches!(kind, K::Until { .. }) {
                    "U"
                } else {
                    "R"
                };
                format!(
                    "({} {symbol}[{},{}] {})",
                    self.render(left)?,
                    bounds.start(),
                    bounds.end(),
                    self.render(right)?
                )
            }
            K::Once { interval, operand } | K::Historically { interval, operand } => {
                self.past = true;
                let operator = if matches!(kind, K::Once { .. }) {
                    PastOperatorKind::Once
                } else {
                    PastOperatorKind::Historically
                };
                self.require(operator)?;
                let TemporalInterval::Closed(bounds) = interval else {
                    return Err(SafetyExportError::UnboundedPast);
                };
                let symbol = if operator == PastOperatorKind::Once {
                    "O"
                } else {
                    "H"
                };
                format!(
                    "{symbol}[{},{}]({})",
                    bounds.start(),
                    bounds.end(),
                    self.render(operand)?
                )
            }
            K::StrongPrevious { operand } => {
                self.past = true;
                self.require(PastOperatorKind::StrongPrevious)?;
                format!("O[1,1]({})", self.render(operand)?)
            }
            K::Since {
                interval,
                left,
                right,
            }
            | K::Triggered {
                interval,
                left,
                right,
            } => {
                self.past = true;
                let trigger = matches!(kind, K::Triggered { .. });
                self.require(if trigger {
                    PastOperatorKind::Triggered
                } else {
                    PastOperatorKind::Since
                })?;
                if trigger {
                    self.require(PastOperatorKind::Since)?;
                }
                let TemporalInterval::Closed(bounds) = interval else {
                    return Err(SafetyExportError::UnboundedPast);
                };
                if trigger {
                    format!(
                        "(!((!{}) S[{},{}] (!{})))",
                        self.render(left)?,
                        bounds.start(),
                        bounds.end(),
                        self.render(right)?
                    )
                } else {
                    format!(
                        "({} S[{},{}] {})",
                        self.render(left)?,
                        bounds.start(),
                        bounds.end(),
                        self.render(right)?
                    )
                }
            }
        };
        Ok(expression)
    }

    fn binary(
        &mut self,
        symbol: &str,
        left: NodeId,
        right: NodeId,
    ) -> Result<String, SafetyExportError> {
        Ok(format!(
            "({} {symbol} {})",
            self.render(left)?,
            self.render(right)?
        ))
    }

    fn require(&self, operator: PastOperatorKind) -> Result<(), SafetyExportError> {
        if self.origin.admitted_operators.contains(&operator) {
            Ok(())
        } else {
            Err(SafetyExportError::TargetOrigin)
        }
    }
}

/// Exports the finite-horizon body of `G[0,)ψ` as a target expression.
/// A target pass is never proof of the infinite property.
pub fn export_safety_monitor(
    request: &PrefixRequest<'_>,
    catalog_document: &SignalCatalogDocument,
    origin: &TargetOriginContract,
    work_limit: u64,
) -> Result<SafetyMappingManifest, SafetyExportError> {
    request
        .validate_shape()
        .map_err(|_| SafetyExportError::Identity)?;
    let graph_id = request.graph_id.to_owned();
    origin
        .validate()
        .map_err(|_| SafetyExportError::TargetOrigin)?;
    let (inner, decision_horizon) =
        finite_horizons(request.formula.formula()).map_err(|error| match error {
            InfiniteError::ResourceIncomplete => SafetyExportError::ResourceIncomplete,
            InfiniteError::InvalidFormula
            | InfiniteError::InvalidLasso
            | InfiniteError::IdentityMismatch => SafetyExportError::UnsupportedShape,
        })?;
    for observation in request.observations {
        if observation.valuation.proposition_map_identity() != request.proposition_map_id {
            return Err(SafetyExportError::Identity);
        }
        if observation.valuation.entries().iter().any(|entry| {
            matches!(
                entry.value,
                PartialValue::Missing | PartialValue::Conflicting
            )
        }) {
            return Err(SafetyExportError::PartialValuation);
        }
    }
    let catalog = catalog_document
        .validate()
        .map_err(|_| SafetyExportError::Catalog)?;
    for proposition in request.propositions {
        if catalog.signal_for_proposition(*proposition).is_none() {
            return Err(SafetyExportError::Catalog);
        }
    }
    let mut renderer = Renderer {
        formula: request.formula.formula(),
        catalog,
        origin,
        visits: 0,
        limit: work_limit,
        past: false,
        future: false,
    };
    let expression = renderer.render(inner)?;
    if renderer.past && renderer.future {
        return Err(SafetyExportError::MixedTargetContext);
    }
    let section = if renderer.past { "PTSPEC" } else { "FTSPEC" };
    let input_sha256 = request
        .content_identity()
        .map_err(|_| SafetyExportError::Identity)?;
    let output_sha256 = sha256_hex(expression.as_bytes());
    Ok(SafetyMappingManifest {
        schema_version: "tl-mltl.infinite-safety-mapping/v1",
        profile: PROFILE,
        clock: request.formula.clock().as_str(),
        provider_revision: crate::TL_MLTL_SOURCE_REVISION,
        graph_id,
        input_sha256,
        target: origin.target.clone(),
        target_origin_evidence_sha256: origin.evidence_sha256.clone(),
        section,
        expression,
        output_sha256,
        decision_horizon,
        refutation_only: true,
    })
}
