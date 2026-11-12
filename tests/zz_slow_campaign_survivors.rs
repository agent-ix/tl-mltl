//! TL-371: the slower campaign and `r2u2_live` binary tests (a campaign run spawns processes; the
//! replay reads 1800 recorded cases). They sort last so a mutant a quicker test already kills does not
//! wait for them; a surviving mutant still runs them all.
use std::fs;
use std::path::Path;
use std::process::Command;

use tl_mltl::campaign::{compare_member, run_campaign, Determinism, Status};

fn members(list: &str) -> String {
    format!(
        r#"{{"schemaVersion":"tl-mltl.campaign-members/v1",
"tools":[{{"name":"rustc","command":["rustc","--version"]}}],"members":[{list}]}}"#
    )
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

fn fixture(list: &str) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("campaign")).unwrap();
    fs::write(dir.path().join("campaign/members.json"), members(list)).unwrap();
    fs::write(dir.path().join("Makefile"), "ci: a\n").unwrap();
    fs::write(dir.path().join("Cargo.lock"), "lock\n").unwrap();
    git(dir.path(), &["init", "-q"]);
    git(dir.path(), &["add", "."]);
    git(dir.path(), &["commit", "-q", "-m", "x"]);
    dir
}

fn member(id: &str, command: &str, extra: &str) -> String {
    format!(
        r#"{{"id":"{id}","technique":"t","kind":"test","command":{command},"determinism":"exact"{extra}}}"#
    )
}

// Trace: TC-127
#[test]
fn a_member_result_keeps_its_detail_tools_and_elapsed_time() {
    let list = [
        member("fails", r#"["sh","-c","exit 3"]"#, r#","tools":["rustc"]"#),
        member("slow", r#"["sh","-c","sleep 0.3"]"#, ""),
        member("killed", r#"["sh","-c","kill -9 $$"]"#, ""),
    ]
    .join(",");
    let dir = fixture(&list);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let by = |id: &str| report.members.iter().find(|m| m.id == id).unwrap();
    assert_eq!(by("fails").status, Status::Fail);
    assert_eq!(by("fails").detail.as_deref(), Some("exit 3"));
    assert!(
        by("fails").tools.contains_key("rustc"),
        "{:?}",
        by("fails").tools
    );
    assert!(by("slow").elapsed_ms >= 250, "{}", by("slow").elapsed_ms);
    assert_eq!(by("killed").detail.as_deref(), Some("exit -1"));
}

// Trace: TC-127
#[test]
fn only_a_subset_run_is_labelled_partial() {
    let list = [
        member("a", r#"["rustc","--version"]"#, ""),
        member("b", r#"["rustc","--version"]"#, ""),
    ]
    .join(",");
    let dir = fixture(&list);
    let out = tempfile::tempdir().unwrap();
    let all = run_campaign(dir.path(), out.path(), &[]).unwrap();
    assert_eq!(all.aggregate, "accept");
    let some = run_campaign(dir.path(), out.path(), &["a".to_owned()]).unwrap();
    assert_eq!(some.aggregate, "partial");
}

// Trace: TC-127
#[test]
fn replay_compares_evidence_digests_only_for_exact_members() {
    let list = member("a", r#"["rustc","--version"]"#, "");
    let dir = fixture(&list);
    let out = tempfile::tempdir().unwrap();
    let report = run_campaign(dir.path(), out.path(), &[]).unwrap();
    let old = report.members[0].clone();
    let mut new = old.clone();
    new.evidence.push(tl_mltl::campaign::EvidenceFile {
        name: "e.json".into(),
        sha256: "1".into(),
        canonical_sha256: "1".into(),
        result: None,
    });
    assert!(compare_member(&old, &new)
        .iter()
        .any(|d| d.contains("evidence digests differ")));
    let (mut seeded_old, mut seeded_new) = (old, new);
    seeded_old.determinism = Determinism::Seeded;
    seeded_new.determinism = Determinism::Seeded;
    assert!(!compare_member(&seeded_old, &seeded_new)
        .iter()
        .any(|d| d.contains("evidence digests differ")));
}

fn campaign(dir: &Path, args: &[&str]) -> bool {
    Command::new(env!("CARGO_BIN_EXE_campaign"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
        .status
        .success()
}

// Trace: TC-127
#[test]
fn campaign_run_exits_zero_only_for_a_passing_selection() {
    let pass = |id: &str| member(id, r#"["rustc","--version"]"#, "");
    let fail = |id: &str, extra: &str| member(id, r#"["sh","-c","exit 1"]"#, extra);

    // Everything selected and passing.
    let dir = fixture(&[pass("a"), pass("b")].join(","));
    assert!(campaign(dir.path(), &["run", "--out", "o"]));

    // A failing required member rejects.
    let dir = fixture(&[pass("a"), fail("b", "")].join(","));
    assert!(!campaign(dir.path(), &["run", "--out", "o"]));

    // A subset that passes is accepted even though another member would fail.
    assert!(campaign(dir.path(), &["run", "--out", "o", "a"]));
    // A subset that contains the failing member is not.
    assert!(!campaign(dir.path(), &["run", "--out", "o", "b"]));

    // A failing member that is not required does not gate a subset, but a required one does.
    let dir = fixture(&[pass("a"), fail("b", r#","required":false"#)].join(","));
    assert!(campaign(dir.path(), &["run", "--out", "o", "b"]));
    let dir = fixture(&[pass("a"), pass("b"), fail("c", r#","required":false"#)].join(","));
    assert!(campaign(dir.path(), &["run", "--out", "o", "a", "c"]));
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
fn r2u2_live_exits_zero_for_a_passing_replay_and_two_when_nothing_ran() {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"));
    let dir = tempfile::tempdir().unwrap();
    copy_tree(&repo.join("corpus/r2u2-live-v1/past"), dir.path());
    let replay = Command::new(env!("CARGO_BIN_EXE_r2u2_live"))
        .args(["replay", "--out"])
        .arg(dir.path())
        .current_dir(repo)
        .output()
        .unwrap();
    assert_eq!(
        replay.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert!(dir.path().join("replay-report.json").exists());

    let none = tempfile::tempdir().unwrap();
    let not_run = Command::new(env!("CARGO_BIN_EXE_r2u2_live"))
        .args(["run", "--out"])
        .arg(none.path())
        .env_remove("R2U2_ROOT")
        .current_dir(repo)
        .output()
        .unwrap();
    assert_eq!(not_run.status.code(), Some(2));
}
