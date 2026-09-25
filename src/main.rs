use std::{env, fs::File, io::Read, process::ExitCode};

use tl_mltl::{
    analyze_horizon, evaluate_closed, evaluate_prefix, map_to_c2po, CommandDocument,
    EvaluationLimits, Operation,
};
use tl_syntax::SemanticProfile;

// The compatibility CLI shares the public owner's immutable byte ceiling.
const MAX_REQUEST_BYTES: usize = tl_mltl::wire::OwnerLimits::owner_max().max_input_bytes;

fn read_bounded_request(mut reader: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    let limit = u64::try_from(MAX_REQUEST_BYTES)
        .map_err(|_| "owner request byte limit exceeds u64".to_owned())?;
    reader
        .by_ref()
        .take(limit + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| format!("read request: {error}"))?;
    if bytes.len() > MAX_REQUEST_BYTES {
        return Err(format!(
            "{}: request exceeds {MAX_REQUEST_BYTES}-byte owner limit",
            tl_mltl::wire::OwnerReadErrorCode::ResourceIncomplete.as_str()
        ));
    }
    Ok(bytes)
}

fn read_request(path: &str) -> Result<Vec<u8>, String> {
    if path == "-" {
        read_bounded_request(std::io::stdin().lock())
    } else {
        let file = File::open(path).map_err(|error| format!("read {path}: {error}"))?;
        read_bounded_request(file)
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
                SemanticProfile::OriginCompleteHistoryV1 => {
                    return Err(
                        "origin-complete history formulas require the typed past-evaluation API"
                            .to_owned(),
                    );
                }
                SemanticProfile::InfiniteTraceV1 => {
                    return Err(
                        "infinite-trace formulas require the opt-in provider API".to_owned()
                    );
                }
            }
            .map_err(|error| format!("evaluate formula: {error}"))?;
            serde_json::to_value(report)
        }
        Operation::MapC2po => {
            let formula_bytes = serde_json::to_vec(&request.formula)
                .map_err(|error| format!("serialize formula: {error}"))?;
            let source = tl_mltl::MappingSourceIdentity {
                revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
                state: tl_mltl::MappingSourceState::parse(env!("TL_MLTL_SOURCE_STATE"))
                    .expect("build script emits a closed source-state value"),
            };
            serde_json::to_value(
                map_to_c2po(
                    formula,
                    request.formula_id,
                    &formula_bytes,
                    source,
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

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::{read_bounded_request, MAX_REQUEST_BYTES};

    // Trace: TL-229; FR-046-AC-1
    #[test]
    fn request_reader_accepts_exact_cap_and_refuses_cap_plus_one() {
        let exact = vec![b' '; MAX_REQUEST_BYTES];
        assert_eq!(
            read_bounded_request(Cursor::new(&exact))
                .expect("exact owner ceiling")
                .len(),
            MAX_REQUEST_BYTES
        );
        let over = vec![b' '; MAX_REQUEST_BYTES + 1];
        assert!(read_bounded_request(Cursor::new(&over))
            .expect_err("over owner ceiling")
            .contains("TL-OWNER-RESOURCE-INCOMPLETE"));
    }

    // Trace: TL-229; FR-046-AC-1
    #[test]
    fn request_reader_admits_schema_maximum_trace_positions() {
        let mut request = br#"{"schemaVersion":"tl-mltl.command/v1","operation":"analyze","formulaId":"large-trace","formula":{"schema_version":"tl-syntax.formula/v1","semantic_profile":"mltl.closed-trace/v1","root":0,"nodes":[{"kind":"true"}]},"trace":{"schemaVersion":"tl-mltl.trace/v1","traceId":"large-trace","closed":true,"instants":["#.to_vec();
        for position in 0..1_000_000 {
            if position != 0 {
                request.push(b',');
            }
            request.extend_from_slice(b"[]");
        }
        request.extend_from_slice(b"]}}");
        let admitted = read_bounded_request(Cursor::new(request)).expect("million-position trace");
        let command: tl_mltl::CommandDocument =
            serde_json::from_slice(&admitted).expect("valid maximum-position command");
        assert_eq!(command.trace.expect("trace").instants.len(), 1_000_000);
    }
}
