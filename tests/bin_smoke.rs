//! Subprocess smoke tests for the `campaign` and `r2u2_live` binaries.
//!
//! Both binaries are thin drivers over library code that has its own tests, so
//! nothing in-process exercises their argument handling, exit codes or the report
//! files they write. Each test runs the built binary and asserts the exit status
//! and the shape of what it writes. `campaign` runs against a throwaway git
//! repository with a temporary members file; `r2u2_live` runs with no R2U2
//! checkout, which is the one configuration that needs no external tool.

use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

use serde_json::Value;

fn git(dir: &Path, args: &[&str]) {
    let ok = Command::new("git")
        .args([
            "-c",
            "user.name=t",
            "-c",
            "user.email=t@example.invalid",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap()
        .success();
    assert!(ok, "git {args:?}");
}

fn campaign_repo(members: &str) -> tempfile::TempDir {
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

fn campaign(repo: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_campaign"))
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

fn members(command: &str) -> String {
    format!(
        r#"{{
  "schemaVersion": "tl-mltl.campaign-members/v1",
  "tools": [],
  "members": [
    {{"id": "probe", "technique": "t", "kind": "test",
      "command": ["sh", "-c", "{command}"], "determinism": "exact"}}
  ]
}}"#
    )
}

fn report(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

// Trace: TC-014
#[test]
fn campaign_inventory_run_and_replay_exit_zero_on_a_passing_member() {
    let repo = campaign_repo(&members("true"));
    let out = tempfile::tempdir().unwrap();
    let out_arg = out.path().to_str().unwrap();

    let inventory = campaign(repo.path(), &["inventory"]);
    assert_eq!(
        inventory.status.code(),
        Some(0),
        "{}",
        text(&inventory.stderr)
    );
    assert!(text(&inventory.stdout).contains("1 members, complete"));

    let run = campaign(repo.path(), &["run", "--out", out_arg]);
    assert_eq!(run.status.code(), Some(0), "{}", text(&run.stderr));
    assert!(text(&run.stdout).contains("aggregate=accept"));
    let retained = report(&out.path().join("report.json"));
    assert_eq!(retained["schemaVersion"], "tl-mltl.campaign-run/v1");
    assert_eq!(retained["aggregate"], "accept");
    assert_eq!(retained["members"][0]["id"], "probe");
    assert_eq!(retained["members"][0]["status"], "pass");
    assert_eq!(retained["members"][0]["exitCode"], 0);
    assert_eq!(retained["reportSha256"].as_str().unwrap().len(), 64);

    let replay_out = tempfile::tempdir().unwrap();
    let replay = campaign(
        repo.path(),
        &[
            "replay",
            out.path().join("report.json").to_str().unwrap(),
            "--out",
            replay_out.path().to_str().unwrap(),
        ],
    );
    assert_eq!(replay.status.code(), Some(0), "{}", text(&replay.stderr));
    assert!(text(&replay.stdout).contains("replay reproduces the retained report"));
    let attempt = report(&replay_out.path().join("replay-report.json"));
    assert_eq!(attempt["aggregate"], "accept");
}

// Trace: TC-014
#[test]
fn campaign_exits_one_with_a_message_on_each_refusal() {
    let repo = campaign_repo(&members("exit 3"));
    let out = tempfile::tempdir().unwrap();
    let out_arg = out.path().to_str().unwrap();

    let usage = campaign(repo.path(), &[]);
    assert_eq!(usage.status.code(), Some(1));
    assert!(text(&usage.stderr).contains("usage: campaign"));

    let failing = campaign(repo.path(), &["run", "--out", out_arg]);
    assert_eq!(failing.status.code(), Some(1));
    assert!(text(&failing.stderr).contains("aggregate reject"));
    let retained = report(&out.path().join("report.json"));
    assert_eq!(retained["aggregate"], "reject");
    assert_eq!(retained["members"][0]["status"], "fail");
    assert_eq!(retained["members"][0]["exitCode"], 3);

    let unknown = campaign(repo.path(), &["run", "--out", out_arg, "no-such-member"]);
    assert_eq!(unknown.status.code(), Some(1));
    assert!(text(&unknown.stderr).contains("unknown member no-such-member"));

    let no_path = campaign(repo.path(), &["replay"]);
    assert_eq!(no_path.status.code(), Some(1));
    assert!(text(&no_path.stderr).contains("replay needs a report path"));

    let bare = tempfile::tempdir().unwrap();
    let missing = campaign(bare.path(), &["inventory"]);
    assert_eq!(missing.status.code(), Some(1));
    assert!(text(&missing.stderr).contains("campaign:"));
}

fn r2u2_live(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_r2u2_live"))
        .args(args)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env_remove("R2U2_ROOT")
        .output()
        .unwrap()
}

// Trace: TC-014
#[test]
fn r2u2_live_without_a_checkout_writes_a_not_run_report_and_exits_two() {
    for suite in ["past", "future"] {
        let out = tempfile::tempdir().unwrap();
        let output = r2u2_live(&[
            "run",
            "--suite",
            suite,
            "--out",
            out.path().to_str().unwrap(),
        ]);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{suite}: {}",
            text(&output.stderr)
        );
        assert!(text(&output.stdout).contains("outcome=not_run"), "{suite}");
        let retained = report(&out.path().join("report.json"));
        assert_eq!(retained["suite"], suite);
        assert_eq!(
            retained["schemaVersion"],
            format!("tl-mltl.r2u2-{suite}-differential/v1")
        );
        assert_eq!(retained["mode"], "tool_unavailable");
        assert_eq!(retained["outcome"], "not_run");
        for key in ["pin", "observed", "summary", "cases", "nonClaims"] {
            assert!(retained.get(key).is_some(), "{suite}: report lacks {key}");
        }
        assert!(out.path().join("inputs").is_dir(), "{suite}");
        assert!(out.path().join("outputs").is_dir(), "{suite}");

        // A report with no live run behind it cannot be replayed.
        let replay = r2u2_live(&[
            "replay",
            "--suite",
            suite,
            "--out",
            out.path().to_str().unwrap(),
        ]);
        assert_eq!(replay.status.code(), Some(1), "{suite}");
        assert!(text(&replay.stderr).contains("not a live run"), "{suite}");
    }
}

// Trace: TC-014
#[test]
fn r2u2_live_probe_and_usage_refusals_use_distinct_exit_codes() {
    let usage = r2u2_live(&["bogus"]);
    assert_eq!(usage.status.code(), Some(1));
    assert!(text(&usage.stderr).contains("usage: r2u2_live"));

    let absent = r2u2_live(&["probe"]);
    assert_eq!(absent.status.code(), Some(2));
    assert!(text(&absent.stderr).contains("no R2U2 checkout"));

    let empty = tempfile::tempdir().unwrap();
    let unusable = r2u2_live(&["probe", "--r2u2-root", empty.path().to_str().unwrap()]);
    assert_eq!(unusable.status.code(), Some(2));
    assert!(text(&unusable.stderr).contains("r2u2_live:"));
}
