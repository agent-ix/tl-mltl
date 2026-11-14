//! Campaign inventory, report and source-bound replay (TL-236).

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use tl_mltl::campaign::{
    aggregate, check_inventory, compare_member, discover_fuzz_targets, parse_members, replay,
    run_campaign, verify_seal, Report, Status,
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

// Trace: TC-121, NFR-004-AC-1
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

fn fuzz_member(command: &str) -> tempfile::TempDir {
    let members = format!(
        r#"{{"schemaVersion": "tl-mltl.campaign-members/v1", "tools": [], "members": [
        {{"id": "f", "technique": "fuzz", "kind": "fuzz",
         "command": ["sh", "-c", {command:?}],
         "determinism": "seeded", "seed": "7", "evidence": {{"env": "EV"}}}}]}}"#
    );
    fixture(&members)
}

#[test]
fn fuzz_evidence_with_a_wrong_schema_no_record_or_unreadable_file_fails() {
    let run = |dir: tempfile::TempDir| {
        let out = tempfile::tempdir().unwrap();
        run_campaign(dir.path(), out.path(), &[])
            .unwrap()
            .members
            .remove(0)
    };
    let wrong = run(fuzz_member(
        r#"printf '{"schemaVersion":"tl-mltl.fuzz-run-record/v0","runs":[]}' > "$EV/r.json""#,
    ));
    assert_eq!(wrong.status, Status::Fail);
    assert!(wrong.detail.unwrap().contains("Schema"));
    let none = run(fuzz_member(r#"printf '{"result":"agree"}' > "$EV/r.json""#));
    assert_eq!(none.status, Status::Fail);
    let empty = run(fuzz_member("true"));
    assert_eq!(empty.status, Status::Fail);
    // A directory named *.json cannot be read as a file.
    let unreadable = run(fuzz_member(r#"mkdir "$EV/r.json""#));
    assert_eq!(unreadable.status, Status::Fail);
}

fn kani_member(command: &str) -> tempfile::TempDir {
    let members = format!(
        r#"{{"schemaVersion": "tl-mltl.campaign-members/v1", "tools": [], "members": [
        {{"id": "k", "technique": "kani", "kind": "kani",
         "command": ["sh", "-c", {command:?}],
         "determinism": "exact", "evidence": {{"env": "EV", "volatileKeys": ["elapsedSeconds"]}}}}]}}"#
    );
    fixture(&members)
}

fn kani_record_json(outcome: &str, satisfied: u32) -> String {
    format!(
        r#"{{"schemaVersion":"tl-mltl.kani-proof-record/v1","proofs":[{{"harness":"m::h","outcome":"{outcome}","kaniVersion":"Kani 0.68.0","cbmcVersion":"6.11.0","coversSatisfied":{satisfied},"coversTotal":2,"elapsedSeconds":3,"bounds":"Proves x.","reproduce":["cargo","kani"]}}]}}"#
    )
}

// Trace: TC-121, FR-023-AC-3
#[test]
fn kani_proof_records_are_bound_into_the_report_and_replay_compares_them() {
    let cmd = format!(
        "cat > \"$EV/p.json\" <<'X'\n{}\nX",
        kani_record_json("verified", 2)
    );
    let dir = kani_member(&cmd);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let member = &report.members[0];
    assert_eq!(member.status, Status::Pass, "{member:?}");
    let proof = &member.kani_proofs[0];
    assert_eq!(proof.harness, "m::h");
    assert_eq!(proof.kani_version, "Kani 0.68.0");
    let mut forged = member.clone();
    forged.kani_proofs[0].outcome = tl_mltl::kani_record::ProofOutcome::Timeout;
    assert!(!compare_member(member, &forged).is_empty());
    let mut other_version = member.clone();
    other_version.kani_proofs[0].kani_version = "Kani 0.69.0".into();
    assert!(!compare_member(member, &other_version).is_empty());
    let mut slower = member.clone();
    slower.kani_proofs[0].elapsed_seconds = 99;
    assert!(compare_member(member, &slower).is_empty());
}

// Trace: TC-121, FR-023-AC-3
#[test]
fn a_proof_that_did_not_verify_fails_the_member_under_its_own_name() {
    let all = [
        ("assertion_failed", "AssertionFailed"),
        ("unwinding_failed", "UnwindingFailed"),
        ("timeout", "Timeout"),
        ("unsupported_construct", "UnsupportedConstruct"),
        ("out_of_memory", "OutOfMemory"),
        ("vacuous_cover", "VacuousCover"),
        ("tool_error", "ToolError"),
    ];
    for (outcome, wanted) in all {
        // `vacuous_cover` requires an unsatisfied cover; the rest need none.
        let satisfied = if outcome == "vacuous_cover" { 1 } else { 0 };
        let cmd = format!(
            "cat > \"$EV/p.json\" <<'X'\n{}\nX",
            kani_record_json(outcome, satisfied)
        );
        let out = tempfile::tempdir().unwrap();
        let report = run_campaign(kani_member(&cmd).path(), out.path(), &[]).unwrap();
        let member = &report.members[0];
        assert_eq!(member.status, Status::Fail, "{outcome}");
        assert!(
            member.detail.as_deref().unwrap().contains(wanted),
            "{outcome}: {:?}",
            member.detail
        );
    }
}

// Trace: TC-121, FR-023-AC-3
#[test]
fn a_listed_harness_without_a_record_fails_the_member() {
    let dir = kani_member(&format!(
        "cat > \"$EV/p.json\" <<'X'\n{}\nX",
        kani_record_json("verified", 2)
    ));
    let list = dir.path().join("scripts");
    fs::create_dir_all(&list).unwrap();
    fs::write(list.join("kani-harnesses.txt"), "m::h\nm::other\n").unwrap();
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let member = &report.members[0];
    assert_eq!(member.status, Status::Fail);
    assert!(member.detail.as_deref().unwrap().contains("m::other"));
    fs::write(list.join("kani-harnesses.txt"), "m::h\n").unwrap();
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    assert_eq!(report.members[0].status, Status::Pass);
}

#[test]
fn a_missing_or_malformed_kani_record_fails_the_member() {
    let run = |cmd: &str| {
        let out = tempfile::tempdir().unwrap();
        run_campaign(kani_member(cmd).path(), out.path(), &[])
            .unwrap()
            .members
            .remove(0)
    };
    assert_eq!(run("true").status, Status::Fail);
    assert_eq!(
        run(r#"printf '{"result":"agree"}' > "$EV/p.json""#).status,
        Status::Fail
    );
    let inconsistent = format!(
        "cat > \"$EV/p.json\" <<'X'\n{}\nX",
        kani_record_json("verified", 1)
    );
    let member = run(&inconsistent);
    assert_eq!(member.status, Status::Fail);
    assert!(member.detail.unwrap().contains("Inconsistent"));
}

fn fuzz_run_json(seed: u64, executions: u64, edges: u64, state: &str) -> String {
    format!(
        r#"{{"target":"t","seed":{seed},"requestedSeconds":9,"observedSeconds":9,
        "plateauWindowSeconds":3,"executions":{executions},"coverageEdges":{edges},
        "coverageFeatures":4,"corpusEntries":5,"state":"{state}","stopReason":"growth_observed",
        "crashes":[]}}"#
    )
}

