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

    let mut observer = Observer::default();
    let mut text = String::new();
    loop {
        match rx.recv_timeout(Duration::from_secs(1)) {
            Ok(line) => {
                log.write_all(line.as_bytes()).map_err(|e| e.to_string())?;
                text.push_str(&line);
                observer.line(&line);
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
        if observer.tick(started.elapsed().as_secs(), args.budget, args.window) {
            interrupt(child.id());
        }
    }
    let _ = reader.join();
    let status = child.wait().map_err(|e| e.to_string())?;
    let observed_s = started.elapsed().as_secs();
    let (mut state, stop_reason) =
        observer.conclude(status.code(), &text, observed_s, args.budget, args.window);
    let mut crashes = Vec::new();
    if state == RunState::Crash {
        crashes = crash_files(&crashes_dir, &prefix)?;
        if crashes.is_empty() {
            // A crash with no reproducer cannot be recorded as a crash.
            state = RunState::ToolError;
        }
    }
    Ok(RunRecord {
        target: target.to_owned(),
        seed,
        requested_seconds: args.budget,
        observed_seconds: observed_s,
        plateau_window_seconds: args.window,
        executions: observer.executions,
        coverage_edges: observer.cov,
        coverage_features: observer.features,
        corpus_entries: observer.corpus_entries,
        state,
        stop_reason,
        crashes,
    })
}

/// What the driver has seen of one run so far.
#[derive(Default)]
struct Observer {
    cov: u64,
    features: u64,
    corpus_entries: u64,
    executions: u64,
    samples: Vec<ProgressSample>,
    stopped_by_plateau: bool,
    /// Until libFuzzer has printed one progress line the feature count is
    /// unknown, not zero, so no plateau may be decided from it.
    seen_progress: bool,
}

impl Observer {
    fn line(&mut self, line: &str) {
        if let Some((c, f, k)) = progress(line) {
            self.seen_progress = true;
            (self.cov, self.features, self.corpus_entries) = (c, f.max(self.features), k);
        }
        if let Some(n) = executed_units(line) {
            self.executions = n;
        }
    }

    /// Record the clock sample for `elapsed_s` and return true exactly once,
    /// when the run should be interrupted because it reached a plateau.
    fn tick(&mut self, elapsed_s: u64, budget: u64, window: u64) -> bool {
        if self.samples.last().map(|s| s.elapsed_s) != Some(elapsed_s) {
            self.samples.push(ProgressSample {
                elapsed_s,
                features: self.features,
            });
        }
        if self.seen_progress
            && !self.stopped_by_plateau
            && matches!(
                decide_stop(&self.samples, budget, window),
                Ok(StopDecision::Plateau { .. })
            )
        {
            self.stopped_by_plateau = true;
            return true;
        }
        false
    }

    /// End state and stop reason of a finished process. The final sample is at
    /// least the budget for a run that ended on its own clock, so the plateau
    /// rule decides it as it would at any other sample. A run that printed no
    /// progress line is a tool error: neither plateau nor growth can be stated.
    fn conclude(
        &mut self,
        code: Option<i32>,
        log: &str,
        observed_s: u64,
        budget: u64,
        window: u64,
    ) -> (RunState, StopReason) {
        self.samples.push(ProgressSample {
            elapsed_s: observed_s.max(if self.stopped_by_plateau { 0 } else { budget }),
            features: self.features,
        });
        let mut state = classify_run(code, log, self.stopped_by_plateau);
        if state == RunState::Completed && !self.seen_progress {
            state = RunState::ToolError;
        }
        let stop_reason = if state == RunState::Completed {
            match decide_stop(&self.samples, budget, window) {
                Ok(StopDecision::Plateau { .. }) => StopReason::Plateau,
                _ => StopReason::GrowthObserved,
            }
        } else {
            StopReason::Abnormal
        };
        (state, stop_reason)
    }
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

    fn feed(o: &mut Observer, secs: &[(u64, u64)], budget: u64, window: u64) -> Vec<bool> {
        secs.iter()
            .map(|(t, ft)| {
                o.line(&format!("#1\tNEW    cov: 5 ft: {ft} corp: 2/9b lim: 4"));
                o.tick(*t, budget, window)
            })
            .collect()
    }

    #[test]
    fn no_plateau_is_decided_before_any_progress_line() {
        let mut o = Observer::default();
        for t in 0..50 {
            assert!(!o.tick(t, 100, 10), "second {t}");
        }
        let (state, stop) = o.conclude(Some(0), "", 60, 100, 10);
        assert_eq!((state, stop), (RunState::ToolError, StopReason::Abnormal));
    }

    #[test]
    fn a_stalled_run_is_interrupted_once_and_concluded_as_a_plateau() {
        let mut o = Observer::default();
        let fired = feed(
            &mut o,
            &[(0, 5), (4, 6), (9, 6), (14, 6), (15, 6), (16, 6)],
            100,
            10,
        );
        assert_eq!(fired, [false, false, false, true, false, false]);
        let (state, stop) = o.conclude(Some(72), "", 17, 100, 10);
        assert_eq!((state, stop), (RunState::Completed, StopReason::Plateau));
    }

    #[test]
    fn a_run_still_growing_at_the_budget_is_growth_observed() {
        let mut o = Observer::default();
        let fired = feed(&mut o, &[(0, 1), (30, 2), (60, 3), (95, 4)], 100, 40);
        assert!(fired.iter().all(|f| !f));
        let (state, stop) = o.conclude(Some(0), "", 100, 100, 40);
        assert_eq!(
            (state, stop),
            (RunState::Completed, StopReason::GrowthObserved)
        );
    }

    #[test]
    fn a_crash_summary_is_abnormal_and_feature_counts_never_fall() {
        let mut o = Observer::default();
        feed(&mut o, &[(0, 9)], 100, 10);
        o.line("#2\tNEW    cov: 5 ft: 3 corp: 2/9b");
        assert_eq!(o.features, 9);
        let (state, stop) = o.conclude(Some(1), "SUMMARY: AddressSanitizer: x", 5, 100, 10);
        assert_eq!((state, stop), (RunState::Crash, StopReason::Abnormal));
    }

    fn fuzz_dir(name: &str, binary: &str) -> Args {
        let dir = std::env::temp_dir().join(format!("fuzz-campaign-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let release = dir.join("fuzz/target/t/release");
        fs::create_dir_all(&release).unwrap();
        fs::create_dir_all(dir.join("fuzz/fuzz_targets")).unwrap();
        fs::create_dir_all(dir.join("fuzz/corpus/a")).unwrap();
        fs::write(dir.join("fuzz/fuzz_targets/a.rs"), "").unwrap();
        std::os::unix::fs::symlink(binary, release.join("a")).unwrap();
        Args {
            out: dir.join("out"),
            budget: 1,
            window: 1,
            seeds: vec![7],
            triple: "t".to_owned(),
            targets: Vec::new(),
            fuzz_dir: dir.join("fuzz"),
        }
    }

    // Trace: TC-109, FR-021-AC-1
    #[test]
    fn the_default_campaign_covers_every_target_under_three_distinct_seeds_with_full_records() {
        let args = parse_args(&["out".to_owned()]).unwrap();
        let seeds: std::collections::BTreeSet<u64> = args.seeds.iter().copied().collect();
        assert!(
            seeds.len() >= 3 && seeds.len() == args.seeds.len(),
            "{:?}",
            args.seeds
        );
        assert!(args.budget > 0 && args.window > 0);
        let fuzz = Path::new(env!("CARGO_MANIFEST_DIR")).join("fuzz");
        let targets = discover(&fuzz).unwrap();
        let manifest = fs::read_to_string(fuzz.join("Cargo.toml")).unwrap();
        assert!(!targets.is_empty());
        for target in &targets {
            assert!(
                manifest.contains(&format!("name = \"{target}\"")),
                "{target}"
            );
        }
        let run = RunRecord {
            target: targets[0].clone(),
            seed: args.seeds[0],
            requested_seconds: 1,
            observed_seconds: 1,
            plateau_window_seconds: 1,
            executions: 1,
            coverage_edges: 1,
            coverage_features: 1,
            corpus_entries: 1,
            state: RunState::Completed,
            stop_reason: StopReason::GrowthObserved,
            crashes: Vec::new(),
        };
        let json = serde_json::to_value(&run).unwrap();
        for field in [
            "target",
            "seed",
            "requestedSeconds",
            "observedSeconds",
            "executions",
            "coverageEdges",
            "coverageFeatures",
            "corpusEntries",
        ] {
            assert!(!json[field].is_null(), "{field}");
        }
    }

    #[test]
    fn targets_are_discovered_from_the_fuzz_target_sources() {
        let args = fuzz_dir("discover", "/usr/bin/true");
        assert_eq!(discover(&args.fuzz_dir).unwrap(), ["a"]);
        assert!(discover(&args.out).is_err());
    }

    #[test]
    fn a_target_that_prints_no_progress_is_a_tool_error_record() {
        let args = fuzz_dir("silent", "/usr/bin/true");
        campaign(&args).unwrap();
        let record =
            parse_record(&fs::read_to_string(args.out.join("record.json")).unwrap()).unwrap();
        assert_eq!(record.runs.len(), 1);
        assert_eq!(record.runs[0].state, RunState::ToolError);
        assert_eq!(record.runs[0].stop_reason, StopReason::Abnormal);
        assert!(args.out.join("logs/a-7.log").exists());
    }

    #[test]
    fn a_failing_tool_is_a_tool_error_and_a_missing_binary_is_a_driver_error() {
        let args = fuzz_dir("failing", "/usr/bin/false");
        campaign(&args).unwrap();
        let record =
            parse_record(&fs::read_to_string(args.out.join("record.json")).unwrap()).unwrap();
        assert_eq!(record.runs[0].state, RunState::ToolError);
        let missing = fuzz_dir("missing", "/nonexistent/binary");
        assert!(campaign(&missing).is_err());
    }
}
