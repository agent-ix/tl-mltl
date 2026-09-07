use core::fmt;
use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tl_syntax::{
    Formula, NodeId, NodeKind, PropositionId, RequirementContextDocument, SemanticProfile,
    SignalCatalog, SignalCatalogDocument, SignalId,
};

use crate::{
    context::{bind_formula, catalog_sha256, contextual_request_sha256, contextual_result_sha256},
    ContextualBindingError, ToolIdentity, MAX_RECURSION_DEPTH, TL_SYNTAX_REVISION,
};

/// Named source identity embedded in a mapping manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MappingSourceIdentity {
    /// Exact source revision.
    pub revision: String,
    /// Whether the identified source tree was clean or modified.
    pub state: MappingSourceState,
}

/// Closed source-state classification for a mapping manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MappingSourceState {
    /// The tracked source tree matched the revision.
    Clean,
    /// The tracked source tree contained modifications.
    Modified,
}

impl MappingSourceState {
    /// Parses the build-time wire value.
    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "clean" => Some(Self::Clean),
            "modified" => Some(Self::Modified),
            _ => None,
        }
    }

    /// Returns the stable mapping-manifest wire value.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Clean => "clean",
            Self::Modified => "modified",
        }
    }
}

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
    /// Whether the source checkout was clean or modified when built.
    pub source_state: String,
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

/// Closed schema identity for a context-bound mapping manifest.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum ContextualMappingSchemaVersion {
    /// Context-bound mapping manifest.
    #[serde(rename = "tl-mltl.monitor-mapping/v2")]
    V2,
}

/// Flat v2 mapping manifest carrying shared catalog and caller context identity.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContextualMappingManifest {
    /// Closed v2 wire identity.
    pub schema_version: ContextualMappingSchemaVersion,
    /// Adapter implementation identity.
    pub adapter_version: String,
    /// Exact tl-mltl source identity supplied by the build.
    pub source_revision: String,
    /// Whether the source checkout was clean or modified when built.
    pub source_state: String,
    /// Exact tl-syntax dependency identity.
    pub syntax_revision: String,
    /// SHA-256 identity of the complete shared catalog.
    pub signal_catalog_sha256: String,
    /// Exact caller context, or deliberate absence encoded as null.
    pub requirement_context: Option<RequirementContextDocument>,
    /// Domain-separated complete request identity.
    pub request_sha256: String,
    /// Domain-separated result identity excluding only this field itself.
    pub result_sha256: String,
    /// Caller-provided formula identity.
    pub formula_id: String,
    /// Online semantic profile identity.
    pub semantic_profile: String,
    /// SHA-256 of the exact formula input bytes.
    pub input_sha256: String,
    /// Deterministic C2PO expression using exact shared names.
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
    /// The shared catalog does not bind the formula.
    Binding(ContextualBindingError),
    /// The caller-supplied catalog document no longer validates.
    InvalidCatalog(String),
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
    /// A shared Boolean signal name is not valid C2PO input syntax.
    UnsupportedSignalName {
        /// Shared signal identity.
        signal: SignalId,
        /// Exact shared signal name.
        name: String,
    },
}

impl fmt::Display for MappingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Binding(error) => error.fmt(formatter),
            Self::InvalidCatalog(error) => write!(formatter, "invalid signal catalog: {error}"),
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
            Self::UnsupportedSignalName { signal, name } => write!(
                formatter,
                "signal {} name {name:?} is not a supported C2PO input identifier",
                signal.0
            ),
        }
    }
}

impl std::error::Error for MappingError {}

struct Renderer<'formula, 'catalog> {
    formula: Formula<'formula>,
    catalog: Option<SignalCatalog<'catalog>>,
    visits: u64,
    limit: u64,
}

impl Renderer<'_, '_> {
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
            NodeKind::Proposition { proposition } => self.proposition(proposition),
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

    fn proposition(&self, proposition: PropositionId) -> Result<String, MappingError> {
        let Some(catalog) = self.catalog else {
            return Ok(format!("p{}", proposition.0));
        };
        let signal = catalog
            .signal_for_proposition(proposition)
            .ok_or(MappingError::InvalidNodeReference(NodeId(proposition.0)))?;
        let name = signal.name();
        if !is_c2po_identifier(name) {
            return Err(MappingError::UnsupportedSignalName {
                signal: signal.id(),
                name: name.to_owned(),
            });
        }
        Ok(name.to_owned())
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

fn is_c2po_identifier(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == b'_') {
        return false;
    }
    if !bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_') {
        return false;
    }
    !matches!(
        value,
        "STRUCT"
            | "ENUM"
            | "INPUT"
            | "DEFINE"
            | "FTSPEC"
            | "PTSPEC"
            | "foreach"
            | "forsome"
            | "forexactly"
            | "foratleast"
            | "foratmost"
            | "TAU"
            | "pow"
            | "sqrt"
            | "abs"
            | "xor"
            | "prev"
            | "G"
            | "F"
            | "H"
            | "O"
            | "U"
            | "R"
            | "S"
            | "T"
            | "M"
            | "true"
            | "false"
    )
}

