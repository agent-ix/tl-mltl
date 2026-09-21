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

pub mod clock;
mod context;
mod differential;
pub mod future;
pub mod mapping;
pub mod past;
pub mod wire;

pub(crate) use future::horizon;

pub(crate) const MAX_RECURSION_DEPTH: u32 = 512;

pub use context::ContextualBindingError;
pub use differential::{
    compare_external, compare_external_with_context, ComparisonStatus, ContextualComparisonError,
    ContextualComparisonStatus, ContextualDifferentialReport, ContextualDifferentialSchemaVersion,
    ContextualExternalVerdict, ContextualExternalVerdictSchemaVersion, DifferentialReport,
    ExternalStatus, ExternalVerdict, ToolIdentity,
};
pub use future::{
    analyze_horizon, analyze_horizon_with_context, ContextualHorizonError, ContextualHorizonReport,
    ContextualHorizonSchemaVersion, HorizonError, HorizonReport,
};
pub use future::{
    evaluate_closed, evaluate_closed_at, evaluate_closed_with_context, evaluate_prefix,
    evaluate_prefix_at, evaluate_prefix_with_context, ContextualEvaluationError,
    ContextualEvaluationReport, ContextualEvaluationSchemaVersion, EvaluationError,
    EvaluationLimits, EvaluationReport, TruthValue,
};
pub use mapping::{
    map_to_c2po, map_to_c2po_with_context, ContextualMappingManifest,
    ContextualMappingSchemaVersion, MappingError, MappingManifest, MappingSourceIdentity,
    MappingSourceState,
};
pub use past::{
    analyze_required_history, evaluate_past, fixed_sample_instant, ClockBinding, ClockError,
    ClockSample, ExactNumber, ExactNumberError, HistoryError, HistoryRequirementError,
    HistoryRequirementReport, HistoryRequirementSchemaVersion, HistoryRequirementValidationError,
    OwnerHistoryState, PastEvaluationError, PastEvaluationLimits, PastEvaluationRelationInput,
    PastEvaluationReport, PastEvaluationSchemaVersion, PastEvaluationStats, PastEvaluatorIdentity,
    PastResultFinality, PastResultReference, PastResultRelation, PastResultRelationKind,
    PastResultValidationError, PositionHistoryDocument, PositionHistoryReference,
    PositionHistorySchemaVersion, PositionHistorySource, PositionObservation, UnsupportedClockKind,
    HISTORY_REQUIREMENT_V1, PAST_EVALUATION_V1, PAST_EVALUATOR_V1, POSITION_HISTORY_V1,
};
pub use wire::{
    CommandDocument, CommandSchemaVersion, Operation, TraceDocument, TraceSchemaVersion,
};

/// Exact tl-syntax source revision this crate is compiled against.
///
/// This is the dependency identity `Cargo.toml` resolves and the value the C2PO
/// mapping manifest reports as `syntaxRevision`. The shared temporal corpus and
/// the future-operator corpus are both read straight out of this same compiled
/// dependency via `tl_syntax::CORPUS_DIR`, so there is no separate basis
/// revision to track for either: reading through the dependency means each
/// tracks whatever this revision names.
pub const TL_SYNTAX_REVISION: &str = "d52d89549b0a6c0c429261bab912cd5396c4a19e";

/// Exact Quire Observation owner revision whose constructor-private assertion
/// views are accepted by the temporal request boundary.
pub const QUIRE_OBSERVATION_REVISION: &str = "924006300f45b38483be1cbdf99b68f899b7d368";

/// Shared temporal corpus identity consumed by this crate.
pub const TL_SYNTAX_CORPUS_REVISION: &str = "tl-syntax-corpus/v1";

/// Merged PGM-01 policy revision governing evidence and qualification boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
