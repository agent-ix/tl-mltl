//! Deterministic finite-trace MLTL reference semantics and monitor interoperability.
//!
//! The library consumes structurally validated [`tl_syntax::Formula`] values.
//! Closed traces use an all-false valuation after the declared end; open
//! prefixes preserve unknown future observations as [`TruthValue::Pending`].

macro_rules! deserialize_contextual_record {
    ($record:ident { $($field:ident: $field_type:ty),+ $(,)? }) => {
        impl<'de> serde::Deserialize<'de> for $record {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                #[derive(serde::Deserialize)]
                #[serde(rename_all = "camelCase", deny_unknown_fields)]
                struct Wire {
                    $( $field: $field_type, )+
                    requirement_context: crate::context::RequiredContext,
                }
                let wire = Wire::deserialize(deserializer)?;
                Ok(Self {
                    $( $field: wire.$field, )+
                    requirement_context: wire.requirement_context.0,
                })
            }
        }
    };
}

mod context;
mod differential;
mod evaluate;
mod horizon;
mod mapping;
mod wire;

pub(crate) const MAX_RECURSION_DEPTH: u32 = 512;

pub use context::ContextualBindingError;
pub use differential::{
    compare_external, compare_external_with_context, ComparisonStatus, ContextualComparisonError,
    ContextualComparisonStatus, ContextualDifferentialReport, ContextualDifferentialSchemaVersion,
    ContextualExternalVerdict, ContextualExternalVerdictSchemaVersion, DifferentialReport,
    ExternalStatus, ExternalVerdict, ToolIdentity,
};
pub use evaluate::{
    evaluate_closed, evaluate_closed_at, evaluate_closed_with_context, evaluate_prefix,
    evaluate_prefix_at, evaluate_prefix_with_context, ContextualEvaluationError,
    ContextualEvaluationReport, ContextualEvaluationSchemaVersion, EvaluationError,
    EvaluationLimits, EvaluationReport, TruthValue,
};
pub use horizon::{
    analyze_horizon, analyze_horizon_with_context, ContextualHorizonError, ContextualHorizonReport,
    ContextualHorizonSchemaVersion, HorizonError, HorizonReport,
};
pub use mapping::{
    map_to_c2po, map_to_c2po_with_context, ContextualMappingManifest,
    ContextualMappingSchemaVersion, MappingError, MappingManifest, MappingSourceIdentity,
    MappingSourceState,
};
pub use wire::{
    CommandDocument, CommandSchemaVersion, Operation, TraceDocument, TraceSchemaVersion,
};

/// Exact tl-syntax source revision this crate is compiled against.
///
/// This is the dependency identity `Cargo.toml` resolves and the value the C2PO
/// mapping manifest reports as `syntaxRevision`. It is a different fact from
/// [`TL_SYNTAX_CORPUS_BASIS`], which names the revision whose corpus bytes were
/// copied into `corpus/tl-syntax-v1`; the compiled pin moved onto tl-syntax
/// `main` and the retained corpus bytes did not move with it.
pub const TL_SYNTAX_REVISION: &str = "1b3c4026ff9567491e87a19fdf2793d3b2e76160";

/// Exact tl-syntax revision whose shared corpus bytes are retained here.
///
/// `corpus/tl-syntax-v1` is a byte-identical copy taken at this revision and
/// verified by its own `SHA256SUMS`. It is deliberately not updated when the
/// compiled dependency advances: the retained bytes are what they are, and
/// restating the newer revision would claim a copy nobody made.
pub const TL_SYNTAX_CORPUS_BASIS: &str = "740182f13b84858008d6f176f75136737d405c1b";

/// Shared temporal corpus identity consumed by this crate.
pub const TL_SYNTAX_CORPUS_REVISION: &str = "tl-syntax-corpus/v1";

/// Merged PGM-01 policy revision governing evidence and qualification boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
