//! Numeric signal values: the strict numeric trace document (FR-053), exact
//! predicate evaluation by reduction to the existing evaluators (FR-054,
//! FR-055) and, in later modules, contracts and export.
//!
//! No floating-point type appears in this module tree (NFR-017).
//!
//! UNBUILT, UNTESTED: written without a compiler (TL-DECISIONS-2026-11-23).

pub mod evaluate;
pub mod model;
pub mod trace;

pub use evaluate::{
    evaluate_numeric, EmbeddedReport, NumericEvaluationError, NumericEvaluationLimits,
    NumericEvaluationReport, NumericSection, PredicateCell, NUMERIC_EVALUATION_V1,
};
pub use model::{Comparison, ExprNode, PredicateDefinition, PredicateTable};
pub use trace::{
    DecimalValue, NumericAdmissionCode, NumericInstant, NumericObservation, NumericTraceDocument,
    NumericTraceError, NumericTraceLimits, NumericTraceSchemaVersion, ValidatedNumericTrace,
    MAX_NUMERIC_VALUES, NUMERIC_TRACE_V1, NUMERIC_TRACE_V1_SCHEMA_BYTES,
};
