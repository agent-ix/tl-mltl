//! Runs Kani harnesses by exact qualified name and fails unless each one
//! verified with every cover property satisfied: `kani_gate --list FILE` runs
//! every name in FILE, `kani_gate HARNESS` runs one.
//!
//! Kani itself exits 0 when a cover is unsatisfied (a vacuous proof), so the
//! gate reads the result lines. When `TL_MLTL_EVIDENCE_DIR` is set (a campaign
//! run), a per-proof record with its own outcome state is written there for the
//! campaign report. `TL_MLTL_KANI_TIMED_OUT=1` records a timeout; a timed-out
//! run is never recorded as verified.

use std::{
    env, fs,
    process::{Command, ExitCode},
    time::Instant,
};

use tl_mltl::kani_record::{
    build_record, harness_bounds, KaniRecord, ProofRun, KANI_RECORD_SCHEMA,
};

/// The source file that holds the harness with this qualified name.
fn source_file(harness: &str) -> &'static str {
    if harness.starts_with("wire::") {
        "src/wire/kani_wire.rs"
    } else if harness.starts_with("future::horizon::") {
        "src/future/horizon.rs"
    } else {
        "src/future/kani_evaluate.rs"
    }
}

fn version(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_owned())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| "unavailable".to_owned())
}

/// Judge a Kani log: `Ok(cover line)` only for a successful verification whose
/// cover properties are all satisfied and non-empty.
fn judge(harness: &str, log: &str) -> Result<String, String> {
    if !log.contains("VERIFICATION:- SUCCESSFUL") {
        return Err(format!("{harness} has no successful verification"));
    }
    let cover = log
        .lines()
        .find(|l| l.contains("** ") && l.contains(" cover properties satisfied"))
        .ok_or_else(|| format!("{harness} has no cover results (vacuous)"))?;
    let words: Vec<&str> = cover.split_whitespace().collect();
    // `** N of M cover properties satisfied`
    let ok = matches!(words.as_slice(), ["**", n, "of", m, ..] if n == m && *m != "0");
    if ok {
        Ok(cover.to_owned())
    } else {
        Err(format!("{harness} has an unsatisfied cover: {cover}"))
    }
}

fn record(harness: &str, log: &str, status: Option<i32>, seconds: u64, dir: &str) {
    let source = fs::read_to_string(source_file(harness)).unwrap_or_default();
    let name = harness.rsplit("::").next().unwrap_or(harness);
    // `cargo kani --version` prints the Kani line, then the CBMC it drives.
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
        exit_code: status,
        log,
        timed_out: env::var("TL_MLTL_KANI_TIMED_OUT").is_ok_and(|v| v == "1"),
        kani_version: &kani_version,
        cbmc_version: &cbmc_version,
        elapsed_seconds: seconds,
        bounds: &bounds,
    };
    let doc = KaniRecord {
        schema_version: KANI_RECORD_SCHEMA.to_owned(),
        proofs: vec![build_record(&run)],
    };
    let file: String = harness
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '_' { c } else { '-' })
        .collect();
    let written = serde_json::to_vec_pretty(&doc)
        .map_err(|e| e.to_string())
        .and_then(|b| fs::write(format!("{dir}/{file}.json"), b).map_err(|e| e.to_string()));
    if let Err(e) = written {
        eprintln!("kani_gate: record not written: {e}");
    }
}

fn run_one(harness: &str) -> Result<(), String> {
    let started = Instant::now();
    let output = Command::new("cargo")
        .args(["kani", "--lib", "-Z", "stubbing", "--output-format", "terse"])
        .args(["--harness", harness, "--exact"])
        .output()
        .map_err(|e| format!("cargo kani: {e}"))?;
    let mut log = String::from_utf8_lossy(&output.stdout).into_owned();
    log.push_str(&String::from_utf8_lossy(&output.stderr));
    if let Ok(dir) = env::var("TL_MLTL_EVIDENCE_DIR") {
        if !dir.is_empty() {
            record(harness, &log, output.status.code(), started.elapsed().as_secs(), &dir);
        }
    }
    if !output.status.success() {
        print!("{log}");
        return Err(format!("{harness} did not verify"));
    }
    print!("{log}");
    judge(harness, &log).map(|_| ())
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();
    let harnesses: Vec<String> = match args.as_slice() {
        [flag, file] if flag == "--list" => match fs::read_to_string(file) {
            Ok(text) => text
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .map(str::to_owned)
                .collect(),
            Err(e) => {
                eprintln!("kani_gate: {file}: {e}");
                return ExitCode::from(2);
            }
        },
        [harness] if !harness.starts_with("--") => vec![harness.clone()],
        _ => {
            eprintln!("usage: kani_gate --list FILE | kani_gate HARNESS");
            return ExitCode::from(2);
        }
    };
    for harness in &harnesses {
        if let Err(e) = run_one(harness) {
            eprintln!("kani_gate: {e}");
            return ExitCode::FAILURE;
        }
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK: &str = "VERIFICATION:- SUCCESSFUL\n ** 2 of 2 cover properties satisfied\n";

    #[test]
    fn a_fully_covered_success_passes() {
        assert!(judge("h", OK).is_ok());
    }

    #[test]
    fn a_vacuous_or_partly_covered_proof_fails() {
        assert!(judge("h", "VERIFICATION:- SUCCESSFUL\n").is_err());
        assert!(judge("h", "VERIFICATION:- SUCCESSFUL\n ** 1 of 2 cover properties satisfied\n").is_err());
        assert!(judge("h", "VERIFICATION:- SUCCESSFUL\n ** 0 of 0 cover properties satisfied\n").is_err());
        assert!(judge("h", "VERIFICATION:- FAILED\n ** 2 of 2 cover properties satisfied\n").is_err());
    }

    #[test]
    fn harnesses_map_to_their_source_files() {
        assert_eq!(source_file("wire::x"), "src/wire/kani_wire.rs");
        assert_eq!(source_file("future::horizon::x"), "src/future/horizon.rs");
        assert_eq!(source_file("future::evaluate::x"), "src/future/kani_evaluate.rs");
    }
}
