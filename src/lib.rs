//! Deterministic finite-trace MLTL reference semantics and monitor interoperability.
//!
//! The library consumes structurally validated [`tl_syntax::Formula`] values.
//! Closed traces use an all-false valuation after the declared end; open
//! prefixes preserve unknown future observations as [`TruthValue::Pending`].

mod differential;
mod evaluate;
mod horizon;
mod mapping;
mod wire;

pub use differential::{
    compare_external, ComparisonStatus, DifferentialReport, ExternalStatus, ExternalVerdict,
    ToolIdentity,
};
pub use evaluate::{
    evaluate_closed, evaluate_prefix, EvaluationError, EvaluationLimits, EvaluationReport,
    TruthValue,
};
pub use horizon::{analyze_horizon, HorizonError, HorizonReport};
pub use mapping::{map_to_c2po, MappingError, MappingManifest};
pub use wire::{
    CommandDocument, CommandSchemaVersion, Operation, TraceDocument, TraceSchemaVersion,
};

/// Exact tl-syntax source revision used for cross-repository development.
pub const TL_SYNTAX_REVISION: &str = "5e59a26d71b4b5d79623850cda50010e18a90dad";

/// Shared temporal corpus identity consumed by this crate.
pub const TL_SYNTAX_CORPUS_REVISION: &str = "tl-syntax-corpus/v1";

/// Merged PGM-01 policy revision governing evidence and qualification boundaries.
pub const PGM01_POLICY_REVISION: &str = "7dac9d8c19952412b56a0347387666e2ca81e01d";
