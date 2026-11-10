//! Fuzz campaign driver (FR-021): every target under several seeds with a fixed
//! budget and a trailing-window plateau stop, writing a [`FuzzRecord`].
//!
//! `fuzz_campaign OUT_DIR [--budget 900] [--window 300] [--seeds 101,102,103]
//! [--triple T] [--targets a,b] [--fuzz-dir fuzz]`
//!
//! Each run starts from the checked-in seed corpus (read-only) into its own
//! scratch corpus, so seeded runs stay independent. Runs of different targets
//! execute in parallel; runs of one target execute in sequence. Build time is
//! excluded from the observed duration.
//!
//! The plateau rule is `fuzz_record::decide_stop`. Progress is sampled on a
//! one-second clock as well as on every libFuzzer progress line, so a target
//! that prints rarely (a low execution rate) still reaches the plateau
//! decision instead of waiting for a line to arrive. A no-crash record is a
//! bounded observation, not a correctness claim. A run that printed no progress
//! line at all is recorded as a tool error, never as a plateau or growth; and
//! when a target's stderr is block-buffered the lines, and so the observed time
//! of growth, are those of the flushes.

use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Write},
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    sync::mpsc::{self, RecvTimeoutError},
    thread,
    time::{Duration, Instant},
};

use tl_mltl::campaign::sha256_hex;
use tl_mltl::fuzz_record::{
    classify_run, decide_stop, parse_record, CrashRecord, FuzzRecord, ProgressSample, RunRecord,
    RunState, StopDecision, StopReason, FUZZ_RECORD_SCHEMA,
};

struct Args {
    out: PathBuf,
    budget: u64,
    window: u64,
    seeds: Vec<u64>,
    triple: String,
    targets: Vec<String>,
    fuzz_dir: PathBuf,
}

fn parse_args(raw: &[String]) -> Result<Args, String> {
    let mut args = Args {
        out: PathBuf::new(),
        budget: 900,
        window: 300,
        seeds: vec![101, 102, 103],
        triple: "aarch64-apple-darwin".to_owned(),
        targets: Vec::new(),
        fuzz_dir: PathBuf::from("fuzz"),
    };
    let mut it = raw.iter();
    let mut out = None;
    while let Some(a) = it.next() {
        let mut value = |name: &str| it.next().cloned().ok_or(format!("{name} needs a value"));
        let number =
            |name: &str, v: String| v.parse::<u64>().map_err(|e| format!("{name}: {v:?}: {e}"));
        match a.as_str() {
            "--budget" => args.budget = number("--budget", value("--budget")?)?,
            "--window" => args.window = number("--window", value("--window")?)?,
            "--seeds" => {
                args.seeds = value("--seeds")?
                    .split(',')
                    .map(|s| number("--seeds", s.to_owned()))
                    .collect::<Result<_, _>>()?;
            }
            "--triple" => args.triple = value("--triple")?,
            "--targets" => {
                args.targets = value("--targets")?.split(',').map(str::to_owned).collect();
            }
            "--fuzz-dir" => args.fuzz_dir = PathBuf::from(value("--fuzz-dir")?),
            flag if flag.starts_with("--") => return Err(format!("unknown option {flag}")),
            _ if out.is_none() => out = Some(PathBuf::from(a)),
            _ => return Err("more than one OUT_DIR".to_owned()),
        }
    }
    args.out = out.ok_or("missing OUT_DIR")?;
    if args.budget == 0 || args.window == 0 || args.seeds.is_empty() {
        return Err("budget, window and seeds must be non-empty and non-zero".to_owned());
    }
    Ok(args)
}

/// Fields of a libFuzzer progress line `#N<TAB>EVENT cov: C ft: F corp: K/...`.
fn progress(line: &str) -> Option<(u64, u64, u64)> {
    if !line.starts_with('#') {
        return None;
    }
    let mut words = line.split_whitespace();
    let mut take = |key: &str| {
        words
            .by_ref()
            .skip_while(|w| *w != key)
            .nth(1)
            .and_then(|v| v.split('/').next()?.parse::<u64>().ok())
    };
    Some((take("cov:")?, take("ft:")?, take("corp:")?))
}

fn executed_units(line: &str) -> Option<u64> {
    line.strip_prefix("stat::number_of_executed_units:")?
        .trim()
        .parse()
        .ok()
}

