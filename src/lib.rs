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

pub(crate) const MAX_RECURSION_DEPTH: u32 = 512;

pub use differential::{
    compare_external, ComparisonStatus, DifferentialReport, ExternalStatus, ExternalVerdict,
    ToolIdentity,
};
pub use evaluate::{
    evaluate_closed, evaluate_closed_at, evaluate_prefix, evaluate_prefix_at, EvaluationError,
    EvaluationLimits, EvaluationReport, TruthValue,
};
pub use horizon::{analyze_horizon, HorizonError, HorizonReport};
pub use mapping::{
    map_to_c2po, MappingError, MappingManifest, MappingSourceIdentity, MappingSourceState,
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
pub const TL_SYNTAX_REVISION: &str = "953ee825e5060335b4c79682f5f41a78c5a1bfae";

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
