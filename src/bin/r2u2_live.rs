//! Live R2U2 differential driver (TL-217).
//!
//! `r2u2_live run --out DIR [--r2u2-root DIR]` generates the past grid, runs a
//! pinned R2U2 checkout over its admitted cases and writes `DIR/report.json`.
//! `R2U2_ROOT` is read when the flag is absent. With no usable R2U2 it still
//! writes the inputs and a `not_run` report and exits 2.
//! `r2u2_live replay --out DIR` recomputes the comparison from the recorded
//! `DIR/outputs/*/r2u2.stdout` without running R2U2.

use std::{env, fs, path::PathBuf, process::ExitCode};

use tl_mltl::r2u2_live::{run, Report, Source};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let flag = |name: &str| {
        args.iter()
            .position(|a| a == name)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    let out = PathBuf::from(flag("--out").unwrap_or_else(|| "target/r2u2".to_owned()));
    let root = flag("--r2u2-root").or_else(|| env::var("R2U2_ROOT").ok());
    let repo = PathBuf::from(".");
    let report = match args.first().map(String::as_str) {
        Some("run") => match &root {
            Some(r) => run(&repo, &Source::Live(&PathBuf::from(r)), &out),
            None => run(
                &repo,
                &Source::None("no R2U2 checkout: pass --r2u2-root or set R2U2_ROOT".to_owned()),
                &out,
            ),
        },
        Some("replay") => run(&repo, &Source::Recorded, &out),
        _ => Err("usage: r2u2_live <run [--r2u2-root DIR] | replay> [--out DIR]".to_owned()),
    };
    match report.and_then(|r| write(&r, &out)) {
        Ok(r) => match r.outcome.as_str() {
            "pass" => ExitCode::SUCCESS,
            "not_run" => ExitCode::from(2),
            _ => ExitCode::FAILURE,
        },
        Err(e) => {
            eprintln!("r2u2_live: {e}");
            ExitCode::FAILURE
        }
    }
}

fn write(r: &Report, out: &std::path::Path) -> Result<Report, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(r).map_err(|e| e.to_string())?;
    fs::write(out.join("report.json"), bytes).map_err(|e| e.to_string())?;
    println!(
        "mode={} outcome={} summary={:?}",
        r.mode, r.outcome, r.summary
    );
    if let Some(d) = &r.tool_detail {
        println!("tool: {d}");
    }
    Ok(r.clone())
}
