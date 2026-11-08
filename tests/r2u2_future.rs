//! Future-time live R2U2 differential harness (TL-237). R2U2 is not executed
//! here; the target side is the retained 4.2 output or a monitor simulated from
//! this crate's own verdicts.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use tl_mltl::r2u2_future::{
    classify_step, future_corpus, future_expression, future_traces, reference_at, run_future,
    FUTURE_TRACE_LEN,
};
use tl_mltl::r2u2_live::{CaseStatus, Report, Source};

const REPO: &str = env!("CARGO_MANIFEST_DIR");

fn trace_name(trace: &[[bool; 2]]) -> String {
    trace
        .iter()
        .map(|r| format!("{}{}", u8::from(r[0]), u8::from(r[1])))
        .collect::<Vec<_>>()
        .join("-")
}

#[test]
fn corpus_is_exhaustive_over_future_operators_intervals_and_traces() {
    let corpus = future_corpus();
    assert_eq!(corpus.len(), 4 * 6, "F G U R x six intervals within 0..=2");
    assert_eq!(future_traces().len(), 4usize.pow(FUTURE_TRACE_LEN as u32));
    for c in &corpus {
        let e = future_expression(c).unwrap();
        assert!(e.contains("p0"), "{e}");
    }
    assert_eq!(future_expression(&corpus[0]).unwrap(), "F[0,0](p0)");
}

#[test]
fn step_classification_never_collapses_pending_into_agreement_with_a_verdict() {
    assert_eq!(classify_step(Some(true), Some(true)), CaseStatus::Agree);
    assert_eq!(
        classify_step(Some(true), Some(false)),
        CaseStatus::Unexplained
    );
    assert_eq!(classify_step(Some(false), None), CaseStatus::MissingTarget);
    assert_eq!(classify_step(None, None), CaseStatus::Agree);
    assert_eq!(
        classify_step(None, Some(true)),
        CaseStatus::PendingResolvedByTarget
    );
}

#[test]
fn without_r2u2_the_future_report_is_not_run() {
    let out = tempfile::tempdir().unwrap();
    let r = run_future(Path::new(REPO), &Source::None("absent".into()), out.path()).unwrap();
    assert_eq!(r.suite, "future");
    assert_eq!(r.outcome, "not_run");
    assert_eq!(r.groups.len(), 64);
    assert!(r.cases.iter().all(|c| c.status == CaseStatus::NotRun));
}

/// Stdout of a monitor returning this crate's conclusive verdicts for every
/// formula and time, optionally flipping one.
fn simulated(flip: Option<(&str, usize, u64)>) -> BTreeMap<String, String> {
    let corpus = future_corpus();
    let mut out = BTreeMap::new();
    for trace in future_traces() {
        let name = trace_name(&trace);
        let mut text = String::new();
        for (idx, case) in corpus.iter().enumerate() {
            for t in 0..FUTURE_TRACE_LEN as u64 {
                if let Some(v) = reference_at(case, &trace, t).unwrap() {
                    let v = if flip == Some((name.as_str(), idx, t)) {
                        !v
                    } else {
                        v
                    };
                    text.push_str(&format!("{idx}:{t},{}\n", if v { 'T' } else { 'F' }));
                }
            }
        }
        out.insert(name, text);
    }
    out
}

fn replay(stdout: &BTreeMap<String, String>) -> Report {
    let out = tempfile::tempdir().unwrap();
    // The live run's report, as a live run would have written it.
    let mut rec = run_future(Path::new(REPO), &Source::None("x".into()), out.path()).unwrap();
    rec.mode = "ran".to_owned();
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
    run_future(Path::new(REPO), &Source::Recorded, out.path()).unwrap()
}

#[test]
fn future_replay_without_the_live_report_is_refused() {
    let out = tempfile::tempdir().unwrap();
    assert!(run_future(Path::new(REPO), &Source::Recorded, out.path()).is_err());
}

#[test]
fn replay_of_a_monitor_that_matches_the_reference_passes() {
    let r = replay(&simulated(None));
    assert_eq!(r.mode, "replayed");
    assert_eq!(r.outcome, "pass", "{:?}", r.summary);
    assert!(r.groups.iter().all(|g| g.stdout_sha256.is_some()));
}

#[test]
fn one_flipped_verdict_is_an_unexplained_disagreement() {
    let traces = future_traces();
    let name = trace_name(&traces[5]);
    let corpus = future_corpus();
    let (idx, t) = (0..corpus.len())
        .flat_map(|i| (0..FUTURE_TRACE_LEN as u64).map(move |t| (i, t)))
        .find(|(i, t)| reference_at(&corpus[*i], &traces[5], *t).unwrap().is_some())
        .unwrap();
    let r = replay(&simulated(Some((&name, idx, t))));
    assert_eq!(r.outcome, "fail");
    assert_eq!(r.summary.get("unexplained"), Some(&1));
}

#[test]
fn a_silent_target_where_the_reference_is_conclusive_is_missing() {
    let mut s = simulated(None);
    let k = s.keys().find(|k| !s[*k].is_empty()).unwrap().clone();
    // Drop the final tuple: a later tuple would cover an earlier position
    // (R2U2 aggregates ranges), so only the last one leaves a position silent.
    let lines: Vec<&str> = s[&k].lines().collect();
    let rest: String = lines[..lines.len() - 1]
        .iter()
        .map(|l| format!("{l}\n"))
        .collect();
    s.insert(k, rest);
    let r = replay(&s);
    assert_eq!(r.outcome, "fail");
    assert!(r.summary.contains_key("missing_target"));
}

#[test]
fn target_resolving_what_the_reference_leaves_pending_is_a_documented_difference_not_a_pass_of_the_row(
) {
    let corpus = future_corpus();
    let traces = future_traces();
    let (ti, ci, t) = traces
        .iter()
        .enumerate()
        .flat_map(|(ti, tr)| {
            let corpus = &corpus;
            (0..corpus.len()).flat_map(move |ci| {
                (0..FUTURE_TRACE_LEN as u64).filter_map(move |t| {
                    reference_at(&corpus[ci], tr, t)
                        .unwrap()
                        .is_none()
                        .then_some((ti, ci, t))
                })
            })
        })
        .next()
        .unwrap();
    let mut s = simulated(None);
    let name = trace_name(&traces[ti]);
    s.get_mut(&name).unwrap().push_str(&format!("{ci}:{t},T\n"));
    let r = replay(&s);
    assert_eq!(
        r.outcome, "pass",
        "documented difference does not fail the run"
    );
    assert_eq!(r.summary.get("pending_resolved_by_target"), Some(&1));
}
