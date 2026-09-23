#![no_main]

use libfuzzer_sys::fuzz_target;
use tl_mltl::future::{evaluate_closed_at, EvaluationLimits};
use tl_syntax::{
    Formula, FormulaDocument, FormulaSchemaVersion, PropositionId, SemanticProfile,
    SyntaxArtifactLimits,
};

fuzz_target!(|data: &[u8]| {
    let Ok(document) = FormulaDocument::from_json_bytes(data, SyntaxArtifactLimits::default())
    else {
        return;
    };
    if document.schema_version() != FormulaSchemaVersion::V2
        || document.semantic_profile() != SemanticProfile::ClosedTraceV1
    {
        return;
    }
    let formula = Formula::new(
        document.semantic_profile(),
        document.root(),
        document.nodes(),
    )
    .expect("strict syntax reader admitted an invalid formula graph");
    let trace = [
        vec![PropositionId(0)],
        vec![PropositionId(1)],
        vec![PropositionId(0), PropositionId(1)],
    ];
    let limits = EvaluationLimits {
        max_node_evaluations: 10_000,
        max_temporal_span: 64,
        max_recursion_depth: 32,
    };
    for selected_position in 0..trace.len() {
        if let Ok(report) = evaluate_closed_at(
            formula,
            "fuzz-formula",
            &trace,
            "fuzz-trace",
            selected_position as u64,
            limits,
        ) {
            assert_eq!(report.trace_length, trace.len() as u64);
            assert_eq!(report.verdict_time, selected_position as u64);
        }
    }
});
