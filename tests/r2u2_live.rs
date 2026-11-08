//! Live R2U2 differential harness (TL-217). No test here executes R2U2: the
//! target side is either the retained 4.2 output or a monitor simulated from
//! this crate's own verdicts, used only to exercise parsing, classification and
//! replay.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::Command;

use tl_mltl::r2u2_live::{
    check_tool, classify, generate_grid, group_inputs, parse_r2u2_stdout, plan, read_pin, run,
    CaseStatus, Group, Pin, Source,
};

const REPO: &str = env!("CARGO_MANIFEST_DIR");

#[test]
fn grid_covers_every_operator_depth_interval_kind_and_trace_class() {
    let grid = generate_grid();
    let ids: BTreeSet<_> = grid.iter().map(|c| c.id.clone()).collect();
    assert_eq!(ids.len(), grid.len(), "ids are unique");
    for depth in 1..=3 {
        assert!(grid.iter().any(|c| c.ops.len() == depth), "depth {depth}");
    }
    let groups: BTreeSet<Group> = grid.iter().map(|c| c.group).collect();
    assert_eq!(groups.len(), 12, "3 trace classes x 4 interval kinds");
    for needle in ["once", "historically", "previous", "since", "triggered"] {
        assert!(grid.iter().any(|c| c.id.contains(needle)), "{needle}");
    }
}

#[test]
fn every_case_is_admitted_or_refused_with_a_typed_reason() {
    let plan = plan().unwrap();
    assert_eq!(plan.cases.len(), plan.exported.len());
    let admitted = plan.exported.iter().filter(|e| e.is_ok()).count();
    assert!(admitted > 0 && admitted < plan.cases.len());
    for e in plan.exported.iter().filter_map(|e| e.as_ref().err()) {
        assert!(e.starts_with("TargetOrigin"), "unexpected refusal {e}");
    }
    for (c, e) in plan.cases.iter().zip(&plan.exported) {
        if let Ok((verdicts, _)) = e {
            assert_eq!(verdicts.len(), 8, "{}", c.id);
        }
    }
}

#[test]
fn retained_r2u2_stdout_parses_into_formula_position_verdicts() {
    let text =
        fs::read_to_string(Path::new(REPO).join("corpus/past-c2po-v1/target-4.2/r2u2.stdout"))
            .unwrap();
    let v = parse_r2u2_stdout(&text);
    assert_eq!(v.len(), 21);
    assert_eq!(v.get(&(0, 1)), Some(&true));
    assert_eq!(v.get(&(1, 1)), Some(&false));
    assert!(parse_r2u2_stdout("garbage\n3:x,T\n1:1,?\n").is_empty());
}

#[test]
fn without_r2u2_the_report_is_not_run_and_never_passes() {
    let out = tempfile::tempdir().unwrap();
    let r = run(Path::new(REPO), &Source::None("absent".into()), out.path()).unwrap();
    assert_eq!(r.mode, "tool_unavailable");
    assert_eq!(r.outcome, "not_run");
    assert!(r
        .cases
        .iter()
        .all(|c| matches!(c.status, CaseStatus::Unsupported | CaseStatus::NotRun)));
    assert!(r.cases.iter().any(|c| c.status == CaseStatus::NotRun));
    assert!(out.path().join("inputs").is_dir());
}

#[test]
fn missing_checkout_is_reported_as_unavailable_not_failure_to_compare() {
    let out = tempfile::tempdir().unwrap();
    let nowhere = out.path().join("no-r2u2");
    let r = run(Path::new(REPO), &Source::Live(&nowhere), out.path()).unwrap();
    assert_eq!(r.outcome, "not_run");
    assert!(r.tool_detail.unwrap().contains("not built"));
}

#[test]
fn tool_check_refuses_a_checkout_that_is_not_the_pinned_revision() {
    let dir = tempfile::tempdir().unwrap();
    for f in ["compiler/c2po.py", "monitors/c/build/r2u2"] {
        let p = dir.path().join(f);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, "x").unwrap();
    }
    let git = |a: &[&str]| {
        assert!(Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@e.invalid"])
            .args(a)
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success());
    };
    git(&["init", "-q"]);
    git(&["add", "."]);
    git(&["commit", "-q", "-m", "x"]);
    let pin = read_pin(Path::new(REPO)).unwrap();
    let e = check_tool(dir.path(), &pin).unwrap_err();
    assert!(e.contains("the pin is"), "{e}");
    let wrong = Pin {
        revision: "0".repeat(40),
        ..pin
    };
    assert!(check_tool(dir.path(), &wrong).is_err());
}

/// Stdout of a monitor that returns this crate's own verdicts, optionally
/// flipping one step of one admitted case.
fn simulated_stdout(flip: Option<(&str, usize)>) -> BTreeMap<String, String> {
    let plan = plan().unwrap();
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    let mut next: BTreeMap<String, usize> = BTreeMap::new();
    for (c, e) in plan.cases.iter().zip(&plan.exported) {
        let Ok((verdicts, _)) = e else { continue };
        let g = c.group.name();
        let idx = next.entry(g.clone()).or_insert(0);
        for (pos, v) in verdicts.iter().enumerate() {
            let v = if flip == Some((c.id.as_str(), pos)) {
                !v
            } else {
                *v
            };
            out.entry(g.clone())
                .or_default()
                .push_str(&format!("{idx}:{pos},{}\n", if v { 'T' } else { 'F' }));
        }
        *idx += 1;
    }
    out
}

