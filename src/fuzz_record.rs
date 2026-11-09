//! Fuzz run records (FR-021): typed run states, the plateau decision, crash
//! conclusiveness and retained-corpus replay.
//!
//! A fuzz driver produces a [`FuzzRecord`]; the campaign runner reads it back
//! strictly and binds each run's seed, duration, executions, coverage and
//! crashes into the campaign report (see [`crate::campaign`]). Nothing here
//! runs a fuzzer. A record states what one bounded run observed; it is not a
//! correctness, release, qualification or certification verdict.

use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::campaign::sha256_hex;

/// Schema of a fuzz run record file.
pub const FUZZ_RECORD_SCHEMA: &str = "tl-mltl.fuzz-run-record/v1";

/// How one fuzz run ended. Every state is distinct; none reads as a pass
/// except [`RunState::Completed`], and even that is a bounded observation.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    /// Ran to its budget or a plateau stop with no finding.
    Completed,
    /// The target panicked or a sanitizer reported a defect.
    Crash,
    /// A single input exceeded the per-input timeout.
    Timeout,
    /// The target exceeded the memory limit.
    OutOfMemory,
    /// The fuzzing tool failed without a finding in the target.
    ToolError,
    /// The tool refused to start for lack of a resource.
    ResourceRefusal,
    /// Interrupted by the operator, not by the plateau rule.
    Cancelled,
    /// Never started.
    NotRun,
}

/// Why a run stopped.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StopReason {
    /// No new coverage feature over the trailing window.
    Plateau,
    /// The budget ended while coverage was still growing.
    GrowthObserved,
    /// Stopped by the state named in [`RunRecord::state`].
    Abnormal,
}

/// A crash input kept by a run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CrashRecord {
    /// File name of the reproducer under the run's crash directory.
    pub file: String,
    /// SHA-256 of the reproducer's exact bytes.
    pub sha256: String,
    /// True only after [`confirm_crash`] reproduced these bytes independently.
    pub conclusive: bool,
}

/// One run of one target under one seed.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunRecord {
    /// Fuzz target name.
    pub target: String,
    /// Seed given to the fuzzer.
    pub seed: u64,
    /// Budget requested, in seconds.
    pub requested_seconds: u64,
    /// Time observed, in seconds, excluding build time.
    pub observed_seconds: u64,
    /// Trailing window of the plateau rule, in seconds.
    pub plateau_window_seconds: u64,
    /// Executions performed.
    pub executions: u64,
    /// Coverage edges reached.
    pub coverage_edges: u64,
    /// Coverage features reached.
    pub coverage_features: u64,
    /// Corpus entries at the end.
    pub corpus_entries: u64,
    /// End state.
    pub state: RunState,
    /// Why it stopped.
    pub stop_reason: StopReason,
    /// Crash inputs kept.
    pub crashes: Vec<CrashRecord>,
}

/// The record file a driver writes for one campaign.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct FuzzRecord {
    /// Must equal [`FUZZ_RECORD_SCHEMA`].
    pub schema_version: String,
    /// Runs in execution order.
    pub runs: Vec<RunRecord>,
}

/// Why a record was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecordErrorCode {
    /// Not valid JSON for the record shape, or an unknown field.
    Malformed,
    /// Wrong `schemaVersion`.
    Schema,
    /// State, stop reason and crash list disagree.
    Inconsistent,
}

/// A refused record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordError {
    /// Stable code.
    pub code: RecordErrorCode,
    /// Human-readable detail; never branch on it.
    pub detail: String,
}

impl fmt::Display for RecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "fuzz record {:?}: {}", self.code, self.detail)
    }
}

impl std::error::Error for RecordError {}

fn refuse(code: RecordErrorCode, detail: String) -> RecordError {
    RecordError { code, detail }
}

