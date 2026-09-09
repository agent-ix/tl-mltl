//! Shared typed-catalog binding and identity helpers for contextual results.
//!
//! This module owns only tl-mltl's operation-level checks and digests. Signal
//! catalogs and requirement contexts remain the exact shared tl-syntax documents.

use serde::Serialize;
use sha2::{Digest, Sha256};
use tl_syntax::{
    Formula, FormulaBindingError, FormulaDocument, PropositionId, RequirementContextDocument,
    SignalCatalogDocument,
};

/// A context field that accepts an explicit JSON null but never a missing key.
pub(crate) struct RequiredContext(pub Option<RequirementContextDocument>);

impl<'de> serde::Deserialize<'de> for RequiredContext {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ContextVisitor;
        impl<'de> serde::de::Visitor<'de> for ContextVisitor {
            type Value = RequiredContext;

            fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                formatter.write_str("an explicit requirement context object or null")
            }

            fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RequiredContext(None))
            }

            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Ok(RequiredContext(None))
            }

            fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                <RequirementContextDocument as serde::Deserialize>::deserialize(
                    serde::de::value::MapAccessDeserializer::new(map),
                )
                .map(|value| RequiredContext(Some(value)))
            }
        }
        deserializer.deserialize_any(ContextVisitor)
    }
}

/// A contextual operation could not establish a complete formula binding.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualBindingError {
    /// The caller-supplied catalog document no longer validates.
    InvalidCatalog(String),
    /// A formula proposition has no direct binding in the supplied catalog.
    MissingProposition { proposition: PropositionId },
    /// A future shared binding refusal was preserved as a fail-closed result.
    Refused(String),
}

impl core::fmt::Display for ContextualBindingError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidCatalog(detail) => write!(formatter, "invalid signal catalog: {detail}"),
            Self::MissingProposition { proposition } => {
                write!(
                    formatter,
                    "missing signal binding for proposition {}",
                    proposition.0
                )
            }
            Self::Refused(detail) => write!(formatter, "signal binding refused: {detail}"),
        }
    }
}

impl std::error::Error for ContextualBindingError {}

/// Validates the shared catalog and binds every proposition before operation work.
pub(crate) fn bind_formula(
    formula: Formula<'_>,
    catalog_document: &SignalCatalogDocument,
) -> Result<(), ContextualBindingError> {
    let catalog = catalog_document
        .validate()
        .map_err(|error| ContextualBindingError::InvalidCatalog(error.to_string()))?;
    match catalog.bind_formula(formula) {
        Ok(_) => Ok(()),
        Err(FormulaBindingError::MissingPropositionBinding { proposition }) => {
            Err(ContextualBindingError::MissingProposition { proposition })
        }
        Err(error) => Err(ContextualBindingError::Refused(error.to_string())),
    }
}

/// Returns the deterministic SHA-256 identity of a complete shared catalog.
pub(crate) fn catalog_sha256(
    catalog_document: &SignalCatalogDocument,
) -> Result<String, serde_json::Error> {
    sha256_json(catalog_document)
}

/// Returns a domain-separated SHA-256 over canonical JSON input.
pub(crate) fn domain_sha256<T: Serialize>(
    domain: &str,
    value: &T,
) -> Result<String, serde_json::Error> {
    let bytes = serde_json::to_vec(value)?;
    let mut digest = Sha256::new();
    digest.update(domain.as_bytes());
    digest.update([0]);
    digest.update(bytes);
    Ok(hex(digest.finalize()))
}

/// Returns an operation request identity bound to complete shared inputs.
pub(crate) fn contextual_request_sha256<T: Serialize>(
    domain: &str,
    operation: &T,
    catalog_document: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<String, serde_json::Error> {
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Request<'a, T> {
        operation: &'a T,
        signal_catalog: &'a SignalCatalogDocument,
        requirement_context: Option<&'a RequirementContextDocument>,
    }
    domain_sha256(
        domain,
        &Request {
            operation,
            signal_catalog: catalog_document,
            requirement_context,
        },
    )
}

/// Returns a contextual request identity whose formula contribution uses the
/// shared span-free semantic document view.
pub(crate) fn contextual_formula_request_sha256<T: Serialize>(
    domain: &str,
    operation: &T,
    formula: Formula<'_>,
    catalog_document: &SignalCatalogDocument,
    requirement_context: Option<&RequirementContextDocument>,
) -> Result<String, String> {
    let formula_document = FormulaDocument::from_formula(formula)
        .map_err(|error| format!("canonical formula identity failed: {error}"))?;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct FormulaRequest<'a, T> {
        #[serde(flatten)]
        operation: &'a T,
        formula: tl_syntax::SemanticFormulaDocument<'a>,
    }
    contextual_request_sha256(
        domain,
        &FormulaRequest {
            operation,
            formula: formula_document.semantic_view(),
        },
        catalog_document,
        requirement_context,
    )
    .map_err(|error| error.to_string())
}

