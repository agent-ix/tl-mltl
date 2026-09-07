//! Shared typed-catalog binding and identity helpers for contextual results.
//!
//! This module owns only tl-mltl's operation-level checks and digests. Signal
//! catalogs and requirement contexts remain the exact shared tl-syntax documents.

use serde::Serialize;
use sha2::{Digest, Sha256};
use tl_syntax::{Formula, FormulaBindingError, PropositionId, SignalCatalogDocument};

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
        SemanticProfile, SignalCatalogDocument, SignalDomain, SignalId,
    };

    use super::{bind_formula, catalog_sha256, domain_sha256, ContextualBindingError};

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
}