fn interrupt(pid: u32) {
    let _ = Command::new("kill")
        .args(["-INT", &pid.to_string()])
        .status();
}

fn run_one(target: &str, seed: u64, args: &Args) -> Result<RunRecord, String> {
    let binary = args
        .fuzz_dir
        .join("target")
        .join(&args.triple)
        .join("release")
        .join(target);
    let scratch = args.out.join("scratch").join(format!("{target}-{seed}"));
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(&scratch).map_err(|e| format!("{}: {e}", scratch.display()))?;
    let crashes_dir = args.out.join("crashes");
    let logs_dir = args.out.join("logs");
    fs::create_dir_all(&crashes_dir).map_err(|e| e.to_string())?;
    fs::create_dir_all(&logs_dir).map_err(|e| e.to_string())?;
    let prefix = format!("{target}-{seed}-");
    let corpus = args.fuzz_dir.join("corpus").join(target);
    let mut child = Command::new(&binary)
        .arg(&scratch)
        .arg(&corpus)
        .arg(format!("-seed={seed}"))
        .arg(format!("-max_total_time={}", args.budget))
        .arg("-rss_limit_mb=8192")
        .arg(format!(
            "-artifact_prefix={}/{prefix}",
            crashes_dir.display()
        ))
        .arg("-print_final_stats=1")
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("{}: {e}", binary.display()))?;
    let started = Instant::now();
    let stderr = child.stderr.take().ok_or("no stderr pipe")?;
    let log_path = logs_dir.join(format!("{target}-{seed}.log"));
    let mut log = File::create(&log_path).map_err(|e| e.to_string())?;
    let (tx, rx) = mpsc::channel::<String>();
    // Lines are read as bytes and decoded lossily: a sanitizer report may hold
    // non-UTF-8 bytes and must not end the reader early.
    let reader = thread::spawn(move || {
        let mut reader = BufReader::new(stderr);
        let mut buf = Vec::new();
        while reader.read_until(b'\n', &mut buf).unwrap_or(0) > 0 {
            let _ = tx.send(String::from_utf8_lossy(&buf).into_owned());
            buf.clear();
        }
    });

    let (mut cov, mut features, mut corpus_entries, mut executions) = (0, 0, 0, 0);
    let mut samples: Vec<ProgressSample> = Vec::new();
    let mut stopped_by_plateau = false;
    // Until libFuzzer has printed one progress line the feature count is
    // unknown, not zero, so no plateau may be decided from it.
    let mut seen_progress = false;
    let mut text = String::new();
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(line) => {
                log.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
                text.push_str(&line);
                if let Some((c, f, k)) = progress(&line) {
                    seen_progress = true;
                    (cov, features, corpus_entries) = (c, f.max(features), k);
                }
                if let Some(n) = executed_units(&line) {
                    executions = n;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        let elapsed_s = started.elapsed().as_secs();
        if samples.last().map(|s| s.elapsed_s) != Some(elapsed_s) {
            samples.push(ProgressSample {
                elapsed_s,
                features,
            });
        }
        if seen_progress
            && !stopped_by_plateau
            && matches!(
                decide_stop(&samples, args.budget, args.window),
                Ok(StopDecision::Plateau { .. })
            )
        {
            stopped_by_plateau = true;
            interrupt(child.id());
        }
    }
    let _ = reader.join();
    let status = child.wait().map_err(|e| e.to_string())?;
    let observed_s = started.elapsed().as_secs();
    // The final sample is at least the budget for a run that ended on its own
    // clock, so the plateau rule decides it as it would at any other sample.
    samples.push(ProgressSample {
        elapsed_s: observed_s.max(if stopped_by_plateau { 0 } else { args.budget }),
        features,
    });
    let mut state = classify_run(status.code(), &text, stopped_by_plateau);
    if state == RunState::Completed && !seen_progress {
        // No feature count was ever observed, so neither plateau nor growth can be stated.
        state = RunState::ToolError;
    }
    let mut crashes = Vec::new();
    if state == RunState::Crash {
        crashes = crash_files(&crashes_dir, &prefix)?;
        if crashes.is_empty() {
            // A crash with no reproducer cannot be recorded as a crash.
            state = RunState::ToolError;
        }
    }
    let stop_reason = if state == RunState::Completed {
        match decide_stop(&samples, args.budget, args.window) {
            Ok(StopDecision::Plateau { .. }) => StopReason::Plateau,
            _ => StopReason::GrowthObserved,
        }
    } else {
        StopReason::Abnormal
    };
    Ok(RunRecord {
        target: target.to_owned(),
        seed,
        requested_seconds: args.budget,
        observed_seconds: observed_s,
        plateau_window_seconds: args.window,
        executions,
        coverage_edges: cov,
        coverage_features: features,
        corpus_entries,
        state,
        stop_reason,
        crashes,
    })
}

