//! TL-225: the hand-authored boundary corpus (`corpus/boundary-v1`).
//!
//! Every operator x interval-shape cell carries at least one positive, one
//! negative and one boundary case, plus the named edge conditions (point
//! interval, lower bound zero, upper bound at the trace end, empty trace,
//! horizon equal to the prefix length, integer extremes, nested windows that
//! only just fit, windows reaching before the origin). Expected verdicts are
//! literals derived by hand from the profile definitions; they are never
//! produced by the code under test. Each closed-trace and origin-complete case
//! is replayed against both the independent `tl-oracle` and production; the
//! online-prefix profile is not modelled by the oracle, so its progress
//! verdicts are checked against production only.

use std::{collections::BTreeSet, fs, path::PathBuf};

use serde_json::Value;
use tl_mltl::{evaluate_prefix_at, EvaluationLimits, TruthValue};
use tl_oracle::population::{CaseResult, PopulationError, PopulationLedger};
use tl_oracle::Formula as O;
use tl_syntax::{Formula, Node, PropositionId, SemanticProfile};

#[path = "support/finite_harness.rs"]
mod harness;
use harness::{
    drop_negation, formula_from_json, lower, lower_start_down, present, production, reference,
    shrink_end, swap_dual, word_from_json, Profile, Word,
};

const CLOSED: &str = "mltl.closed-trace/v1";
const PAST: &str = "mltl.origin-complete-history/v1";
const PREFIX: &str = "mltl.online-prefix/v1";

struct Case {
    id: String,
    profile: String,
    operator: String,
    shape: String,
    kind: String,
    edges: Vec<String>,
    formula: O,
    trace: Word,
    at: usize,
    closed: bool,
    expected: Value,
    checked_by: Vec<String>,
}

fn load() -> Vec<Case> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("corpus/boundary-v1/cases.json");
    let doc: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    assert_eq!(doc["schema"], "tl-mltl.boundary-corpus/v1");
    doc["cases"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| {
            let s = |k: &str| c[k].as_str().unwrap().to_owned();
            Case {
                id: s("id"),
                profile: s("profile"),
                operator: s("operator"),
                shape: s("shape"),
                kind: s("kind"),
                edges: c["edges"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|e| e.as_str().unwrap().to_owned())
                    .collect(),
                formula: formula_from_json(&c["formula"]),
                trace: word_from_json(&c["trace"]),
                at: c["at"].as_u64().unwrap() as usize,
                closed: c.get("closed").and_then(Value::as_bool).unwrap_or(true),
                expected: c["expected"].clone(),
                checked_by: c.get("checkedBy").and_then(Value::as_array).map_or_else(
                    || vec!["oracle".to_owned(), "production".to_owned()],
                    |a| a.iter().map(|v| v.as_str().unwrap().to_owned()).collect(),
                ),
            }
        })
        .collect()
}

fn oracle_profile(case: &Case) -> Option<Profile> {
    match case.profile.as_str() {
        CLOSED => Some(Profile::ClosedFuture),
        PAST => Some(Profile::OriginPast),
        _ => None,
    }
}

fn stored(case: &Case) -> CaseResult<bool> {
    match &case.expected {
        Value::Bool(b) => CaseResult::Admitted(*b),
        Value::String(s) if s == "refused" => CaseResult::Refused,
        other => panic!("{}: unexpected stored verdict {other}", case.id),
    }
}

/// Production side for the online-prefix profile: the verdict token or `refused`.
fn prefix_verdict(f: &O, word: &Word, closed: bool, at: usize) -> String {
    let mut nodes: Vec<Node> = Vec::new();
    let root = lower(f, &mut nodes);
    let formula = Formula::new(SemanticProfile::OnlinePrefixV1, root, &nodes).expect("formula");
    let trace: Vec<Vec<PropositionId>> = word.iter().map(present).collect();
    match evaluate_prefix_at(
        formula,
        "f",
        &trace,
        "t",
        closed,
        at as u64,
        EvaluationLimits::default(),
    ) {
        Ok(report) => match report.verdict {
            TruthValue::True => "true".to_owned(),
            TruthValue::False => "false".to_owned(),
            TruthValue::Pending => "pending".to_owned(),
        },
        Err(_) => "refused".to_owned(),
    }
}

