//! Future-time finite-trace evaluation and horizon analysis.

mod evaluate;
pub(crate) mod horizon;

pub use evaluate::{
    evaluate_closed, evaluate_closed_at, evaluate_closed_at_with_stats,
    evaluate_closed_with_context, evaluate_prefix, evaluate_prefix_at,
    evaluate_prefix_at_with_stats, evaluate_prefix_with_context, ContextualEvaluationError,
    ContextualEvaluationReport, ContextualEvaluationSchemaVersion, EvaluationError,
    EvaluationLimits, EvaluationReport, EvaluationStats, TruthValue,
};
pub use horizon::{
    analyze_horizon, analyze_horizon_with_context, ContextualHorizonError, ContextualHorizonReport,
    ContextualHorizonSchemaVersion, HorizonError, HorizonReport,
};
