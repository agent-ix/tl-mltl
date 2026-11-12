//! TL-371: behavior tests for the Kani and fuzz record parsers, which the first full cargo-mutants
//! run showed were not asserted one operand at a time.

use tl_mltl::fuzz_record::{
    decide_stop, parse_record, DecisionError, ProgressSample, RecordErrorCode, StopDecision,
    FUZZ_RECORD_SCHEMA,
};
use tl_mltl::kani_record::{classify_proof, parse_covers, ProofOutcome};

// Trace: TC-111
#[test]
fn cover_counts_need_the_exact_line_shape() {
    assert_eq!(
        parse_covers("** 3 of 5 cover properties satisfied"),
        Some((3, 5))
    );
    assert_eq!(
        parse_covers("noise\n** 0 of 2 cover properties satisfied\n"),
        Some((0, 2))
    );
    // Each of the three anchor words alone is wrong.
    assert_eq!(parse_covers("x 3 of 5 cover properties satisfied"), None);
    assert_eq!(parse_covers("** 3 off 5 cover properties satisfied"), None);
    assert_eq!(parse_covers("** 3 of 5 covers properties satisfied"), None);
    assert_eq!(
        parse_covers("** three of 5 cover properties satisfied"),
        None
    );
    assert_eq!(parse_covers("no cover line"), None);
}

// Trace: TC-111
#[test]
fn proofs_are_classified_by_the_first_matching_class() {
    let ok = "VERIFICATION:- SUCCESSFUL\n** 2 of 2 cover properties satisfied";
    assert_eq!(classify_proof(Some(0), ok, false), ProofOutcome::Verified);
    assert_eq!(classify_proof(Some(0), ok, true), ProofOutcome::Timeout);
    // A cover that is not fully satisfied, or one with no properties, is vacuous.
    let partial = "VERIFICATION:- SUCCESSFUL\n** 1 of 2 cover properties satisfied";
    assert_eq!(
        classify_proof(Some(0), partial, false),
        ProofOutcome::VacuousCover
    );
    let none = "VERIFICATION:- SUCCESSFUL\n** 0 of 0 cover properties satisfied";
    assert_eq!(
        classify_proof(Some(0), none, false),
        ProofOutcome::VacuousCover
    );
    assert_eq!(
        classify_proof(Some(0), "VERIFICATION:- SUCCESSFUL", false),
        ProofOutcome::VacuousCover
    );
    // Memory exhaustion is recognized by each of its four signs alone.
    assert_eq!(
        classify_proof(Some(137), "plain", false),
        ProofOutcome::OutOfMemory
    );
    assert_eq!(
        classify_proof(Some(1), "Out Of Memory", false),
        ProofOutcome::OutOfMemory
    );
    assert_eq!(
        classify_proof(Some(1), "std::bad_alloc", false),
        ProofOutcome::OutOfMemory
    );
    assert_eq!(
        classify_proof(Some(1), "Cannot allocate memory", false),
        ProofOutcome::OutOfMemory
    );
    assert_eq!(
        classify_proof(Some(1), "something else", false),
        ProofOutcome::ToolError
    );
}

fn s(elapsed_s: u64, features: u64) -> ProgressSample {
    ProgressSample {
        elapsed_s,
        features,
    }
}

// Trace: TC-110
#[test]
fn progress_samples_may_repeat_a_second_but_never_go_back() {
    assert!(decide_stop(&[s(10, 5), s(10, 5)], 900, 300).is_ok());
    assert!(decide_stop(&[s(10, 5), s(20, 5)], 900, 300).is_ok());
    assert_eq!(
        decide_stop(&[s(10, 5), s(9, 5)], 900, 300).unwrap_err(),
        DecisionError::NotMonotone
    );
    assert_eq!(
        decide_stop(&[s(10, 5), s(20, 4)], 900, 300).unwrap_err(),
        DecisionError::NotMonotone
    );
    let _ = StopDecision::Continue;
}

// Trace: TC-111
#[test]
fn a_conclusive_crash_needs_a_full_reproducer_digest() {
    let record = |conclusive: bool, sha: &str| {
        format!(
            r#"{{"schemaVersion":"{FUZZ_RECORD_SCHEMA}","runs":[{{"target":"t","seed":1,
            "requestedSeconds":900,"observedSeconds":900,"plateauWindowSeconds":300,
            "executions":1,"coverageEdges":1,"coverageFeatures":1,"corpusEntries":1,
            "state":"crash","stopReason":"abnormal",
            "crashes":[{{"file":"c","sha256":"{sha}","conclusive":{conclusive}}}]}}]}}"#
        )
    };
    let full = "a".repeat(64);
    assert!(parse_record(&record(true, &full)).is_ok());
    assert!(parse_record(&record(false, &full)).is_ok());
    assert!(parse_record(&record(false, "abc")).is_ok());
    // A short digest is refused only when the crash claims to be conclusive.
    let refused = parse_record(&record(true, "abc"));
    assert!(refused.is_err(), "{refused:?}");
    let _ = RecordErrorCode::Inconsistent;
}

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use tl_mltl::r2u2_live::{check_tool, classify, generate_grid, run, CaseStatus, Op, Pin, Source};

// Trace: TC-100
#[test]
fn the_origin_reach_of_a_case_adds_one_per_previous_and_the_bound_per_other_operator() {
    for case in generate_grid() {
        let bound = match case.id.split('/').nth(1).unwrap() {
            "zero" => 0,
            "fromzero" | "point" => 2,
            "window" => 3,
            other => panic!("unknown interval kind {other}"),
        };
        let expected: u64 = case
            .ops
            .iter()
            .map(|op| if *op == Op::Previous { 1 } else { bound })
            .sum();
        assert_eq!(case.origin_reach(), expected, "{}", case.id);
    }
}

