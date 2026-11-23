use std::{env, fmt::Display, fs, io::Read, process::ExitCode};

use tl_mltl::{analyze_horizon, map_to_c2po, record::evaluate_command, CommandDocument, Operation};

/// Names the stage that refused so every failure renders as `stage: error`.
fn refusal<S: Display, E: Display>(stage: S) -> impl FnOnce(E) -> String {
    move |error| format!("{stage}: {error}")
}

fn build_source_identity() -> Result<tl_mltl::MappingSourceIdentity, String> {
    source_identity(
        env!("TL_MLTL_SOURCE_REVISION"),
        env!("TL_MLTL_SOURCE_STATE"),
    )
}

/// Build-time source identity; a state value outside the closed vocabulary is a
/// typed refusal, never a panic.
fn source_identity(revision: &str, state: &str) -> Result<tl_mltl::MappingSourceIdentity, String> {
    Ok(tl_mltl::MappingSourceIdentity {
        revision: revision.to_owned(),
        state: tl_mltl::MappingSourceState::parse(state)
            .ok_or_else(|| format!("build source state {state:?} is not clean or modified"))?,
    })
}

fn read_request(path: &str) -> Result<Vec<u8>, String> {
    if path == "-" {
        let mut bytes = Vec::new();
        std::io::stdin()
            .read_to_end(&mut bytes)
            .map_err(refusal("read stdin"))?;
        Ok(bytes)
    } else {
        fs::read(path).map_err(refusal(format_args!("read {path}")))
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
        serde_json::from_slice(&bytes).map_err(refusal("parse request"))?;
    let formula = request
        .formula
        .validate()
        .map_err(refusal("validate formula"))?;
    let output = match request.operation {
        Operation::Analyze => serde_json::to_value(
            analyze_horizon(formula, request.formula_id).map_err(refusal("analyze formula"))?,
        ),
        Operation::Evaluate => serde_json::to_value(evaluate_command(&request)?),
        Operation::MapC2po => {
            let formula_bytes =
                serde_json::to_vec(&request.formula).map_err(refusal("serialize formula"))?;
            let source = build_source_identity()?;
            serde_json::to_value(
                map_to_c2po(
                    formula,
                    request.formula_id,
                    &formula_bytes,
                    source,
                    None,
                    100_000,
                )
                .map_err(refusal("map formula"))?,
            )
        }
    }
    .map_err(refusal("serialize result"))?;
    println!(
        "{}",
        serde_json::to_string_pretty(&output).map_err(refusal("serialize result"))?
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
    use super::*;

    // Trace: TC-014, FR-005-AC-1
    #[test]
    fn source_identity_refuses_a_state_outside_the_closed_vocabulary() {
        assert!(source_identity("abc", "clean").is_ok());
        assert!(source_identity("abc", "modified").is_ok());
        assert_eq!(
            source_identity("abc", "dirty").unwrap_err(),
            "build source state \"dirty\" is not clean or modified"
        );
    }
}
