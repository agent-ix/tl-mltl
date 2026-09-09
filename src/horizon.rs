use core::fmt;

use serde::{Deserialize, Serialize};
use tl_syntax::{Formula, NodeId, NodeKind, RequirementContextDocument, SignalCatalogDocument};

use crate::{
    context::{
        bind_formula, catalog_sha256, contextual_formula_request_sha256, contextual_result_sha256,
    },
    ContextualBindingError, TL_SYNTAX_CORPUS_REVISION, TL_SYNTAX_REVISION,
};

/// Versioned, identity-bearing horizon and buffer result.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct HorizonReport {
    /// Wire identity.
    #[serde(deserialize_with = "deserialize_horizon_v1_schema")]
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

fn deserialize_horizon_v1_schema<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let schema_version = String::deserialize(deserializer)?;
    if schema_version == "tl-mltl.horizon/v1" {
        Ok(schema_version)
    } else {
        Err(serde::de::Error::custom(format!(
            "expected tl-mltl.horizon/v1, found {schema_version}"
        )))
    }
}

/// Closed schema identity for a context-bound horizon record.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContextualHorizonSchemaVersion {
    /// Context-bound horizon report.
    #[serde(rename = "tl-mltl.horizon/v2")]
    V2,
}

/// Flat v2 horizon report carrying shared catalog and caller context identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextualHorizonReport {
    /// Closed v2 wire identity.
    pub schema_version: ContextualHorizonSchemaVersion,
    /// Exact tl-mltl source revision compiled into this result.
    pub source_revision: String,
    /// Exact shared tl-syntax dependency revision.
    pub syntax_revision: String,
    /// SHA-256 identity of the complete shared catalog.
    pub signal_catalog_sha256: String,
    /// Exact caller context, or deliberate absence encoded as null.
    pub requirement_context: Option<RequirementContextDocument>,
    /// Domain-separated complete request identity.
    pub request_sha256: String,
    /// Domain-separated result identity excluding only this field itself.
    pub result_sha256: String,
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

deserialize_contextual_record!(ContextualHorizonReport {
    schema_version: ContextualHorizonSchemaVersion,
    source_revision: String,
    syntax_revision: String,
    signal_catalog_sha256: String,
    request_sha256: String,
    result_sha256: String,
    formula_id: String,
    formula_root: u32,
    semantic_profile: String,
    corpus_revision: String,
    lookahead: u64,
    propagation_delay: u64,
    required_buffer: u64,
    unit: String,
});

/// Failure from context-bound horizon analysis.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualHorizonError {
    /// The shared catalog does not bind the formula.
    Binding(ContextualBindingError),
    /// Existing horizon analysis refused the operation.
    Horizon(HorizonError),
    /// The deterministic identity could not be constructed or serialized.
    Identity(String),
}

impl fmt::Display for ContextualHorizonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(error) => error.fmt(formatter),
            Self::Horizon(error) => error.fmt(formatter),
            Self::Identity(error) => write!(formatter, "contextual identity failed: {error}"),
        }
    }
}

impl std::error::Error for ContextualHorizonError {}

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

#[cfg(kani)]
mod kani_proofs {
    use tl_syntax::NodeId;

    use super::add_bound;

    // Trace: TC-006, FR-002-AC-2, NFR-001-AC-1
    // This is intentionally the arithmetic primitive used by horizon traversal;
    // it does not claim a proof of arbitrary formula traversal.
    #[kani::proof]
    fn horizon_bound_addition_preserves_zero_and_refuses_overflow() {
        let child: u64 = kani::any();
        assert_eq!(add_bound(NodeId(0), 0, child), Ok(child));

        let positive_bound: u32 = kani::any();
        kani::assume(positive_bound > 0);
        assert!(matches!(
            add_bound(NodeId(0), positive_bound, u64::MAX),
            Err(super::HorizonError::ArithmeticOverflow { node: NodeId(0) })
        ));
    }
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

/// Computes horizon analysis after binding every proposition through the exact
/// caller-supplied shared catalog.
pub fn analyze_horizon_with_context(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<ContextualHorizonReport, ContextualHorizonError> {
    bind_formula(formula, signal_catalog).map_err(ContextualHorizonError::Binding)?;
    let formula_id = formula_id.into();
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Request<'a> {
        formula_id: &'a str,
        source_revision: &'a str,
        syntax_revision: &'a str,
    }
    let request = Request {
        formula_id: &formula_id,
        source_revision: env!("TL_MLTL_SOURCE_REVISION"),
        syntax_revision: TL_SYNTAX_REVISION,
    };
    let request_sha256 = contextual_formula_request_sha256(
        "tl-mltl.contextual-horizon/v2/request",
        &request,
        formula,
        signal_catalog,
        requirement_context,
    )
    .map_err(ContextualHorizonError::Identity)?;
    let report =
        analyze_horizon(formula, formula_id.clone()).map_err(ContextualHorizonError::Horizon)?;
    let mut contextual = ContextualHorizonReport {
        schema_version: ContextualHorizonSchemaVersion::V2,
        source_revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        signal_catalog_sha256: catalog_sha256(signal_catalog)
            .map_err(|error| ContextualHorizonError::Identity(error.to_string()))?,
        requirement_context: requirement_context.cloned(),
        request_sha256,
        result_sha256: String::new(),
        formula_id: report.formula_id,
        formula_root: report.formula_root,
        semantic_profile: report.semantic_profile,
        corpus_revision: report.corpus_revision,
        lookahead: report.lookahead,
        propagation_delay: report.propagation_delay,
        required_buffer: report.required_buffer,
        unit: report.unit,
    };
    contextual.result_sha256 =
        contextual_result_sha256("tl-mltl.contextual-horizon/v2/result", &contextual)
            .map_err(|error| ContextualHorizonError::Identity(error.to_string()))?;
    Ok(contextual)
}

#[cfg(test)]
mod tests {
    use tl_syntax::{
        FormulaDocument, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
        RequirementContextDocument, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
        SourceSpan,
    };

    use super::{
        analyze_horizon_with_context, ContextualHorizonReport, ContextualHorizonSchemaVersion,
    };

    // Trace: TC-025, TC-029, FR-007-AC-1, FR-007-AC-5
    #[test]
    fn contextual_horizon_binds_context_and_rejects_missing_request_identity() {
        let document = FormulaDocument::new(
            SemanticProfile::ClosedTraceV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition {
                proposition: tl_syntax::PropositionId(7),
            })],
        )
        .unwrap();
        let catalog = SignalCatalogDocument::new(
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
        .unwrap();
        let context = RequirementContextDocument::new(
            "agent-ix/tl-mltl/FR-007".to_owned(),
            "1".to_owned(),
            "AC-1".to_owned(),
            "contextual-horizon".to_owned(),
            SourceSpan::new(0, 1).unwrap(),
        )
        .unwrap();
        let report = analyze_horizon_with_context(
            document.validate().unwrap(),
            "formula",
            &catalog,
            Some(&context),
        )
        .unwrap();
        assert_eq!(report.schema_version, ContextualHorizonSchemaVersion::V2);
        assert_eq!(report.requirement_context, Some(context));
        let mut wire = serde_json::to_value(report).unwrap();
        wire.as_object_mut().unwrap().remove("requestSha256");
        assert!(serde_json::from_value::<ContextualHorizonReport>(wire).is_err());
    }
}
