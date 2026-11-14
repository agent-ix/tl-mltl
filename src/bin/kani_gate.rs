//! Runs Kani harnesses by exact qualified name and fails unless each one
//! verified with every cover property satisfied: `kani_gate --list FILE` runs
//! every name in FILE and reports every failure, `kani_gate HARNESS` runs one.
//!
//! Kani itself exits 0 when a cover is unsatisfied (a vacuous proof), so the
//! gate reads the result lines. When `TL_MLTL_EVIDENCE_DIR` is set (a campaign
//! run), a per-proof record with its own outcome state is written there for the
//! campaign report; a record that cannot be written is reported but does not
//! change the verdict. Each harness runs under a wall-clock budget (`TL_MLTL_KANI_BUDGET_SECS`, default 1800); a run still going at the budget is killed, recorded as a timeout and fails the gate under its harness name. A timed-out
//! run is never recorded as verified.

use std::{
    env, fs,
    io::Read,
    os::unix::process::CommandExt,
    process::{Command, ExitCode, Stdio},
    thread,
    time::{Duration, Instant},
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
    timed_out: bool,
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
        timed_out,
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
    /// The run was killed at its wall-clock budget; it is never a verified proof.
    timed_out: bool,
}

/// Wall-clock budget of one harness when `TL_MLTL_KANI_BUDGET_SECS` is not set. The proofs are
/// bounded and finish in minutes; half an hour separates a slow host from a diverging proof.
const DEFAULT_BUDGET_SECS: u64 = 1800;

fn budget() -> Duration {
    let secs = env::var("TL_MLTL_KANI_BUDGET_SECS")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .filter(|s| *s > 0)
        .unwrap_or(DEFAULT_BUDGET_SECS);
    Duration::from_secs(secs)
}

/// The command that runs exactly the harness with this qualified name.
fn kani_command(harness: &str) -> Command {
    let mut command = Command::new("cargo");
    command
        .args([
            "kani",
            "--lib",
            "-Z",
            "stubbing",
            "--output-format",
            "terse",
        ])
        .args(["--harness", harness, "--exact"]);
    command
}

fn drain<R: Read + Send + 'static>(mut stream: R) -> thread::JoinHandle<String> {
    thread::spawn(move || {
        let mut bytes = Vec::new();
        let _ = stream.read_to_end(&mut bytes);
        String::from_utf8_lossy(&bytes).into_owned()
    })
}

/// Run `command` in its own process group and kill the whole group when `budget` expires, so
/// a verifier child (CBMC) does not outlive the gate.
fn run_with_budget(mut command: Command, budget: Duration) -> Result<Ran, String> {
    let started = Instant::now();
    let mut child = command
        .process_group(0)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {e}", command.get_program().to_string_lossy()))?;
    let out = drain(child.stdout.take().unwrap());
    let err = drain(child.stderr.take().unwrap());
    let mut timed_out = false;
    let status = loop {
        match child.try_wait().map_err(|e| e.to_string())? {
            Some(status) => break status,
            None if started.elapsed() >= budget => {
                timed_out = true;
                let _ = Command::new("kill")
                    .args(["-KILL", "--", &format!("-{}", child.id())])
                    .status();
                let _ = child.kill();
                break child.wait().map_err(|e| e.to_string())?;
            }
            None => thread::sleep(Duration::from_millis(20)),
        }
    };
    let mut log = out.join().unwrap_or_default();
    log.push_str(&err.join().unwrap_or_default());
    Ok(Ran {
        code: status.code(),
        log,
        seconds: started.elapsed().as_secs(),
        timed_out,
    })
}

fn run_kani(harness: &str) -> Result<Ran, String> {
    run_with_budget(kani_command(harness), budget())
}
/// Judge one finished run, writing its record when `evidence` is set.
fn evaluate(harness: &str, ran: &Ran, evidence: Option<&str>) -> Result<(), String> {
    if let Some(dir) = evidence {
        if let Err(e) = write_record(harness, &ran.log, ran.code, ran.seconds, ran.timed_out, dir) {
            eprintln!("kani_gate: record not written: {e}");
        }
    }
    print!("{}", ran.log);
    if ran.timed_out {
        return Err(format!("{harness} TIMED OUT"));
    }
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
            timed_out: false,
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
        assert!(write_record("h", OK, Some(0), 1, false, &missing).is_err());
        assert!(evaluate("h", &ran(0, OK), Some(&missing)).is_ok());
        assert!(evaluate("h", &ran(1, OK), Some(&missing)).is_err());
    }

    fn sh(script: &str) -> Command {
        let mut command = Command::new("sh");
        command.args(["-c", script]);
        command
    }

    #[test]
    fn the_kani_command_names_the_harness_exactly() {
        let command = kani_command("wire::some_harness");
        let args: Vec<_> = command
            .get_args()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        let at = args.iter().position(|a| a == "--harness").unwrap();
        assert_eq!(args[at + 1], "wire::some_harness");
        assert_eq!(args[at + 2], "--exact");
        assert_eq!(args[..2], ["kani", "--lib"]);
    }

    #[test]
    fn a_child_that_finishes_inside_its_budget_is_judged_by_its_log() {
        let ran = run_with_budget(
            sh("printf 'VERIFICATION:- SUCCESSFUL\\n ** 2 of 2 cover properties satisfied\\n'"),
            Duration::from_secs(30),
        )
        .unwrap();
        assert!(!ran.timed_out);
        assert_eq!(ran.code, Some(0));
        assert!(evaluate("h", &ran, None).is_ok());
    }

    #[test]
    fn a_child_past_its_budget_is_killed_and_fails_under_the_harness_name() {
        let started = Instant::now();
        // A slow child that also holds a grandchild, as cargo kani holds CBMC.
        let ran = run_with_budget(
            sh("echo started; sleep 60 & sleep 60"),
            Duration::from_millis(300),
        )
        .unwrap();
        assert!(
            started.elapsed() < Duration::from_secs(30),
            "the child was not killed"
        );
        assert!(ran.timed_out);
        assert!(ran.log.contains("started"));
        // Even a log that looks successful cannot pass a timed-out run.
        let looks_fine = Ran {
            log: OK.to_owned(),
            code: Some(0),
            ..ran
        };
        let err = evaluate("future::horizon::slow", &looks_fine, None).unwrap_err();
        assert!(
            err.contains("future::horizon::slow") && err.contains("TIMED OUT"),
            "{err}"
        );
    }

    #[test]
    fn a_timeout_is_recorded_as_a_timeout_and_the_budget_has_a_default() {
        let dir = scratch("timeout");
        let ran = Ran {
            timed_out: true,
            ..ran(0, OK)
        };
        assert!(evaluate("w", &ran, Some(&dir)).is_err());
        let text = fs::read_to_string(format!("{dir}/w.json")).unwrap();
        assert!(text.contains("timeout"), "{text}");
        assert_eq!(DEFAULT_BUDGET_SECS, 1800);
    }

    #[test]
    fn an_unstartable_command_is_an_error_naming_the_program() {
        let err = run_with_budget(
            Command::new("no-such-program-tl-mltl"),
            Duration::from_secs(1),
        )
        .err()
        .unwrap();
        assert!(err.contains("no-such-program-tl-mltl"), "{err}");
    }
}