/// A recorded live run directory: outputs plus the `report.json` a live run
/// would have written (digests of the generated inputs and the given stdout).
fn recorded_dir(stdout: &BTreeMap<String, String>, mode: &str) -> tempfile::TempDir {
    let out = tempfile::tempdir().unwrap();
    let mut rec = run(Path::new(REPO), &Source::None("x".into()), out.path()).unwrap();
    rec.mode = mode.to_owned();
    for g in &mut rec.groups {
        g.stdout_sha256 = stdout
            .get(&g.group)
            .map(|t| tl_mltl::campaign::sha256_hex(t.as_bytes()));
    }
    fs::write(
        out.path().join("report.json"),
        serde_json::to_vec(&rec).unwrap(),
    )
    .unwrap();
    for (g, text) in stdout {
        let d = out.path().join("outputs").join(g);
        fs::create_dir_all(&d).unwrap();
        fs::write(d.join("r2u2.stdout"), text).unwrap();
    }
    out
}

fn replay_with(stdout: &BTreeMap<String, String>) -> tl_mltl::r2u2_live::Report {
    let out = recorded_dir(stdout, "ran");
    run(Path::new(REPO), &Source::Recorded, out.path()).unwrap()
}

#[test]
fn replay_refuses_unrecorded_unlive_or_altered_outputs() {
    let stdout = simulated_stdout(None);
    let none = tempfile::tempdir().unwrap();
    assert!(run(Path::new(REPO), &Source::Recorded, none.path()).is_err());
    let notlive = recorded_dir(&stdout, "tool_unavailable");
    let e = run(Path::new(REPO), &Source::Recorded, notlive.path()).unwrap_err();
    assert!(e.contains("not a live run"), "{e}");
    let altered = recorded_dir(&stdout, "ran");
    let g = stdout.keys().next().unwrap();
    fs::write(
        altered.path().join("outputs").join(g).join("r2u2.stdout"),
        "0:0,T\n",
    )
    .unwrap();
    let e = run(Path::new(REPO), &Source::Recorded, altered.path()).unwrap_err();
    assert!(e.contains("does not match"), "{e}");
}

#[test]
fn a_run_that_compares_nothing_never_passes() {
    let r = replay_with(&BTreeMap::new());
    assert_ne!(r.outcome, "pass");
}

#[test]
fn replay_of_agreeing_recorded_output_passes_and_pins_digests() {
    let r = replay_with(&simulated_stdout(None));
    assert_eq!(r.mode, "replayed");
    assert_eq!(r.outcome, "pass", "{:?}", r.summary);
    assert!(r.summary.get("agree").copied().unwrap_or(0) > 0);
    assert!(r
        .groups
        .iter()
        .any(|g| g.stdout_sha256.is_some() && g.cases > 0));
    assert_eq!(r.pin.revision, read_pin(Path::new(REPO)).unwrap().revision);
}

#[test]
fn a_disagreement_outside_the_origin_window_fails_the_run() {
    let plan = plan().unwrap();
    let (case, _) = plan
        .cases
        .iter()
        .zip(&plan.exported)
        .find(|(_, e)| e.is_ok())
        .unwrap();
    let r = replay_with(&simulated_stdout(Some((&case.id, 7))));
    assert_eq!(r.outcome, "fail");
    let row = r.cases.iter().find(|c| c.id == case.id).unwrap();
    assert_eq!(row.status, CaseStatus::Unexplained);
    assert_eq!(row.differing_steps, vec![7]);
}

#[test]
fn a_silent_target_step_is_missing_not_agreement() {
    let mut s = simulated_stdout(None);
    let g = s.keys().next().unwrap().clone();
    let first = s[&g].lines().skip(1).collect::<Vec<_>>().join("\n");
    s.insert(g, first);
    let r = replay_with(&s);
    assert_eq!(r.outcome, "fail");
    assert!(r.summary.contains_key("missing_target"));
}

#[test]
fn disagreement_inside_the_look_back_window_is_a_documented_profile_difference() {
    let case = generate_grid()
        .into_iter()
        .find(|c| c.origin_reach() > 0 && c.id.starts_with("once/fromzero/alltrue"))
        .unwrap();
    let expected = [true; 8];
    let mut target: BTreeMap<(usize, u64), bool> = (0..8).map(|i| ((0, i), true)).collect();
    target.insert((0, 0), false);
    let row = classify(&case, &expected, 0, &target);
    assert_eq!(row.status, CaseStatus::OriginProfileDifference);
    target.insert((0, 7), false);
    assert_eq!(
        classify(&case, &expected, 0, &target).status,
        CaseStatus::Unexplained
    );
}

#[test]
fn group_inputs_list_one_expression_per_admitted_case() {
    let plan = plan().unwrap();
    let group = plan.cases[0].group;
    let admitted: Vec<_> = plan
        .cases
        .iter()
        .zip(&plan.exported)
        .filter(|(c, _)| c.group == group)
        .filter_map(|(c, e)| e.as_ref().ok().map(|(_, x)| (c, x.clone())))
        .collect();
    let inputs = group_inputs(group, &admitted);
    assert!(inputs.spec.starts_with("INPUT\n  p,q: bool;\nPTSPEC\n"));
    assert_eq!(inputs.spec.lines().count(), 3 + admitted.len());
    assert_eq!(inputs.trace.lines().count(), 9);
}