pub(crate) fn render_contextual_expression(
    formula: Formula<'_>,
    catalog_document: &SignalCatalogDocument,
    work_limit: u64,
) -> Result<String, MappingError> {
    bind_formula(formula, catalog_document).map_err(MappingError::Binding)?;
    let catalog = catalog_document
        .validate()
        .map_err(|error| MappingError::InvalidCatalog(error.to_string()))?;
    let mut renderer = Renderer {
        formula,
        catalog: Some(catalog),
        visits: 0,
        limit: work_limit,
    };
    renderer.render(formula.root(), 0)
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
    source: MappingSourceIdentity,
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
        catalog: None,
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
        source_revision: source.revision,
        source_state: source.state.as_str().to_owned(),
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

/// Emits a contextual v2 mapping manifest using exact shared Boolean names.
pub fn map_to_c2po_with_context(
    formula: Formula<'_>,
    formula_id: impl Into<String>,
    formula_bytes: &[u8],
    source: MappingSourceIdentity,
    external_tool: Option<ToolIdentity>,
    work_limit: u64,
    signal_catalog: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<ContextualMappingManifest, MappingError> {
    if formula.profile() != SemanticProfile::OnlinePrefixV1 {
        return Err(MappingError::UnsupportedProfile {
            actual: formula.profile().as_str(),
        });
    }
    let formula_id = formula_id.into();
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Request<'a> {
        formula_id: &'a str,
        formula_nodes: &'a [tl_syntax::Node],
        formula_bytes: &'a [u8],
        source_revision: &'a str,
        source_state: &'a str,
        external_tool: Option<&'a ToolIdentity>,
        work_limit: u64,
        syntax_revision: &'a str,
    }
    let request = Request {
        formula_id: &formula_id,
        formula_nodes: formula.nodes(),
        formula_bytes,
        source_revision: &source.revision,
        source_state: source.state.as_str(),
        external_tool: external_tool.as_ref(),
        work_limit,
        syntax_revision: TL_SYNTAX_REVISION,
    };
    let request_sha256 = contextual_request_sha256(
        "tl-mltl.contextual-mapping/v2/request",
        &request,
        signal_catalog,
        requirement_context,
    )
    .map_err(|error| MappingError::InvalidCatalog(error.to_string()))?;
    let expression = render_contextual_expression(formula, signal_catalog, work_limit)?;
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
    let mut manifest = ContextualMappingManifest {
        schema_version: ContextualMappingSchemaVersion::V2,
        adapter_version: env!("CARGO_PKG_VERSION").to_owned(),
        source_revision: source.revision,
        source_state: source.state.as_str().to_owned(),
        syntax_revision: TL_SYNTAX_REVISION.to_owned(),
        signal_catalog_sha256: catalog_sha256(signal_catalog)
            .map_err(|error| MappingError::InvalidCatalog(error.to_string()))?,
        requirement_context: requirement_context.cloned(),
        request_sha256,
        result_sha256: String::new(),
        formula_id,
        semantic_profile: formula.profile().as_str().to_owned(),
        input_sha256: sha256_hex(formula_bytes),
        output_sha256: sha256_hex(expression.as_bytes()),
        expression,
        proposition_ids,
        external_tool,
        limitation:
            "mapping evidence does not establish external monitor timing, memory, or qualification"
                .to_owned(),
    };
    manifest.result_sha256 =
        contextual_result_sha256("tl-mltl.contextual-mapping/v2/result", &manifest)
            .map_err(|error| MappingError::InvalidCatalog(error.to_string()))?;
    Ok(manifest)
}

#[cfg(test)]
mod tests {
    use tl_syntax::{
        FormulaDocument, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
        SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
    };

    use super::{
        map_to_c2po_with_context, render_contextual_expression, ContextualMappingManifest,
        MappingError, MappingSourceIdentity, MappingSourceState,
    };

    fn formula() -> FormulaDocument {
        FormulaDocument::new(
            SemanticProfile::OnlinePrefixV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition {
                proposition: tl_syntax::PropositionId(7),
            })],
        )
        .unwrap()
    }

    fn catalog(name: &str) -> SignalCatalogDocument {
        SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(1),
                name.to_owned(),
                SignalDomain::Boolean,
            )],
            vec![PropositionBinding::new(
                tl_syntax::PropositionId(7),
                SignalId(1),
            )],
        )
        .unwrap()
    }

    #[test]
    fn contextual_mapping_renders_the_exact_shared_signal_name() {
        let document = formula();
        assert_eq!(
            render_contextual_expression(
                document.validate().unwrap(),
                &catalog("request_ready"),
                8
            )
            .unwrap(),
            "request_ready"
        );
    }

    #[test]
    fn contextual_mapping_refuses_reserved_names_without_an_expression() {
        let document = formula();
        assert!(matches!(
            render_contextual_expression(document.validate().unwrap(), &catalog("G"), 8),
            Err(MappingError::UnsupportedSignalName {
                signal: SignalId(1),
                ..
            })
        ));
    }

    #[test]
    fn contextual_mapping_emits_a_closed_v2_manifest() {
        let document = formula();
        let report = map_to_c2po_with_context(
            document.validate().unwrap(),
            "formula",
            b"p7",
            MappingSourceIdentity {
                revision: "source".to_owned(),
                state: MappingSourceState::Clean,
            },
            None,
            8,
            &catalog("request_ready"),
            None,
        )
        .unwrap();
        assert_eq!(report.expression, "request_ready");
        let mut wire = serde_json::to_value(report).unwrap();
        wire.as_object_mut().unwrap().remove("signalCatalogSha256");
        assert!(serde_json::from_value::<ContextualMappingManifest>(wire).is_err());
    }
}
