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

pub mod campaign;
pub mod ci_guard;
pub mod clock;
mod context;
mod differential;
pub mod future;
#[cfg(feature = "infinite-trace")]
pub mod infinite;
pub mod mapping;
pub mod past;
pub mod r2u2_future;
pub mod r2u2_live;
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
    evaluate_closed, evaluate_closed_at, evaluate_closed_at_with_stats,
    evaluate_closed_with_context, evaluate_prefix, evaluate_prefix_at,
    evaluate_prefix_at_with_stats, evaluate_prefix_with_context, ContextualEvaluationError,
    ContextualEvaluationReport, ContextualEvaluationSchemaVersion, EvaluationError,
    EvaluationLimits, EvaluationReport, EvaluationStats, TruthValue,
};
pub use mapping::{
    map_past_to_c2po, map_to_c2po, map_to_c2po_with_context, ContextualMappingManifest,
    ContextualMappingSchemaVersion, MappingError, MappingManifest, MappingSourceIdentity,
    MappingSourceState, PastMappingError, PastMappingManifest, TargetOriginContract,
};
pub use past::{
    analyze_required_history, evaluate_past, evaluate_past_with_stats, fixed_sample_instant,
    ClockBinding, ClockError, ClockSample, ExactNumber, ExactNumberError, HistoryError,
    HistoryRequirementError, HistoryRequirementReport, HistoryRequirementSchemaVersion,
    HistoryRequirementValidationError, OwnerHistoryState, PastEvaluationError,
    PastEvaluationLimits, PastEvaluationRelationInput, PastEvaluationReport,
    PastEvaluationSchemaVersion, PastEvaluationStats, PastEvaluatorIdentity, PastResultFinality,
    PastResultReference, PastResultRelation, PastResultRelationKind, PastResultValidationError,
    PositionHistoryDocument, PositionHistoryReference, PositionHistorySchemaVersion,
    PositionHistorySource, PositionObservation, UnsupportedClockKind, HISTORY_REQUIREMENT_V1,
    PAST_EVALUATION_V1, PAST_EVALUATOR_V1, POSITION_HISTORY_V1,
};
pub use wire::{
    CommandDocument, CommandSchemaVersion, Operation, TraceDocument, TraceSchemaVersion,
};

/// Shared temporal corpus identity consumed by this crate.
pub const TL_SYNTAX_CORPUS_REVISION: &str = "tl-syntax-corpus/v1";

/// Exact tl-mltl source revision this crate was built from, as set by
/// `build.rs` via `TL_MLTL_SOURCE_REVISION` (the environment value when
/// building outside a git checkout, otherwise the checked-out `HEAD`).
///
/// Call sites within this crate already read this through `env!` directly
/// because `build.rs` runs for this crate; this constant exists for
/// downstream crates (such as a future `quire-mltl`) that cannot invoke
/// `env!` for a value only this crate's build script sets.
pub const TL_MLTL_SOURCE_REVISION: &str = env!("TL_MLTL_SOURCE_REVISION");

#[cfg(test)]
mod limit_register {
    //! NFR-001 Limit Register rows whose limits have no public input.

    use tl_syntax::{Formula, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

    use crate::{
        evaluate_closed_at, map_to_c2po, ClockBinding, EvaluationError, EvaluationLimits,
        MappingError, MappingSourceIdentity, MappingSourceState, PastEvaluationError,
        PastEvaluationLimits, PastEvaluationRelationInput, PositionHistoryDocument,
        PositionObservation, MAX_RECURSION_DEPTH,
    };

    /// A proposition under `depth` negations; its deepest node is at `depth`.
    fn negations(depth: u32) -> Vec<Node> {
        let mut nodes = vec![Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        })];
        for index in 0..depth {
            nodes.push(Node::new(NodeKind::Not {
                operand: NodeId(index),
            }));
        }
        nodes
    }

    fn formula(profile: SemanticProfile, nodes: &[Node]) -> Formula<'_> {
        Formula::new(
            profile,
            NodeId(u32::try_from(nodes.len() - 1).unwrap()),
            nodes,
        )
        .unwrap()
    }

    // Trace: TC-175, NFR-001-AC-3
    #[test]
    fn recursion_ceiling_accepts_the_ceiling_and_refuses_one_past_in_every_evaluator() {
        // Debug frames at the ceiling exceed the default test-thread stack.
        std::thread::Builder::new()
            .stack_size(256 * 1024 * 1024)
            .spawn(recursion_ceiling_in_every_evaluator)
            .unwrap()
            .join()
            .unwrap();
    }

    fn recursion_ceiling_in_every_evaluator() {
        let ceiling = MAX_RECURSION_DEPTH;
        assert_eq!(ceiling, 512);
        let at = negations(ceiling);
        let past = negations(ceiling + 1);

        let trace = vec![vec![PropositionId(0)]];
        let eval = |nodes: &[Node]| {
            evaluate_closed_at(
                formula(SemanticProfile::ClosedTraceV1, nodes),
                "f",
                &trace,
                "t",
                0,
                EvaluationLimits::default(),
            )
        };
        eval(&at).unwrap();
        assert_eq!(
            eval(&past),
            Err(EvaluationError::RecursionDepthExceeded { limit: ceiling })
        );

        let history = PositionHistoryDocument::new(
            "h",
            1,
            0,
            0,
            Some(ClockBinding::EventPosition),
            vec![PositionObservation::new(0, vec![PropositionId(0)], None)],
        )
        .unwrap();
        let past_eval = |nodes: &[Node]| {
            crate::evaluate_past(
                formula(SemanticProfile::OriginCompleteHistoryV1, nodes),
                "f",
                &history,
                0,
                "m",
                1,
                PastEvaluationRelationInput::Original,
                PastEvaluationLimits::default(),
            )
        };
        past_eval(&at).unwrap();
        assert_eq!(
            past_eval(&past).unwrap_err(),
            PastEvaluationError::RecursionDepthExceeded { limit: ceiling }
        );

        let map = |nodes: &[Node]| {
            map_to_c2po(
                formula(SemanticProfile::OnlinePrefixV1, nodes),
                "f",
                b"{}",
                MappingSourceIdentity {
                    revision: "r".to_owned(),
                    state: MappingSourceState::Clean,
                },
                None,
                u64::MAX,
            )
        };
        map(&at).unwrap();
        assert_eq!(
            map(&past).unwrap_err(),
            MappingError::RecursionDepthExceeded { limit: ceiling }
        );
    }
}
