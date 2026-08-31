use std::{env, fs, io::Read, process::ExitCode};

use tl_mltl::{
    analyze_horizon, evaluate_closed, evaluate_prefix, map_to_c2po, CommandDocument,
    EvaluationLimits, Operation,
};
use tl_syntax::SemanticProfile;

fn read_request(path: &str) -> Result<Vec<u8>, String> {
    if path == "-" {
        let mut bytes = Vec::new();
        std::io::stdin()
            .read_to_end(&mut bytes)
            .map_err(|error| format!("read stdin: {error}"))?;
        Ok(bytes)
    } else {
        fs::read(path).map_err(|error| format!("read {path}: {error}"))
    }
}

fn run() -> Result<(), String> {
    let mut arguments = env::args().skip(1);
    let path = arguments
        .next()
        .ok_or_else(|| "usage: tl-mltl REQUEST.json (or - for stdin)".to_owned())?;
    if arguments.next().is_some() {
        return Err("usage: tl-mltl REQUEST.json (or - for stdin)".to_owned());
    }
    let bytes = read_request(&path)?;
    let request: CommandDocument =
        serde_json::from_slice(&bytes).map_err(|error| format!("parse request: {error}"))?;
    let formula = request
        .formula
        .validate()
        .map_err(|error| format!("validate formula: {error}"))?;
    let output = match request.operation {
        Operation::Analyze => serde_json::to_value(
            analyze_horizon(formula, request.formula_id)
                .map_err(|error| format!("analyze formula: {error}"))?,
        ),
        Operation::Evaluate => {
            let trace = request
                .trace
                .ok_or_else(|| "evaluate operation requires trace".to_owned())?;
            let report = match formula.profile() {
                SemanticProfile::ClosedTraceV1 => {
                    if !trace.closed {
                        return Err(
                            "closed-trace formula requires a closed trace document".to_owned()
                        );
                    }
                    evaluate_closed(
                        formula,
                        request.formula_id,
                        &trace.instants,
                        trace.trace_id,
                        EvaluationLimits::default(),
                    )
                }
                SemanticProfile::OnlinePrefixV1 => evaluate_prefix(
                    formula,
                    request.formula_id,
                    &trace.instants,
                    trace.trace_id,
                    trace.closed,
                    EvaluationLimits::default(),
                ),
            }
            .map_err(|error| format!("evaluate formula: {error}"))?;
            serde_json::to_value(report)
        }
        Operation::MapC2po => {
            let formula_bytes = serde_json::to_vec(&request.formula)
                .map_err(|error| format!("serialize formula: {error}"))?;
            let source_revision = env!("TL_MLTL_SOURCE_REVISION");
            let source_state = env!("TL_MLTL_SOURCE_STATE");
            serde_json::to_value(
                map_to_c2po(
                    formula,
                    request.formula_id,
                    &formula_bytes,
                    source_revision,
                    source_state,
                    None,
                    100_000,
                )
                .map_err(|error| format!("map formula: {error}"))?,
            )
        }
    }
    .map_err(|error| format!("serialize result: {error}"))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&output)
            .map_err(|error| format!("serialize result: {error}"))?
    );
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("tl-mltl: {error}");
            ExitCode::from(2)
        }
    }
}