fn expected_token(case: &Case) -> String {
    match &case.expected {
        Value::Bool(b) => b.to_string(),
        Value::String(s) => s.clone(),
        other => panic!("{}: {other}", case.id),
    }
}

/// Replays the oracle-modelled cases through one side, recording each once.
fn replay(
    cases: &[Case],
    side_name: &str,
    side: impl Fn(Profile, &O, &Word, usize) -> CaseResult<bool>,
    fault: &dyn Fn(&O) -> O,
) -> Result<(u64, u64), PopulationError> {
    let modelled: Vec<&Case> = cases
        .iter()
        .filter(|c| oracle_profile(c).is_some() && c.checked_by.iter().any(|s| s == side_name))
        .collect();
    let mut ledger = PopulationLedger::new(modelled.len() as u64)?;
    for (id, case) in modelled.iter().enumerate() {
        let profile = oracle_profile(case).unwrap();
        let observed = side(profile, &fault(&case.formula), &case.trace, case.at);
        if let Err(e) = ledger.record(id as u64, stored(case), observed) {
            eprintln!("{side_name} disagrees on {}: {}", case.id, case.expected);
            return Err(e);
        }
    }
    let summary = ledger.finish()?;
    assert_eq!(summary.declared, modelled.len() as u64);
    Ok((summary.admitted, summary.refused))
}

fn replay_prefix(cases: &[Case], fault: &dyn Fn(&O) -> O) -> Vec<String> {
    cases
        .iter()
        .filter(|c| c.profile == PREFIX)
        .filter(|c| {
            prefix_verdict(&fault(&c.formula), &c.trace, c.closed, c.at) != expected_token(c)
        })
        .map(|c| c.id.clone())
        .collect()
}

#[test]
fn corpus_is_non_empty_with_unique_ids() {
    let cases = load();
    let ids: BTreeSet<&str> = cases.iter().map(|c| c.id.as_str()).collect();
    assert_eq!(ids.len(), cases.len(), "duplicate case id");
    assert!(cases.len() >= 100);
}

/// Every stored verdict is confirmed by the independent oracle.
// Trace: TC-107
#[test]
fn stored_verdicts_agree_with_the_independent_oracle() {
    let (admitted, refused) = replay(&load(), "oracle", reference, &Clone::clone)
        .unwrap_or_else(|e| panic!("a hand-authored verdict disagrees with the oracle: {e:?}"));
    assert!(admitted > 0 && refused > 0);
}

/// Every stored verdict is confirmed by the production evaluators.
// Trace: TC-107
#[test]
fn stored_verdicts_agree_with_production() {
    let (admitted, refused) = replay(&load(), "production", production, &Clone::clone)
        .unwrap_or_else(|e| panic!("production disagrees with a hand-authored verdict: {e:?}"));
    assert!(admitted > 0 && refused > 0);
}

/// Pending, early-witness, early-counterexample and closed-prefix transitions
/// at each future operator and interval-boundary class.
// Trace: TC-094
#[test]
fn online_prefix_progress_matches_the_stored_verdicts() {
    let cases = load();
    let failures = replay_prefix(&cases, &Clone::clone);
    assert!(failures.is_empty(), "{failures:?}");
    // Every future operator reaches pending, true and false, and both an open
    // and a closed prefix appear.
    let prefix: Vec<&Case> = cases.iter().filter(|c| c.profile == PREFIX).collect();
    for op in ["future", "globally", "until", "release"] {
        let verdicts: BTreeSet<String> = prefix
            .iter()
            .filter(|c| c.operator == op)
            .map(|c| expected_token(c))
            .collect();
        for want in ["pending", "true", "false"] {
            assert!(verdicts.contains(want), "{op} has no prefix `{want}` case");
        }
    }
    assert!(prefix.iter().any(|c| c.closed) && prefix.iter().any(|c| !c.closed));
    // Every future operator meets every interval-boundary class in progress terms.
    for op in ["future", "globally", "until", "release"] {
        for shape in ["point-zero", "from-zero", "point", "interior"] {
            assert!(
                prefix.iter().any(|c| c.operator == op && c.shape == shape),
                "no prefix case for {op} {shape}"
            );
        }
    }
    assert!(prefix.iter().any(|c| c.shape == "max"));
}

