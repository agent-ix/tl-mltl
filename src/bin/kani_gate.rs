//! Runs Kani harnesses by exact qualified name and fails unless each one
//! verified with every cover property satisfied: `kani_gate --list FILE` runs
//! every name in FILE and reports every failure, `kani_gate HARNESS` runs one.
//!
//! Kani itself exits 0 when a cover is unsatisfied (a vacuous proof), so the
//! gate reads the result lines. When `TL_MLTL_EVIDENCE_DIR` is set (a campaign
//! run), a per-proof record with its own outcome state is written there for the
//! campaign report; a record that cannot be written is reported but does not
//! change the verdict. `TL_MLTL_KANI_TIMED_OUT=1` records a timeout; a timed-out
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

/// Judge a Kani log: it must report a successful verification and at least one
/// cover line, and every cover line must be fully satisfied and non-empty.
fn judge(harness: &str, log: &str) -> Result<(), String> {
    if !log.contains("VERIFICATION:- SUCCESSFUL") {
        return Err(format!("{harness} has no successful verification"));
    }
    let covers: Vec<&str> = log
        .lines()
        .filter(|l| l.contains("** ") && l.contains(" cover properties satisfied"))
        .collect();
    if covers.is_empty() {
        return Err(format!("{harness} has no cover results (vacuous)"));
    }
    for cover in covers {
        let words: Vec<&str> = cover.split_whitespace().collect();
        // `** N of M cover properties satisfied`
        let ok = matches!(words.as_slice(), ["**", n, "of", m, ..] if n == m && *m != "0");
        if !ok {
            return Err(format!("{harness} has an unsatisfied cover: {cover}"));
        }
    }
    Ok(())
}

fn write_record(
    harness: &str,
    log: &str,
    status: Option<i32>,
    seconds: u64,
    dir: &str,
) -> Result<(), String> {
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
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();
    let bytes = serde_json::to_vec_pretty(&doc).map_err(|e| e.to_string())?;
    fs::write(format!("{dir}/{file}.json"), bytes).map_err(|e| e.to_string())
}

/// What one `cargo kani` run produced.
struct Ran {
    code: Option<i32>,
    log: String,
    seconds: u64,
}

fn run_kani(harness: &str) -> Result<Ran, String> {
    let started = Instant::now();
    let output = Command::new("cargo")
        .args([
            "kani",
            "--lib",
            "-Z",
            "stubbing",
            "--output-format",
            "terse",
        ])
        .args(["--harness", harness, "--exact"])
        .output()
        .map_err(|e| format!("cargo kani: {e}"))?;
    let mut log = String::from_utf8_lossy(&output.stdout).into_owned();
    log.push_str(&String::from_utf8_lossy(&output.stderr));
    Ok(Ran {
        code: output.status.code(),
        log,
        seconds: started.elapsed().as_secs(),
    })
}

/// Judge one finished run, writing its record when `evidence` is set.
fn evaluate(harness: &str, ran: &Ran, evidence: Option<&str>) -> Result<(), String> {
    if let Some(dir) = evidence {
        if let Err(e) = write_record(harness, &ran.log, ran.code, ran.seconds, dir) {
            eprintln!("kani_gate: record not written: {e}");
        }
    }
    print!("{}", ran.log);
    if ran.code != Some(0) {
        return Err(format!("{harness} did not verify"));
    }
    judge(harness, &ran.log)
}

