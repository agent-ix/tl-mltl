//! Writes one `tl-mltl.evaluation-record/v1` JSON record for a request file.
//!
//! `cargo run --example evaluation_record -- REQUEST.json [--out RECORD.json]`

use std::{env, fs, process::ExitCode};

fn run() -> Result<(), String> {
    let usage = "usage: evaluation_record REQUEST.json [--out RECORD.json]";
    let mut arguments = env::args().skip(1);
    let input_path = arguments.next().ok_or_else(|| usage.to_owned())?;
    let output_path = match arguments.next().as_deref() {
        None => None,
        Some("--out") => Some(arguments.next().ok_or_else(|| usage.to_owned())?),
        Some(_) => return Err(usage.to_owned()),
    };
    if arguments.next().is_some() {
        return Err(usage.to_owned());
    }
    let input = fs::read(&input_path).map_err(|error| format!("read {input_path}: {error}"))?;
    let record = tl_mltl::record::build_record(&input)?;
    let text = serde_json::to_string_pretty(&record)
        .map_err(|error| format!("serialize record: {error}"))?;
    match output_path {
        Some(path) => fs::write(&path, format!("{text}\n"))
            .map_err(|error| format!("write {path}: {error}"))?,
        None => println!("{text}"),
    }
    Ok(())
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("evaluation_record: {error}");
            ExitCode::from(2)
        }
    }
}