const FUTURE: [&str; 4] = ["future", "globally", "until", "release"];
const PASTS: [&str; 4] = ["once", "historically", "since", "triggered"];
const SHAPES: [&str; 4] = ["point-zero", "from-zero", "point", "interior"];

/// No operator x interval-shape cell is left without its three cases, and
/// every named edge condition and extreme appears.
#[test]
fn every_cell_has_positive_negative_and_boundary_cases() {
    let cases = load();
    let has = |profile: &str, op: &str, shape: &str, kind: &str| {
        cases
            .iter()
            .any(|c| c.profile == profile && c.operator == op && c.shape == shape && c.kind == kind)
    };
    for (profile, ops) in [(CLOSED, FUTURE), (PAST, PASTS)] {
        for op in ops {
            for shape in SHAPES {
                for kind in ["positive", "negative", "boundary"] {
                    assert!(
                        has(profile, op, shape, kind),
                        "{profile} {op} {shape} {kind}"
                    );
                }
            }
            // An interval at u32::MAX has only a refusal verdict.
            assert!(has(profile, op, "max", "boundary"), "{profile} {op} max");
        }
    }
    // Boolean connectives have no interval: each needs a positive and a negative.
    for profile in [CLOSED, PAST] {
        for op in ["not", "and", "or", "implies", "equivalent"] {
            for kind in ["positive", "negative"] {
                assert!(
                    has(profile, op, "not-applicable", kind),
                    "{profile} {op} {kind}"
                );
            }
        }
    }
    for kind in ["positive", "negative", "boundary"] {
        assert!(
            has(PAST, "yesterday", "not-applicable", kind),
            "yesterday {kind}"
        );
    }
    assert!(has(CLOSED, "true", "not-applicable", "positive"));
    assert!(has(CLOSED, "false", "not-applicable", "negative"));
    let mut tagged: BTreeSet<&str> = BTreeSet::new();
    for c in &cases {
        tagged.extend(c.edges.iter().map(String::as_str));
    }
    for edge in [
        "a=b",
        "a=0",
        "b=trace-end",
        "empty-trace",
        "horizon=prefix",
        "integer-extreme",
        "nested-just-fits",
        "window-before-origin",
    ] {
        assert!(tagged.contains(edge), "no case exercises `{edge}`");
    }
    // The extremes really are the u32 extremes.
    assert!(cases
        .iter()
        .any(|c| format!("{:?}", c.formula).contains("4294967295")));
}

/// A corpus that cannot fail proves nothing: each seeded fault must turn the
/// replay red, both through the oracle-modelled profiles and the prefix profile.
// Trace: TC-107
#[test]
fn seeded_faults_turn_the_corpus_red() {
    type Fault = fn(&O) -> O;
    let faults: [(&str, Fault); 4] = [
        ("interval end off by one", shrink_end),
        ("lower bound off by one", lower_start_down),
        ("dual operator swapped", swap_dual),
        ("negation dropped", drop_negation),
    ];
    let cases = load();
    for (name, fault) in faults {
        let closed_past = replay(&cases, "production", production, &fault);
        let prefix = replay_prefix(&cases, &fault);
        assert!(
            matches!(closed_past, Err(PopulationError::Mismatch { .. })) || !prefix.is_empty(),
            "fault `{name}` was not detected by the corpus"
        );
    }
    // The two interval faults must also be caught by the oracle-modelled
    // profiles alone, where the stored verdicts are independently confirmed.
    for fault in [
        shrink_end as Fault,
        lower_start_down as Fault,
        swap_dual as Fault,
    ] {
        assert!(matches!(
            replay(&cases, "production", production, &fault),
            Err(PopulationError::Mismatch { .. })
        ));
    }
}
