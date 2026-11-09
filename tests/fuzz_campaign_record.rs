//! The retained 900 s fuzz baseline record names every fuzz target under at
//! least three distinct seeds with the fields FR-021 requires. This checks the
//! record's shape; it does not rerun any fuzzer and proves nothing about the
//! targets.

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::Value;
use tl_mltl::campaign::sha256_hex;
use tl_mltl::fuzz_record::{
    classify_run, confirm_crash, decide_stop, parse_record, replay_corpus, CrashRecord,
    CrashStatus, DecisionError, ProgressSample, RecordErrorCode, RunState, StopDecision,
    FUZZ_RECORD_SCHEMA,
};
use tl_mltl::wire::{OwnerLimits, ValidatedCommand};

// Trace: FR-021-AC-1
#[test]
fn baseline_record_covers_every_target_with_three_distinct_seeds() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let record: Value = serde_json::from_slice(
        &fs::read(root.join("fuzz/evidence/2026-11-08/campaign-900s.json")).unwrap(),
    )
    .unwrap();
    let runs = record["runs"].as_array().unwrap();
    let mut targets: BTreeSet<String> = fs::read_dir(root.join("fuzz/fuzz_targets"))
        .unwrap()
        .map(|entry| {
            let name = entry.unwrap().file_name();
            Path::new(&name)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(targets.len(), 6);
    for target in std::mem::take(&mut targets) {
        let mine: Vec<&Value> = runs.iter().filter(|r| r["target"] == target).collect();
        let seeds: BTreeSet<u64> = mine.iter().map(|r| r["seed"].as_u64().unwrap()).collect();
        assert!(seeds.len() >= 3, "{target}: seeds {seeds:?}");
        for run in mine {
            for field in [
                "requested_seconds",
                "observed_seconds",
                "executions",
                "final_cov_edges",
                "final_features",
                "final_corpus_entries",
                "stop_reason",
            ] {
                assert!(!run[field].is_null(), "{target}: missing {field}");
            }
            assert!(run["executions"].as_u64().unwrap() > 0);
        }
    }
}

fn run_json(extra: &str) -> String {
    format!(
        r#"{{"target":"wire_cli_decode","seed":101,"requestedSeconds":900,
        "observedSeconds":900,"plateauWindowSeconds":300,"executions":10,
        "coverageEdges":5,"coverageFeatures":7,"corpusEntries":3,{extra}}}"#
    )
}

