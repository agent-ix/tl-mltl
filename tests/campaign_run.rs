//! Campaign inventory, report and source-bound replay (TL-236).

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use tl_mltl::campaign::{
    aggregate, check_inventory, discover_fuzz_targets, parse_members, replay, run_campaign,
    verify_seal, Report, Status,
};

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

const MEMBERS: &str = r#"{
  "schemaVersion": "tl-mltl.campaign-members/v1",
  "tools": [
    {"name": "rustc", "command": ["rustc", "--version"]},
    {"name": "absent", "command": ["rustc", "--no-such-flag"]}
  ],
  "members": [
    {"id": "ok", "technique": "t", "kind": "test", "command": ["rustc", "--version"],
     "tools": ["rustc"], "determinism": "exact"},
    {"id": "sweep", "technique": "t", "kind": "sweep",
     "command": ["sh", "-c", "printf '{\"result\":\"agree\",\"runtimeMs\":%s,\"n\":3}' $$ > \"$EV/a.json\""],
     "determinism": "exact",
     "evidence": {"env": "EV", "volatileKeys": ["runtimeMs"]}},
    {"id": "needs-absent", "technique": "t", "kind": "kani", "command": ["rustc", "--version"],
     "tools": ["absent"], "determinism": "exact"},
    {"id": "later", "technique": "t", "kind": "mutation", "state": "planned",
     "ticket": "TL-231", "determinism": "seeded"}
  ]
}"#;

fn fixture(members: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("campaign")).unwrap();
    fs::write(dir.path().join("campaign/members.json"), members).unwrap();
    fs::write(dir.path().join("Makefile"), "ci: a\n").unwrap();
    fs::write(dir.path().join("Cargo.lock"), "lock\n").unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "x"]);
    dir
}

fn sets(names: &[&str]) -> BTreeSet<String> {
    names.iter().map(|s| (*s).to_owned()).collect()
}

// Trace: TC-127
#[test]
fn real_member_list_covers_every_evidence_target() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let file =
        parse_members(&fs::read_to_string(root.join("campaign/members.json")).unwrap()).unwrap();
    let findings = check_inventory(
        &file,
        &fs::read_to_string(root.join("Makefile")).unwrap(),
        &discover_fuzz_targets(root),
    );
    assert!(findings.is_empty(), "{findings:?}");
}

// Trace: TC-127
#[test]
fn inventory_refuses_uncovered_targets_and_duplicates() {
    let file = parse_members(MEMBERS).unwrap();
    let mk = "kani-check:\n\tx\nmutants:\n\tx\n";
    let f = check_inventory(&file, mk, &sets(&["new_target"]));
    assert!(f.iter().any(|x| x.contains("kani-check")), "{f:?}");
    assert!(f.iter().any(|x| x.contains("mutants")), "{f:?}");
    assert!(f.iter().any(|x| x.contains("new_target")), "{f:?}");
    let dup = MEMBERS.replace("\"id\": \"later\"", "\"id\": \"ok\"");
    let f = check_inventory(&parse_members(&dup).unwrap(), "", &sets(&[]));
    assert!(f.iter().any(|x| x.contains("duplicate")), "{f:?}");
}

// Trace: TC-127
#[test]
fn run_records_every_member_and_never_passes_a_missing_one() {
    let dir = fixture(MEMBERS);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let st: Vec<_> = report.members.iter().map(|m| m.status).collect();
    assert_eq!(
        st,
        [
            Status::Pass,
            Status::Pass,
            Status::Unavailable,
            Status::NotRegistered
        ]
    );
    assert_eq!(report.aggregate, "incomplete");
    assert_eq!(report.members[1].evidence.len(), 1);
    assert!(verify_seal(&report));
    assert!(!report.source.dirty);
}

#[test]
fn failed_required_member_rejects() {
    let bad = MEMBERS.replace(
        "\"rustc\", \"--version\"],\n     \"tools\": [\"rustc\"]",
        "\"rustc\", \"--no-such-flag\"],\n     \"tools\": [\"rustc\"]",
    );
    let dir = fixture(&bad);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    assert_eq!(report.members[0].status, Status::Fail);
    assert_eq!(aggregate(&report.members), "reject");
}

// Trace: TC-103, NFR-004-AC-1
#[test]
fn replay_reproduces_exact_members_despite_volatile_evidence() {
    let dir = fixture(MEMBERS);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let again = tempfile::tempdir().unwrap();
    let (attempt, diffs) = replay(&report, dir.path(), again.path()).unwrap();
    assert!(diffs.is_empty(), "{diffs:?}");
    // The raw evidence differs (pid in runtimeMs); the canonical digest does not.
    assert_ne!(
        report.members[1].evidence[0].sha256,
        attempt.members[1].evidence[0].sha256
    );
}