fn check_run(run: &RunRecord) -> Result<(), RecordError> {
    let id = format!("{} seed {}", run.target, run.seed);
    let completed = run.state == RunState::Completed;
    if completed == (run.stop_reason == StopReason::Abnormal) {
        return Err(refuse(
            RecordErrorCode::Inconsistent,
            format!(
                "{id}: state {:?} with stop {:?}",
                run.state, run.stop_reason
            ),
        ));
    }
    if (run.state == RunState::Crash) == run.crashes.is_empty() {
        return Err(refuse(
            RecordErrorCode::Inconsistent,
            format!(
                "{id}: state {:?} with {} crash inputs",
                run.state,
                run.crashes.len()
            ),
        ));
    }
    if run
        .crashes
        .iter()
        .any(|c| c.conclusive && c.sha256.len() != 64)
    {
        return Err(refuse(
            RecordErrorCode::Inconsistent,
            format!("{id}: conclusive crash without a reproducer digest"),
        ));
    }
    Ok(())
}

/// Read a record strictly: unknown fields, a wrong schema and inconsistent
/// runs are refused, so a read followed by a write is the identity.
pub fn parse_record(text: &str) -> Result<FuzzRecord, RecordError> {
    let record: FuzzRecord = serde_json::from_str(text)
        .map_err(|e| refuse(RecordErrorCode::Malformed, e.to_string()))?;
    if record.schema_version != FUZZ_RECORD_SCHEMA {
        return Err(refuse(
            RecordErrorCode::Schema,
            format!("schema {:?}", record.schema_version),
        ));
    }
    record.runs.iter().try_for_each(check_run)?;
    Ok(record)
}

/// True when the text carries this record's schema marker, so a reader knows
/// to apply [`parse_record`] rather than treat the JSON as other evidence.
pub fn is_record(text: &[u8]) -> bool {
    serde_json::from_slice::<serde_json::Value>(text)
        .ok()
        .and_then(|v| v.get("schemaVersion")?.as_str().map(str::to_owned))
        .is_some_and(|s| s == FUZZ_RECORD_SCHEMA)
}

/// Classify a finished libFuzzer process. `stopped_by_plateau` is true when the
/// driver itself sent the interrupt.
pub fn classify_run(exit_code: Option<i32>, log: &str, stopped_by_plateau: bool) -> RunState {
    if log.contains("SUMMARY: libFuzzer: timeout") {
        RunState::Timeout
    } else if log.contains("out-of-memory") {
        RunState::OutOfMemory
    } else if log.contains("SUMMARY:") {
        RunState::Crash
    } else {
        match exit_code {
            Some(0) => RunState::Completed,
            Some(130) | None if stopped_by_plateau => RunState::Completed,
            Some(130) => RunState::Cancelled,
            _ => RunState::ToolError,
        }
    }
}

/// One progress observation: seconds since the target started and the number
/// of coverage features then reached.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProgressSample {
    /// Seconds since the run started, excluding build time.
    pub elapsed_s: u64,
    /// Coverage features reached.
    pub features: u64,
}

/// Outcome of the plateau rule over a sample history.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StopDecision {
    /// Neither the budget nor a plateau has been reached.
    Continue,
    /// No new feature over the trailing window; stop.
    Plateau {
        /// Second at which the last new feature appeared.
        last_growth_s: u64,
    },
    /// The budget ended while coverage still grew.
    GrowthObserved {
        /// Second at which the last new feature appeared.
        last_growth_s: u64,
    },
}

/// Why a sample history could not be judged.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DecisionError {
    /// The window or the budget is zero.
    EmptyWindow,
    /// Time ran backwards or the feature count fell.
    NotMonotone,
}