fn record_json(run: &str) -> String {
    format!(r#"{{"schemaVersion":"{FUZZ_RECORD_SCHEMA}","runs":[{run}]}}"#)
}

const COMPLETED: &str = r#""state":"completed","stopReason":"growth_observed","crashes":[]"#;

// Trace: TC-111, FR-021-AC-3
#[test]
fn run_record_reads_strictly_and_round_trips_every_state() {
    let text = record_json(&run_json(COMPLETED));
    let record = parse_record(&text).unwrap();
    let again = parse_record(&serde_json::to_string(&record).unwrap()).unwrap();
    assert_eq!(record, again);

    for state in [
        "crash",
        "timeout",
        "out_of_memory",
        "tool_error",
        "resource_refusal",
        "cancelled",
        "not_run",
    ] {
        let crashes = if state == "crash" {
            format!(
                r#"[{{"file":"c","sha256":"{}","conclusive":false}}]"#,
                "a".repeat(64)
            )
        } else {
            "[]".to_owned()
        };
        let run = run_json(&format!(
            r#""state":"{state}","stopReason":"abnormal","crashes":{crashes}"#
        ));
        let parsed = parse_record(&record_json(&run)).unwrap();
        let back: Value = serde_json::to_value(&parsed).unwrap();
        assert_eq!(back["runs"][0]["state"], state);
    }

    let unknown = record_json(&run_json(&format!("{COMPLETED},\"verdict\":\"safe\"")));
    assert_eq!(
        parse_record(&unknown).unwrap_err().code,
        RecordErrorCode::Malformed
    );
    let bad_state = record_json(&run_json(
        r#""state":"passed","stopReason":"plateau","crashes":[]"#,
    ));
    assert_eq!(
        parse_record(&bad_state).unwrap_err().code,
        RecordErrorCode::Malformed
    );
    let schema = text.replace("run-record/v1", "run-record/v0");
    assert_eq!(
        parse_record(&schema).unwrap_err().code,
        RecordErrorCode::Schema
    );
}

// Trace: TC-111, FR-021-AC-3
#[test]
fn run_record_refuses_a_state_that_disagrees_with_its_crashes_or_stop() {
    let crash_without_input = run_json(r#""state":"crash","stopReason":"abnormal","crashes":[]"#);
    assert_eq!(
        parse_record(&record_json(&crash_without_input))
            .unwrap_err()
            .code,
        RecordErrorCode::Inconsistent
    );
    let completed_with_crash = run_json(&format!(
        r#""state":"completed","stopReason":"plateau","crashes":[{{"file":"c","sha256":"{}","conclusive":false}}]"#,
        "a".repeat(64)
    ));
    assert_eq!(
        parse_record(&record_json(&completed_with_crash))
            .unwrap_err()
            .code,
        RecordErrorCode::Inconsistent
    );
    let timeout_as_plateau = run_json(r#""state":"timeout","stopReason":"plateau","crashes":[]"#);
    assert_eq!(
        parse_record(&record_json(&timeout_as_plateau))
            .unwrap_err()
            .code,
        RecordErrorCode::Inconsistent
    );
}

// Trace: TC-111, FR-021-AC-3
#[test]
fn libfuzzer_outcomes_classify_into_distinct_states() {
    let crash = "==1==ERROR: AddressSanitizer\nSUMMARY: AddressSanitizer: heap-buffer-overflow";
    assert_eq!(classify_run(Some(77), crash, false), RunState::Crash);
    assert_eq!(
        classify_run(Some(70), "SUMMARY: libFuzzer: timeout", false),
        RunState::Timeout
    );
    assert_eq!(
        classify_run(Some(71), "SUMMARY: libFuzzer: out-of-memory", false),
        RunState::OutOfMemory
    );
    assert_eq!(classify_run(Some(0), "done", false), RunState::Completed);
    assert_eq!(classify_run(Some(130), "", true), RunState::Completed);
    assert_eq!(classify_run(Some(130), "", false), RunState::Cancelled);
    assert_eq!(
        classify_run(Some(1), "no such binary", false),
        RunState::ToolError
    );
    assert_eq!(classify_run(None, "", false), RunState::ToolError);
}

fn s(elapsed_s: u64, features: u64) -> ProgressSample {
    ProgressSample {
        elapsed_s,
        features,
    }
}

// Trace: TC-110, FR-021-AC-2
#[test]
fn plateau_is_a_pure_decision_over_recorded_progress() {
    let stalled = [s(10, 5), s(100, 9), s(200, 9), s(399, 9), s(400, 9)];
    assert_eq!(
        decide_stop(&stalled, 900, 300).unwrap(),
        StopDecision::Plateau { last_growth_s: 100 }
    );
    // One second short of the window keeps running.
    assert_eq!(
        decide_stop(&stalled[..4], 900, 300).unwrap(),
        StopDecision::Continue
    );
    // Growth inside the last window at the budget is growth observed, not a plateau.
    let growing = [s(10, 5), s(300, 6), s(700, 8), s(900, 9)];
    assert_eq!(
        decide_stop(&growing, 900, 300).unwrap(),
        StopDecision::GrowthObserved { last_growth_s: 900 }
    );
    // A run still growing before the budget keeps running.
    assert_eq!(
        decide_stop(&growing[..3], 900, 300).unwrap(),
        StopDecision::Continue
    );
    // A stall that spans the budget is a plateau, never growth.
    let late = [s(10, 5), s(600, 5), s(900, 5)];
    assert_eq!(
        decide_stop(&late, 900, 300).unwrap(),
        StopDecision::Plateau { last_growth_s: 10 }
    );
    // A plateau needs a full window since the start, however flat the start is.
    assert_eq!(
        decide_stop(&[s(0, 0), s(299, 0)], 900, 300).unwrap(),
        StopDecision::Continue
    );
    assert_eq!(
        decide_stop(&[s(5, 3), s(4, 3)], 900, 300),
        Err(DecisionError::NotMonotone)
    );
    assert_eq!(
        decide_stop(&[s(5, 3), s(6, 2)], 900, 300),
        Err(DecisionError::NotMonotone)
    );
    assert_eq!(decide_stop(&[], 900, 0), Err(DecisionError::EmptyWindow));
}

fn decode_target(bytes: &[u8]) {
    ValidatedCommand::from_json_bytes(bytes, OwnerLimits::owner_max())
        .expect("owner reader accepts the input");
}

fn crash_for(bytes: &[u8]) -> CrashRecord {
    CrashRecord {
        file: "crash-1".to_owned(),
        sha256: sha256_hex(bytes),
        conclusive: false,
    }
}

// Trace: TC-112, FR-021-AC-3
#[test]
fn a_crash_is_conclusive_only_when_the_exact_bytes_reproduce() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ok_bytes = fs::read(root.join("fuzz/corpus/wire_cli_decode/canonical-command")).unwrap();
    let bad_bytes = b"{ not a command".to_vec();

    // A reproducer the decoder refuses reproduces the (test) failure.
    let crash = crash_for(&bad_bytes);
    assert!(matches!(
        confirm_crash(&crash, &bad_bytes, decode_target),
        CrashStatus::Conclusive { .. }
    ));
    // Recorded as a crash but the bytes run clean: not conclusive.
    let clean = crash_for(&ok_bytes);
    assert_eq!(
        confirm_crash(&clean, &ok_bytes, decode_target),
        CrashStatus::NotReproduced
    );
    // Different bytes than the run recorded never confirm, even if they panic.
    assert_eq!(
        confirm_crash(&clean, &bad_bytes, decode_target),
        CrashStatus::ByteMismatch
    );
}

// Trace: TC-112, FR-021-AC-4
#[test]
fn retained_corpus_replays_from_outside_git_and_reports_panics() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().unwrap();
    fs::copy(
        root.join("fuzz/corpus/wire_cli_decode/canonical-command"),
        dir.path().join("keep-ok"),
    )
    .unwrap();
    fs::write(dir.path().join("keep-bad"), b"{").unwrap();
    let replayed = replay_corpus(dir.path(), decode_target).unwrap();
    assert_eq!(replayed.files, 2);
    assert_eq!(replayed.panicked, ["keep-bad"]);
    // A corpus that is not there is an error, never an empty pass.
    assert!(replay_corpus(&dir.path().join("missing"), decode_target).is_err());
}