// Trace: NFR-004-AC-1
#[test]
fn replay_refuses_a_moved_dirty_or_tampered_source() {
    let dir = fixture(MEMBERS);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let scratch = tempfile::tempdir().unwrap();

    let mut tampered: Report = report.clone();
    tampered.members[0].elapsed_ms += 1;
    let e = replay(&tampered, dir.path(), scratch.path()).unwrap_err();
    assert!(e.contains("digest"), "{e}");

    fs::write(dir.path().join("Cargo.lock"), "other\n").unwrap();
    let e = replay(&report, dir.path(), scratch.path()).unwrap_err();
    assert!(e.contains("uncommitted") && e.contains("Cargo.lock"), "{e}");

    git(dir.path(), &["commit", "-qam", "move"]);
    let e = replay(&report, dir.path(), scratch.path()).unwrap_err();
    assert!(e.contains("bound to"), "{e}");
}

#[test]
fn replay_reports_a_changed_result() {
    let dir = fixture(MEMBERS);
    let out = tempfile::tempdir().unwrap();
    let mut report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    report.members[0].status = Status::Fail;
    tl_mltl::campaign::seal(&mut report).unwrap();
    let again = tempfile::tempdir().unwrap();
    let (_, diffs) = replay(&report, dir.path(), again.path()).unwrap();
    assert_eq!(diffs.len(), 1, "{diffs:?}");
}

#[test]
fn unparseable_evidence_fails_the_member() {
    let bad = r#"{
      "schemaVersion": "tl-mltl.campaign-members/v1",
      "tools": [],
      "members": [
        {"id": "garbled", "technique": "t", "kind": "sweep",
         "command": ["sh", "-c", "printf '{\"result\":\"agree\"}' > \"$EV/a.json\"; printf 'not json' > \"$EV/b.json\""],
         "determinism": "exact", "evidence": {"env": "EV"}}
      ]
    }"#;
    let dir = fixture(bad);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    assert_eq!(
        report.members[0].status,
        Status::Fail,
        "{:?}",
        report.members[0]
    );
    assert_eq!(report.aggregate, "reject");
}

fn fuzz_fixture(state: &str, stop: &str) -> tempfile::TempDir {
    let record = format!(
        r#"{{"schemaVersion":"tl-mltl.fuzz-run-record/v1","runs":[{{"target":"t","seed":7,
        "requestedSeconds":9,"observedSeconds":9,"plateauWindowSeconds":3,"executions":11,
        "coverageEdges":2,"coverageFeatures":4,"corpusEntries":5,"state":"{state}",
        "stopReason":"{stop}","crashes":[]}}]}}"#
    );
    let members = r#"{"schemaVersion": "tl-mltl.campaign-members/v1", "tools": [], "members": [
        {"id": "f", "technique": "fuzz", "kind": "fuzz",
         "command": ["sh", "-c", "cp rec.json \"$EV/r.json\""],
         "determinism": "seeded", "seed": "7", "evidence": {"env": "EV"}}]}"#;
    let dir = fixture(members);
    fs::write(dir.path().join("rec.json"), record).unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "rec"]);
    dir
}

// Trace: TC-121
#[test]
fn fuzz_run_records_are_bound_into_the_report_and_replay_compares_them() {
    let dir = fuzz_fixture("completed", "growth_observed");
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    assert_eq!(report.members[0].status, Status::Pass);
    let run = &report.members[0].fuzz_runs[0];
    assert_eq!((run.seed, run.executions, run.coverage_edges), (7, 11, 2));
    assert_eq!(run.observed_seconds, 9);
    assert!(verify_seal(&report));

    let again = tempfile::tempdir().unwrap();
    let (_, diffs) = replay(&report, dir.path(), again.path()).unwrap();
    assert!(diffs.is_empty(), "{diffs:?}");

    // A retained report whose run seed was edited no longer matches its seal.
    let mut forged = report.clone();
    forged.members[0].fuzz_runs[0].seed = 8;
    assert!(!verify_seal(&forged));
    // A rerun that ends in a different state is reported as a difference.
    let mut was_cancelled = report.clone();
    was_cancelled.members[0].fuzz_runs[0].state = tl_mltl::fuzz_record::RunState::Cancelled;
    // The retained seal is irrelevant to the comparison: tamper, then reseal.
    tl_mltl::campaign::seal(&mut was_cancelled).unwrap();
    let (_, diffs) = replay(&was_cancelled, dir.path(), again.path()).unwrap();
    assert!(!diffs.is_empty());
}

#[test]
fn a_fuzz_run_that_did_not_complete_fails_the_member() {
    let dir = fuzz_fixture("timeout", "abnormal");
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    assert_eq!(report.members[0].status, Status::Fail);
    assert_eq!(aggregate(&report.members), "reject");
}

#[test]
fn a_malformed_fuzz_record_fails_the_member() {
    let dir = fuzz_fixture("crash", "abnormal");
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    assert_eq!(report.members[0].status, Status::Fail);
    assert!(report.members[0].fuzz_runs.is_empty());
}
