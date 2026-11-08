//! Live R2U2 differential driver (TL-217).
//!
//! `r2u2_live run --out DIR [--r2u2-root DIR]` generates the past grid, runs a
//! pinned R2U2 checkout over its admitted cases and writes `DIR/report.json`.
//! `R2U2_ROOT` is read when the flag is absent. With no usable R2U2 it still
//! writes the inputs and a `not_run` report and exits 2.
//! `r2u2_live replay --out DIR` recomputes the comparison from the recorded
//! `DIR/outputs/*/r2u2.stdout` without running R2U2, refuses unless the inputs and
//! outputs match the digests in the live `DIR/report.json`, and writes
//! `DIR/replay-report.json`, never replacing the live report. `r2u2_live probe`
//! exits 0 only for a usable pinned checkout (campaign tool probe).

use std::{env, fs, path::PathBuf, process::ExitCode};

use tl_mltl::r2u2_future::run_future;
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
    let future = flag("--suite").as_deref() == Some("future");
    let go = |source: &Source<'_>| {
        if future {
            run_future(&repo, source, &out)
        } else {
            run(&repo, source, &out)
        }
    };
    let report = match args.first().map(String::as_str) {
        Some("run") => match &root {
            Some(r) => go(&Source::Live(&PathBuf::from(r))),
            None => go(&Source::None(
                "no R2U2 checkout: pass --r2u2-root or set R2U2_ROOT".to_owned(),
            )),
        },
        Some("replay") => go(&Source::Recorded),
        Some("probe") => return probe(&repo, root.as_deref()),
        _ => Err(
            "usage: r2u2_live <run [--r2u2-root DIR] | replay | probe> [--suite past|future] [--out DIR]"
                .to_owned(),
        ),
    };
    // A replay never replaces the live run's report it is bound to.
    let name = if args.first().map(String::as_str) == Some("replay") {
        "replay-report.json"
    } else {
        "report.json"
    };
    match report.and_then(|r| write(&r, &out, name)) {
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

/// Exit 0 with the pinned revision only when `root` is a usable pinned build.
fn probe(repo: &std::path::Path, root: Option<&str>) -> ExitCode {
    let Some(root) = root else {
        eprintln!("r2u2_live: no R2U2 checkout: pass --r2u2-root or set R2U2_ROOT");
        return ExitCode::from(2);
    };
    match tl_mltl::r2u2_live::probe(repo, &PathBuf::from(root)) {
        Ok(o) => {
            println!("r2u2 {}", o.get("r2u2Revision").map_or("?", String::as_str));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("r2u2_live: {e}");
            ExitCode::from(2)
        }
    }
}

fn write(r: &Report, out: &std::path::Path, name: &str) -> Result<Report, String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    let bytes = serde_json::to_vec_pretty(r).map_err(|e| e.to_string())?;
    fs::write(out.join(name), bytes).map_err(|e| e.to_string())?;
    println!(
        "mode={} outcome={} summary={:?}",
        r.mode, r.outcome, r.summary
    );
    if let Some(d) = &r.tool_detail {
        println!("tool: {d}");
    }
    Ok(r.clone())
}