/// Pure plateau rule. Scan `samples` in order; the first sample at which no
/// new feature has appeared for `window_s` (and at least `window_s` have
/// elapsed) is a plateau; otherwise the first sample at or past `budget_s`
/// ends the run with growth observed. The plateau test comes first, so a run
/// that has stalled when the budget ends is a plateau, never growth.
pub fn decide_stop(
    samples: &[ProgressSample],
    budget_s: u64,
    window_s: u64,
) -> Result<StopDecision, DecisionError> {
    if window_s == 0 || budget_s == 0 {
        return Err(DecisionError::EmptyWindow);
    }
    let mut best = 0;
    let mut last_growth_s = 0;
    let mut prev: Option<ProgressSample> = None;
    for s in samples {
        if prev.is_some_and(|p| s.elapsed_s < p.elapsed_s || s.features < p.features) {
            return Err(DecisionError::NotMonotone);
        }
        prev = Some(*s);
        if s.features > best {
            best = s.features;
            last_growth_s = s.elapsed_s;
        }
        if s.elapsed_s >= window_s && s.elapsed_s - last_growth_s >= window_s {
            return Ok(StopDecision::Plateau { last_growth_s });
        }
        if s.elapsed_s >= budget_s {
            return Ok(StopDecision::GrowthObserved { last_growth_s });
        }
    }
    Ok(StopDecision::Continue)
}

/// Result of running a target on exact bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Reproduction {
    /// The target panicked.
    Reproduced {
        /// Panic message when it was a string.
        message: String,
    },
    /// The target returned normally.
    DidNotReproduce,
}

/// Run `target` once on `bytes`, catching a panic. This is the same
/// decode-style call the seed-replay tests make on checked-in seeds.
pub fn replay_input<F: Fn(&[u8])>(target: F, bytes: &[u8]) -> Reproduction {
    match catch_unwind(AssertUnwindSafe(|| target(bytes))) {
        Ok(()) => Reproduction::DidNotReproduce,
        Err(p) => Reproduction::Reproduced {
            message: p
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| p.downcast_ref::<&str>().map(|s| (*s).to_owned()))
                .unwrap_or_default(),
        },
    }
}

/// Verdict on a recorded crash after independent replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CrashStatus {
    /// The bytes handed in are not the bytes the run recorded.
    ByteMismatch,
    /// The exact recorded bytes did not make the target panic.
    NotReproduced,
    /// The exact recorded bytes reproduce the panic.
    Conclusive {
        /// Panic message.
        message: String,
    },
}

/// A crash is conclusive only when the bytes match the recorded digest and an
/// independent run of `target` on them panics.
pub fn confirm_crash<F: Fn(&[u8])>(crash: &CrashRecord, bytes: &[u8], target: F) -> CrashStatus {
    if sha256_hex(bytes) != crash.sha256 {
        return CrashStatus::ByteMismatch;
    }
    match replay_input(target, bytes) {
        Reproduction::Reproduced { message } => CrashStatus::Conclusive { message },
        Reproduction::DidNotReproduce => CrashStatus::NotReproduced,
    }
}

/// Outcome of replaying a retained corpus directory.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CorpusReplay {
    /// Files replayed.
    pub files: usize,
    /// Names of files on which the target panicked, sorted.
    pub panicked: Vec<String>,
}

/// Replay every file in `dir` through `target`. The corpus is generated
/// evidence kept outside git; a directory that cannot be read is an error, not
/// an empty pass.
pub fn replay_corpus<F: Fn(&[u8])>(dir: &Path, target: F) -> Result<CorpusReplay, String> {
    let mut names = BTreeSet::new();
    for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
        let entry = entry.map_err(|e| e.to_string())?;
        if entry.file_type().map_err(|e| e.to_string())?.is_file() {
            names.insert(entry.file_name().to_string_lossy().into_owned());
        }
    }
    let mut panicked = Vec::new();
    for name in &names {
        let bytes = fs::read(dir.join(name)).map_err(|e| format!("{name}: {e}"))?;
        if matches!(
            replay_input(&target, &bytes),
            Reproduction::Reproduced { .. }
        ) {
            panicked.push(name.clone());
        }
    }
    Ok(CorpusReplay {
        files: names.len(),
        panicked,
    })
}