// Trace: TC-121, NFR-005-AC-1
// A stochastic member that repeats a run keeps each repetition as its own run with its own seed
// and counts, and replay compares every repetition separately.
#[test]
fn stochastic_repetitions_are_kept_separately_and_replayed_one_by_one() {
    let record = format!(
        r#"{{"schemaVersion":"tl-mltl.fuzz-run-record/v1","runs":[{},{}]}}"#,
        fuzz_run_json(7, 11, 2, "completed"),
        fuzz_run_json(8, 40, 6, "completed")
    );
    let members = r#"{"schemaVersion": "tl-mltl.campaign-members/v1", "tools": [], "members": [
        {"id": "f", "technique": "fuzz", "kind": "fuzz",
         "command": ["sh", "-c", "cp rec.json \"$EV/r.json\""],
         "determinism": "seeded", "seed": "7", "evidence": {"env": "EV"}}]}"#;
    let dir = fixture(members);
    fs::write(dir.path().join("rec.json"), record).unwrap();
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "rec"]);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let runs = &report.members[0].fuzz_runs;
    // Two runs, neither merged nor averaged.
    assert_eq!(runs.len(), 2);
    assert_eq!(
        runs.iter()
            .map(|r| (r.seed, r.executions, r.coverage_edges))
            .collect::<Vec<_>>(),
        [(7, 11, 2), (8, 40, 6)]
    );
    let again = tempfile::tempdir().unwrap();
    let (_, diffs) = replay(&report, dir.path(), again.path()).unwrap();
    assert!(diffs.is_empty(), "{diffs:?}");

    let replay_diffs = |retained: &Report| {
        tl_mltl::campaign::replay(retained, dir.path(), again.path())
            .unwrap()
            .1
    };
    // A retained repetition the rerun does not reproduce is a difference.
    let mut missing = report.clone();
    missing.members[0].fuzz_runs.pop();
    tl_mltl::campaign::seal(&mut missing).unwrap();
    assert!(!replay_diffs(&missing).is_empty());
    // So is a retained repetition that ended differently, whichever one it is.
    for index in 0..2 {
        let mut changed = report.clone();
        changed.members[0].fuzz_runs[index].state = tl_mltl::fuzz_record::RunState::Cancelled;
        tl_mltl::campaign::seal(&mut changed).unwrap();
        assert!(!replay_diffs(&changed).is_empty(), "repetition {index}");
    }
    // And one with a different seed.
    let mut reseeded = report.clone();
    reseeded.members[0].fuzz_runs[1].seed = 9;
    tl_mltl::campaign::seal(&mut reseeded).unwrap();
    assert!(!replay_diffs(&reseeded).is_empty());
}
