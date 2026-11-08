#![no_main]
#![forbid(unsafe_code)]

use libfuzzer_sys::fuzz_target;
use tl_mltl::wire::{OwnerLimits, ValidatedCommand, ValidatedTrace};
use tl_mltl::{
    analyze_horizon, evaluate_closed, evaluate_prefix, CommandDocument, EvaluationLimits, Operation,
};
use tl_syntax::SemanticProfile;

const MAX_INPUT: usize = 16 * 1024;

// Mirrors the request handling in `src/main.rs` (parse, validate, dispatch),
// minus process I/O and the build-script-dependent C2PO source identity.
fn cli_request(bytes: &[u8]) {
    let Ok(request) = serde_json::from_slice::<CommandDocument>(bytes) else {
        return;
    };
    let Ok(formula) = request.formula.validate() else {
        return;
    };
    match request.operation {
        Operation::Analyze => {
            let _ = analyze_horizon(formula, request.formula_id);
        }
        Operation::Evaluate => {
            let Some(trace) = request.trace else { return };
            let _ = match formula.profile() {
                SemanticProfile::ClosedTraceV1 if trace.closed => evaluate_closed(
                    formula,
                    request.formula_id,
                    &trace.instants,
                    trace.trace_id,
                    EvaluationLimits::default(),
                ),
                SemanticProfile::OnlinePrefixV1 => evaluate_prefix(
                    formula,
                    request.formula_id,
                    &trace.instants,
                    trace.trace_id,
                    trace.closed,
                    EvaluationLimits::default(),
                ),
                _ => return,
            };
        }
        Operation::MapC2po => {}
    }
}

// Trace: TC-090, FR-018-AC-1, FR-018-AC-2
fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT {
        return;
    }
    cli_request(data);

    // Owner readers: typed error or a value whose canonical bytes are the
    // input and which re-reads to itself, under owner maxima and tight limits.
    let tight = OwnerLimits {
        max_input_bytes: 512,
        max_formula_nodes: 8,
        max_positions: 4,
        max_propositions: 4,
        ..OwnerLimits::owner_max()
    };
    for limits in [OwnerLimits::owner_max(), tight] {
        if let Ok(command) = ValidatedCommand::from_json_bytes(data, limits) {
            assert_eq!(command.bytes(), data);
            let again = ValidatedCommand::from_json_bytes(command.bytes(), limits)
                .expect("a validated command re-reads");
            assert_eq!(again.document(), command.document());
        }
        if let Ok(trace) = ValidatedTrace::from_json_bytes(data, limits) {
            assert_eq!(trace.canonical_json_bytes(), data);
            let again = ValidatedTrace::from_json_bytes(trace.bytes(), limits)
                .expect("a validated trace re-reads");
            assert_eq!(again.document(), trace.document());
        }
    }
});
