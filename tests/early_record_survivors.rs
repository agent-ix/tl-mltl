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
