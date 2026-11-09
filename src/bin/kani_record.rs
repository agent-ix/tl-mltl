//! Writes one Kani proof record: `kani_record HARNESS LOG EXIT SECONDS SOURCE OUT`.
//!
//! Called by `scripts/kani-gate.sh` after each harness run. `EXIT` is the
//! `cargo kani` exit code (`none` when killed) and `SOURCE` the file holding
//! the harness. Setting `TL_MLTL_KANI_TIMED_OUT=1` records a timeout.

use std::{
    env, fs,
    process::{Command, ExitCode},
};

use tl_mltl::kani_record::{
    build_record, harness_bounds, KaniRecord, ProofRun, KANI_RECORD_SCHEMA,
};

fn version(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "unavailable".to_owned())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let [harness, log, exit, seconds, source, out] = &args[..] else {
        eprintln!("usage: kani_record HARNESS LOG EXIT SECONDS SOURCE OUT");
        return ExitCode::from(2);
    };
    let log = fs::read_to_string(log).unwrap_or_default();
    let source = fs::read_to_string(source).unwrap_or_default();
    let name = harness.rsplit("::").next().unwrap_or(harness);
    // `cargo kani --version` prints the Kani line, then the CBMC it drives (the
    // standalone `cbmc` on PATH is not the one Kani uses).
    let versions = version("cargo", &["kani", "--version"]);
    let mut lines = versions.lines();
    let kani_version = lines.next().unwrap_or("unavailable").trim().to_owned();
    let cbmc_version = lines
        .find_map(|l| l.trim().strip_prefix("CBMC "))
        .unwrap_or("unavailable")
        .to_owned();
    let bounds = harness_bounds(&source, name).unwrap_or_default();
    let run = ProofRun {
        harness,
        exit_code: exit.parse().ok(),
        log: &log,
        timed_out: env::var("TL_MLTL_KANI_TIMED_OUT").is_ok_and(|v| v == "1"),
        kani_version: &kani_version,
        cbmc_version: &cbmc_version,
        elapsed_seconds: seconds.parse().unwrap_or(0),
        bounds: &bounds,
    };
    let record = KaniRecord {
        schema_version: KANI_RECORD_SCHEMA.to_owned(),
        proofs: vec![build_record(&run)],
    };
    match serde_json::to_vec_pretty(&record)
        .map_err(|e| e.to_string())
        .and_then(|b| fs::write(out, b).map_err(|e| e.to_string()))
    {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("kani_record: {e}");
            ExitCode::FAILURE
        }
    }
}