/// Returns a result identity without recursively including its own digest field.
pub(crate) fn contextual_result_sha256<T: Serialize>(
    domain: &str,
    report: &T,
) -> Result<String, serde_json::Error> {
    let mut value = serde_json::to_value(report)?;
    if let Some(object) = value.as_object_mut() {
        object.remove("resultSha256");
    }
    domain_sha256(domain, &value)
}

fn sha256_json<T: Serialize>(value: &T) -> Result<String, serde_json::Error> {
    Ok(hex(Sha256::digest(serde_json::to_vec(value)?)))
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    bytes
        .as_ref()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[cfg(test)]
mod tests {
    use tl_syntax::{
        FormulaDocument, Node, NodeId, NodeKind, OwnedSignalDeclaration, PropositionBinding,
        RequirementContextDocument, SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
        SourceSpan,
    };

    use super::{
        bind_formula, catalog_sha256, contextual_request_sha256, contextual_result_sha256,
        domain_sha256, ContextualBindingError,
    };

    fn formula(proposition: u32) -> FormulaDocument {
        FormulaDocument::new(
            SemanticProfile::ClosedTraceV1,
            NodeId(0),
            vec![Node::new(NodeKind::Proposition {
                proposition: tl_syntax::PropositionId(proposition),
            })],
        )
        .unwrap()
    }

    fn catalog(proposition: u32, name: &str) -> SignalCatalogDocument {
        SignalCatalogDocument::new(
            vec![OwnedSignalDeclaration::new(
                SignalId(1),
                name.to_owned(),
                SignalDomain::Boolean,
            )],
            vec![PropositionBinding::new(
                tl_syntax::PropositionId(proposition),
                SignalId(1),
            )],
        )
        .unwrap()
    }

    // Trace: TC-025, FR-007-AC-1
    #[test]
    fn binding_refuses_the_exact_missing_proposition_before_work() {
        let document = formula(7);
        let formula = document.validate().unwrap();
        assert_eq!(
            bind_formula(formula, &catalog(8, "request_ready")),
            Err(ContextualBindingError::MissingProposition {
                proposition: tl_syntax::PropositionId(7),
            })
        );
    }

    // Trace: TC-028, FR-007-AC-4
    #[test]
    fn catalog_and_domain_identities_detect_distinct_inputs() {
        let first = catalog(7, "request_ready");
        let second = catalog(7, "response_ready");
        assert_ne!(
            catalog_sha256(&first).unwrap(),
            catalog_sha256(&second).unwrap()
        );
        assert_ne!(
            domain_sha256("tl-mltl.contextual-evaluation/v2", &first).unwrap(),
            domain_sha256("tl-mltl.contextual-horizon/v2", &first).unwrap(),
        );
    }

    // Trace: TC-028, FR-007-AC-4
    #[test]
    fn request_identity_binds_context_presence_and_value() {
        let catalog = catalog(7, "request_ready");
        let context = RequirementContextDocument::new(
            "agent-ix/tl-mltl/FR-007".to_owned(),
            "1".to_owned(),
            "AC-1".to_owned(),
            "contextual-request".to_owned(),
            SourceSpan::new(4, 9).unwrap(),
        )
        .unwrap();
        let operation = serde_json::json!({"formulaId": "request"});
        let absent = contextual_request_sha256(
            "tl-mltl.contextual-evaluation/v2",
            &operation,
            &catalog,
            None,
        )
        .unwrap();
        let present = contextual_request_sha256(
            "tl-mltl.contextual-evaluation/v2",
            &operation,
            &catalog,
            Some(&context),
        )
        .unwrap();
        assert_ne!(absent, present);
    }

    // Trace: TC-028, FR-007-AC-4
    #[test]
    fn result_identity_ignores_only_its_own_field() {
        let first = serde_json::json!({"resultSha256": "old", "outcome": true});
        let second = serde_json::json!({"resultSha256": "new", "outcome": true});
        let changed = serde_json::json!({"resultSha256": "old", "outcome": false});
        let first = contextual_result_sha256("tl-mltl.contextual-evaluation/v2", &first).unwrap();
        assert_eq!(
            first,
            contextual_result_sha256("tl-mltl.contextual-evaluation/v2", &second).unwrap()
        );
        assert_ne!(
            first,
            contextual_result_sha256("tl-mltl.contextual-evaluation/v2", &changed).unwrap()
        );
    }
}