// Trace: TC-100
#[test]
fn a_difference_at_the_reach_is_not_a_profile_difference() {
    let case = generate_grid()
        .into_iter()
        .find(|c| c.id.starts_with("once/fromzero/alltrue"))
        .unwrap();
    assert_eq!(case.origin_reach(), 2);
    let expected = [true; 8];
    let target_with = |flip: u64| -> BTreeMap<(usize, u64), bool> {
        (0..8).map(|i| ((0, i), i != flip)).collect()
    };
    // Inside the reach (position 1 < 2): documented difference. At the reach: unexplained.
    let inside = classify(&case, &expected, 0, &target_with(1));
    assert_eq!(inside.status, CaseStatus::OriginProfileDifference);
    assert_eq!(inside.target, "TFTTTTTT");
    assert_eq!(inside.expected, "TTTTTTTT");
    let at = classify(&case, &expected, 0, &target_with(2));
    assert_eq!(at.status, CaseStatus::Unexplained);
    assert_eq!(at.differing_steps, vec![2]);
}

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

fn checkout(with_compiler: bool, with_binary: bool) -> (tempfile::TempDir, Pin) {
    let dir = tempfile::tempdir().unwrap();
    if with_compiler {
        fs::create_dir_all(dir.path().join("compiler")).unwrap();
        fs::write(dir.path().join("compiler/c2po.py"), "x").unwrap();
    }
    if with_binary {
        fs::create_dir_all(dir.path().join("monitors/c/build")).unwrap();
        fs::write(dir.path().join("monitors/c/build/r2u2"), "bin").unwrap();
    }
    fs::write(dir.path().join("README"), "x").unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "x"]);
    let revision = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(dir.path())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    let pin = Pin {
        repository: "x".into(),
        revision,
    };
    (dir, pin)
}

// Trace: TC-100
#[test]
fn a_usable_checkout_reports_its_identities_and_each_defect_alone_is_refused() {
    let (dir, pin) = checkout(true, true);
    let observed = check_tool(dir.path(), &pin).unwrap();
    assert_eq!(observed["r2u2Revision"], pin.revision);
    assert!(observed["python"].starts_with("Python"), "{observed:?}");
    assert_eq!(observed["r2u2Binary"].len(), 64);

    let (no_compiler, pin) = checkout(false, true);
    assert!(check_tool(no_compiler.path(), &pin)
        .unwrap_err()
        .contains("not built"));
    let (no_binary, pin) = checkout(true, false);
    assert!(check_tool(no_binary.path(), &pin)
        .unwrap_err()
        .contains("not built"));

    fs::write(dir.path().join("untracked"), "x").unwrap();
    git(dir.path(), &["add", "untracked"]);
    let dirty = check_tool(dir.path(), &pin_of(&dir)).unwrap_err();
    assert!(dirty.contains("uncommitted"), "{dirty}");
}

fn pin_of(dir: &tempfile::TempDir) -> Pin {
    let revision = String::from_utf8(
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(dir.path())
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    Pin {
        repository: "x".into(),
        revision,
    }
}

// Trace: TC-100
#[test]
fn refusal_counts_add_up_to_the_unsupported_cases() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().unwrap();
    copy_tree(&repo.join("corpus/r2u2-live-v1/past"), dir.path());
    let report = run(repo, &Source::Recorded, dir.path()).unwrap();
    let unsupported = report.summary["unsupported"];
    assert_eq!(report.refusals.values().sum::<usize>(), unsupported);
    assert!(report.refusals.len() > 1, "{:?}", report.refusals);
    let shown: String = report.cases.iter().map(|c| c.target.clone()).collect();
    assert!(shown.contains('T') && shown.contains('F'));
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

// Trace: TC-100
#[test]
fn future_traces_enumerate_every_two_proposition_word_in_order() {
    let traces = tl_mltl::r2u2_future::future_traces();
    let len = traces[0].len();
    assert_eq!(traces.len(), 4usize.pow(u32::try_from(len).unwrap()));
    assert!(traces[0].iter().all(|row| *row == [false, false]));
    assert_eq!(traces[1][0], [true, false]);
    assert_eq!(traces[2][0], [false, true]);
    assert_eq!(traces[3][0], [true, true]);
    assert_eq!(traces[1][1], [false, false]);
    assert_eq!(traces[4][1], [true, false]);
    assert_eq!(traces[8][1], [false, true]);
    assert_eq!(traces[12][1], [true, true]);
}

// Trace: TC-100
#[test]
fn a_run_with_no_tool_marks_admitted_cases_not_run_with_an_empty_target() {
    let out = tempfile::tempdir().unwrap();
    let report = run(
        Path::new(env!("CARGO_MANIFEST_DIR")),
        &Source::None("x".into()),
        out.path(),
    )
    .unwrap();
    let not_run: Vec<_> = report
        .cases
        .iter()
        .filter(|c| c.status == CaseStatus::NotRun)
        .collect();
    assert!(!not_run.is_empty());
    assert!(not_run.iter().all(|c| c.target.is_empty()));
}

// Trace: TC-100
#[test]
fn a_replay_refuses_when_either_input_digest_alone_differs() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    for field in ["specSha256", "traceSha256"] {
        let dir = tempfile::tempdir().unwrap();
        copy_tree(&repo.join("corpus/r2u2-live-v1/past"), dir.path());
        let path = dir.path().join("report.json");
        let mut value: serde_json::Value =
            serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value["groups"][0][field] = serde_json::json!("0".repeat(64));
        fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
        let error = run(repo, &Source::Recorded, dir.path()).unwrap_err();
        assert!(
            error.contains("no longer generates the inputs"),
            "{field}: {error}"
        );
    }
}