fn crash_files(dir: &Path, prefix: &str) -> Result<Vec<CrashRecord>, String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok()?.file_name().into_string().ok())
        .filter(|n| n.starts_with(prefix))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|file| {
            let bytes = fs::read(dir.join(&file)).map_err(|e| format!("{file}: {e}"))?;
            Ok(CrashRecord {
                sha256: sha256_hex(&bytes),
                file,
                conclusive: false,
            })
        })
        .collect()
}

fn discover(fuzz_dir: &Path) -> Result<Vec<String>, String> {
    let dir = fuzz_dir.join("fuzz_targets");
    let mut names: Vec<String> = fs::read_dir(&dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| {
            let path = e.ok()?.path();
            (path.extension()? == "rs").then(|| path.file_stem()?.to_str().map(str::to_owned))?
        })
        .collect();
    names.sort();
    Ok(names)
}

fn campaign(args: &Args) -> Result<(), String> {
    fs::create_dir_all(&args.out).map_err(|e| e.to_string())?;
    let targets = if args.targets.is_empty() {
        discover(&args.fuzz_dir)?
    } else {
        args.targets.clone()
    };
    let results: BTreeMap<String, Result<Vec<RunRecord>, String>> = thread::scope(|scope| {
        let handles: Vec<_> = targets
            .iter()
            .map(|t| {
                let handle = scope.spawn(|| {
                    args.seeds
                        .iter()
                        .map(|s| run_one(t, *s, args))
                        .collect::<Result<Vec<_>, _>>()
                });
                (t.clone(), handle)
            })
            .collect();
        handles
            .into_iter()
            .map(|(t, h)| {
                let r = h
                    .join()
                    .unwrap_or_else(|_| Err("run thread panicked".into()));
                (t, r)
            })
            .collect()
    });
    let mut runs = Vec::new();
    for t in &targets {
        runs.extend(results[t].clone()?);
    }
    let record = FuzzRecord {
        schema_version: FUZZ_RECORD_SCHEMA.to_owned(),
        runs,
    };
    let text = serde_json::to_string_pretty(&record).map_err(|e| e.to_string())?;
    // Refuse to write a record the strict reader would refuse.
    parse_record(&text).map_err(|e| e.to_string())?;
    let path = args.out.join("record.json");
    fs::write(&path, format!("{text}\n")).map_err(|e| format!("{}: {e}", path.display()))?;
    println!("{text}");
    Ok(())
}

fn main() -> ExitCode {
    let raw: Vec<String> = env::args().skip(1).collect();
    match parse_args(&raw).and_then(|a| campaign(&a)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fuzz_campaign: {e}");
            ExitCode::from(2)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_line_fields_are_read() {
        let line =
            "#2161\tNEW    cov: 2218 ft: 12424 corp: 1303/29Kb lim: 172 exec/s: 2161 rss: 304Mb";
        assert_eq!(progress(line), Some((2218, 12424, 1303)));
        assert_eq!(progress("INFO: Seed: 1"), None);
        assert_eq!(progress("#5\tpulse"), None);
    }

    #[test]
    fn final_stats_executions_are_read() {
        assert_eq!(
            executed_units("stat::number_of_executed_units: 41361"),
            Some(41361)
        );
        assert_eq!(executed_units("stat::average_exec_per_sec: 3"), None);
    }

    #[test]
    fn arguments_default_and_refuse_bad_input() {
        let a = parse_args(&["out".to_owned()]).unwrap();
        assert_eq!(
            (a.budget, a.window, a.seeds),
            (900, 300, vec![101, 102, 103])
        );
        assert!(parse_args(&[]).is_err());
        assert!(parse_args(&["out".into(), "--budget".into(), "0".into()]).is_err());
        assert!(parse_args(&["out".into(), "--nope".into()]).is_err());
    }
}