/// Run every harness and return every failure; one failing harness does not
/// stop the others.
fn run_all<F>(harnesses: &[String], evidence: Option<&str>, run: F) -> Vec<String>
where
    F: Fn(&str) -> Result<Ran, String>,
{
    harnesses
        .iter()
        .filter_map(|h| run(h).and_then(|ran| evaluate(h, &ran, evidence)).err())
        .collect()
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
    let evidence = env::var("TL_MLTL_EVIDENCE_DIR")
        .ok()
        .filter(|d| !d.is_empty());
    let failures = run_all(&harnesses, evidence.as_deref(), run_kani);
    for failure in &failures {
        eprintln!("kani_gate: {failure}");
    }
    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK: &str = "VERIFICATION:- SUCCESSFUL\n ** 2 of 2 cover properties satisfied\n";

    fn ran(code: i32, log: &str) -> Ran {
        Ran {
            code: Some(code),
            log: log.to_owned(),
            seconds: 1,
        }
    }

    fn scratch(name: &str) -> String {
        let dir = std::env::temp_dir().join(format!("kani-gate-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir.to_string_lossy().into_owned()
    }

    #[test]
    fn a_fully_covered_success_passes() {
        assert!(judge("h", OK).is_ok());
    }

    #[test]
    fn a_vacuous_or_partly_covered_proof_fails() {
        assert!(judge("h", "VERIFICATION:- SUCCESSFUL\n").is_err());
        assert!(judge(
            "h",
            "VERIFICATION:- SUCCESSFUL\n ** 1 of 2 cover properties satisfied\n"
        )
        .is_err());
        assert!(judge(
            "h",
            "VERIFICATION:- SUCCESSFUL\n ** 0 of 0 cover properties satisfied\n"
        )
        .is_err());
        assert!(judge(
            "h",
            "VERIFICATION:- FAILED\n ** 2 of 2 cover properties satisfied\n"
        )
        .is_err());
    }

    #[test]
    fn every_cover_line_is_judged_not_just_the_first() {
        let log = "VERIFICATION:- SUCCESSFUL\n ** 2 of 2 cover properties satisfied\n ** 1 of 3 cover properties satisfied\n";
        assert!(judge("h", log).unwrap_err().contains("1 of 3"));
    }

    #[test]
    fn harnesses_map_to_their_source_files() {
        assert_eq!(source_file("wire::x"), "src/wire/kani_wire.rs");
        assert_eq!(source_file("future::horizon::x"), "src/future/horizon.rs");
        assert_eq!(
            source_file("future::evaluate::x"),
            "src/future/kani_evaluate.rs"
        );
    }

    #[test]
    fn a_nonzero_kani_exit_fails_even_when_the_log_says_successful() {
        assert!(evaluate("h", &ran(1, OK), None)
            .unwrap_err()
            .contains("did not verify"));
        assert!(evaluate("h", &ran(0, OK), None).is_ok());
    }

    #[test]
    fn every_harness_runs_and_every_failure_is_reported() {
        let list: Vec<String> = ["a", "b", "c"].map(str::to_owned).to_vec();
        let seen = std::cell::RefCell::new(Vec::new());
        let failures = run_all(&list, None, |h| {
            seen.borrow_mut().push(h.to_owned());
            match h {
                "b" => Ok(ran(1, "")),
                "c" => Err("cargo kani: not found".to_owned()),
                _ => Ok(ran(0, OK)),
            }
        });
        assert_eq!(*seen.borrow(), ["a", "b", "c"]);
        assert_eq!(failures.len(), 2, "{failures:?}");
        assert!(failures[0].contains("b did not verify"));
        assert!(failures[1].contains("not found"));
    }

    #[test]
    fn a_record_is_written_for_each_harness_when_evidence_is_requested() {
        let dir = scratch("record");
        assert!(evaluate("future::horizon::h", &ran(0, OK), Some(&dir)).is_ok());
        let text = fs::read_to_string(format!("{dir}/future--horizon--h.json")).unwrap();
        assert!(text.contains(KANI_RECORD_SCHEMA), "{text}");
        // A failing proof is recorded too, and is still a failure.
        assert!(evaluate("w", &ran(1, "VERIFICATION:- FAILED\n"), Some(&dir)).is_err());
        assert!(fs::metadata(format!("{dir}/w.json")).is_ok());
    }

    #[test]
    fn a_record_that_cannot_be_written_is_reported_and_does_not_change_the_verdict() {
        let missing = format!("{}/does-not-exist", scratch("missing"));
        assert!(write_record("h", OK, Some(0), 1, &missing).is_err());
        assert!(evaluate("h", &ran(0, OK), Some(&missing)).is_ok());
        assert!(evaluate("h", &ran(1, OK), Some(&missing)).is_err());
    }
}
