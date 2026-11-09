//! Kani proof records (FR-023-AC-2, FR-023-AC-3): one typed outcome per
//! harness, the tool versions and the bounds the harness states.
//!
//! A gate run produces a [`KaniRecord`]; the campaign runner reads it back
//! strictly and binds each proof's outcome into the campaign report (see
//! [`crate::campaign`]). Nothing here runs Kani. A record states what one
//! bounded run observed; it is not a correctness, release, qualification or
//! certification verdict, and `verified` means only that the harness's stated
//! bounds were checked.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Schema of a Kani proof record file.
pub const KANI_RECORD_SCHEMA: &str = "tl-mltl.kani-proof-record/v1";

/// How one proof run ended. Every state is distinct; none reads as a pass
/// except [`ProofOutcome::Verified`].
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ProofOutcome {
    /// Every assertion held, unwinding was complete and every cover was
    /// satisfied.
    Verified,
    /// An assertion or other property check failed with a counterexample.
    AssertionFailed,
    /// A loop or recursion needs a larger unwind bound; the verdict is
    /// incomplete.
    UnwindingFailed,
    /// The run exceeded the time budget and was stopped.
    Timeout,
    /// A reachable construct is not supported by Kani.
    UnsupportedConstruct,
    /// The run exhausted memory.
    OutOfMemory,
    /// Verification succeeded but a cover check was unsatisfied or absent, so
    /// the proof may be vacuous.
    VacuousCover,
    /// Kani failed without a verification result.
    ToolError,
}

/// One harness run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProofRecord {
    /// Exact qualified harness name.
    pub harness: String,
    /// End state.
    pub outcome: ProofOutcome,
    /// `cargo kani --version` output.
    pub kani_version: String,
    /// CBMC version Kani drove.
    pub cbmc_version: String,
    /// Cover properties satisfied.
    pub covers_satisfied: u32,
    /// Cover properties in the harness.
    pub covers_total: u32,
    /// Wall-clock seconds of the run; volatile, never compared on replay.
    pub elapsed_seconds: u64,
    /// The harness doc comment: the bounds it covers and what it does not
    /// claim.
    pub bounds: String,
    /// Command that reproduces the run, including any counterexample.
    pub reproduce: Vec<String>,
}

/// The record file a gate run writes for one harness.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct KaniRecord {
    /// Must equal [`KANI_RECORD_SCHEMA`].
    pub schema_version: String,
    /// Proof runs in execution order.
    pub proofs: Vec<ProofRecord>,
}

/// Why a record was refused.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum KaniRecordErrorCode {
    /// Not valid JSON for the record shape, or an unknown field.
    Malformed,
    /// Wrong `schemaVersion`.
    Schema,
    /// Outcome, cover counts and versions disagree.
    Inconsistent,
}

/// A refused record.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KaniRecordError {
    /// Stable code.
    pub code: KaniRecordErrorCode,
    /// Human-readable detail; never branch on it.
    pub detail: String,
}

impl fmt::Display for KaniRecordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "kani record {:?}: {}", self.code, self.detail)
    }
}

impl std::error::Error for KaniRecordError {}

fn refuse(code: KaniRecordErrorCode, detail: String) -> KaniRecordError {
    KaniRecordError { code, detail }
}

fn check_proof(proof: &ProofRecord) -> Result<(), KaniRecordError> {
    let id = &proof.harness;
    let bad = |detail: &str| {
        Err(refuse(
            KaniRecordErrorCode::Inconsistent,
            format!("{id}: {detail}"),
        ))
    };
    if proof.harness.is_empty() || proof.kani_version.is_empty() || proof.cbmc_version.is_empty() {
        return bad("empty harness or tool version");
    }
    if proof.bounds.trim().is_empty() {
        return bad("empty bounds: the harness doc comment states none");
    }
    if proof.covers_satisfied > proof.covers_total {
        return bad("more covers satisfied than exist");
    }
    let all_covered = proof.covers_total > 0 && proof.covers_satisfied == proof.covers_total;
    match proof.outcome {
        ProofOutcome::Verified if !all_covered => bad("verified without every cover satisfied"),
        ProofOutcome::VacuousCover if all_covered => bad("vacuous with every cover satisfied"),
        _ => Ok(()),
    }
}

/// Read a record strictly: unknown fields, a wrong schema and inconsistent
/// proofs are refused, so a read followed by a write is the identity.
pub fn parse_record(text: &str) -> Result<KaniRecord, KaniRecordError> {
    let record: KaniRecord = serde_json::from_str(text)
        .map_err(|e| refuse(KaniRecordErrorCode::Malformed, e.to_string()))?;
    if record.schema_version != KANI_RECORD_SCHEMA {
        return Err(refuse(
            KaniRecordErrorCode::Schema,
            format!("schema {:?}", record.schema_version),
        ));
    }
    record.proofs.iter().try_for_each(check_proof)?;
    Ok(record)
}

/// `(satisfied, total)` from Kani's `** N of M cover properties satisfied`
/// line, or `None` when the log has no such line.
pub fn parse_covers(log: &str) -> Option<(u32, u32)> {
    log.lines().find_map(|line| {
        let words: Vec<&str> = line.split_whitespace().collect();
        let at = words
            .windows(5)
            .position(|w| w[0] == "**" && w[2] == "of" && w[4] == "cover")?;
        Some((words[at + 1].parse().ok()?, words[at + 3].parse().ok()?))
    })
}

/// True when a `Failed Checks:` line names something other than an unwinding
/// assertion.
fn has_property_failure(log: &str) -> bool {
    log.lines().any(|line| {
        line.trim_start().starts_with("Failed Checks:") && !line.contains("unwinding assertion")
    })
}

/// Classify a finished `cargo kani` run from its exit code and log.
/// `timed_out` is true when the gate stopped the run at its time budget.
///
/// The classes are checked in a fixed order: a timeout (a timed-out run is
/// never `verified`, whatever its exit code or log), then a successful
/// verification (a vacuous cover is its own state because Kani exits 0 for
/// it), then a failed one, then memory exhaustion; anything else is a tool
/// error. For a failed verification a genuine property failure wins over
/// incomplete unwinding: a log with both a failed assertion and an unwinding
/// assertion is `assertion_failed`, because the counterexample is real whatever
/// the unwind bound. Then an unsupported construct, then unwinding failure.
pub fn classify_proof(exit_code: Option<i32>, log: &str, timed_out: bool) -> ProofOutcome {
    if timed_out {
        return ProofOutcome::Timeout;
    }
    if log.contains("VERIFICATION:- SUCCESSFUL") {
        return match parse_covers(log) {
            Some((satisfied, total)) if total > 0 && satisfied == total => ProofOutcome::Verified,
            _ => ProofOutcome::VacuousCover,
        };
    }
    if log.contains("VERIFICATION:- FAILED") {
        return if has_property_failure(log) {
            ProofOutcome::AssertionFailed
        } else if log.contains("is not currently supported by Kani") {
            ProofOutcome::UnsupportedConstruct
        } else if log.contains("unwinding assertion") {
            ProofOutcome::UnwindingFailed
        } else {
            ProofOutcome::AssertionFailed
        };
    }
    let lower = log.to_ascii_lowercase();
    if exit_code == Some(137)
        || lower.contains("out of memory")
        || lower.contains("bad_alloc")
        || lower.contains("cannot allocate memory")
    {
        ProofOutcome::OutOfMemory
    } else {
        ProofOutcome::ToolError
    }
}

/// Everything the gate observed about one harness run.
#[derive(Clone, Debug)]
pub struct ProofRun<'a> {
    /// Exact qualified harness name.
    pub harness: &'a str,
    /// `cargo kani` exit code, `None` when killed.
    pub exit_code: Option<i32>,
    /// Combined stdout and stderr.
    pub log: &'a str,
    /// Whether the gate stopped the run at its time budget.
    pub timed_out: bool,
    /// `cargo kani --version` output.
    pub kani_version: &'a str,
    /// CBMC version.
    pub cbmc_version: &'a str,
    /// Wall-clock seconds.
    pub elapsed_seconds: u64,
    /// The harness doc comment.
    pub bounds: &'a str,
}

/// Build the record for one run.
pub fn build_record(run: &ProofRun<'_>) -> ProofRecord {
    let (covers_satisfied, covers_total) = parse_covers(run.log).unwrap_or((0, 0));
    ProofRecord {
        harness: run.harness.to_owned(),
        outcome: classify_proof(run.exit_code, run.log, run.timed_out),
        kani_version: run.kani_version.to_owned(),
        cbmc_version: run.cbmc_version.to_owned(),
        covers_satisfied,
        covers_total,
        elapsed_seconds: run.elapsed_seconds,
        bounds: run.bounds.to_owned(),
        reproduce: [
            "cargo",
            "kani",
            "--lib",
            "-Z",
            "stubbing",
            "--harness",
            run.harness,
            "--exact",
        ]
        .map(str::to_owned)
        .to_vec(),
    }
}

/// The doc comment of `fn <name>(` in `source`, with the `///` markers removed.
pub fn harness_bounds(source: &str, name: &str) -> Option<String> {
    let lines: Vec<&str> = source.lines().collect();
    let at = lines
        .iter()
        .position(|line| line.trim_start().starts_with(&format!("fn {name}(")))?;
    let mut doc: Vec<&str> = lines[..at]
        .iter()
        .rev()
        .map(|line| line.trim())
        .take_while(|line| line.starts_with("///") || line.starts_with("#["))
        .filter(|line| line.starts_with("///"))
        .map(|line| line.trim_start_matches("///").trim())
        .collect();
    doc.reverse();
    (!doc.is_empty()).then(|| doc.join(" "))
}

#[cfg(test)]
mod tests {
    use super::*;

    const OK: &str = "** 3 of 3 cover properties satisfied\n\nVERIFICATION:- SUCCESSFUL\n";

    fn proof(outcome: ProofOutcome, satisfied: u32, total: u32) -> ProofRecord {
        ProofRecord {
            harness: "h".into(),
            outcome,
            kani_version: "Kani 0.68.0".into(),
            cbmc_version: "6.11.0".into(),
            covers_satisfied: satisfied,
            covers_total: total,
            elapsed_seconds: 1,
            bounds: "b".into(),
            reproduce: vec![],
        }
    }

    fn text(p: &ProofRecord) -> String {
        serde_json::to_string(&KaniRecord {
            schema_version: KANI_RECORD_SCHEMA.into(),
            proofs: vec![p.clone()],
        })
        .unwrap()
    }

    // Trace: TC-121, FR-023-AC-3
    #[test]
    fn built_records_keep_each_outcome_and_reject_an_empty_bounds_text() {
        let logs: [(&str, Option<i32>, bool, ProofOutcome); 6] = [
            (OK, Some(0), false, ProofOutcome::Verified),
            (
                "** 1 of 2 cover properties satisfied\nVERIFICATION:- SUCCESSFUL",
                Some(0),
                false,
                ProofOutcome::VacuousCover,
            ),
            (
                "Failed Checks: assertion failed\nVERIFICATION:- FAILED",
                Some(1),
                false,
                ProofOutcome::AssertionFailed,
            ),
            (
                "Failed Checks: unwinding assertion loop 0\nVERIFICATION:- FAILED",
                Some(1),
                false,
                ProofOutcome::UnwindingFailed,
            ),
            ("", Some(137), false, ProofOutcome::OutOfMemory),
            (OK, Some(0), true, ProofOutcome::Timeout),
        ];
        for (log, code, timed_out, want) in logs {
            let run = ProofRun {
                harness: "m::h",
                exit_code: code,
                log,
                timed_out,
                kani_version: "Kani 0.68.0",
                cbmc_version: "6.11.0",
                elapsed_seconds: 1,
                bounds: "Proves x.",
            };
            assert_eq!(build_record(&run).outcome, want, "{log:?}");
        }
        let mut empty = proof(ProofOutcome::Verified, 3, 3);
        empty.bounds = "  ".into();
        assert_eq!(
            parse_record(&text(&empty)).unwrap_err().code,
            KaniRecordErrorCode::Inconsistent
        );
    }

    // Partly backs FR-023-AC-2: the classifier names each state; the gate
    // script does not enforce a time budget, so a timeout is only recorded
    // when a caller sets TL_MLTL_KANI_TIMED_OUT.
    #[test]
    fn every_failure_class_has_its_own_outcome() {
        let classify = |code, log, timed| classify_proof(code, log, timed);
        assert_eq!(classify(Some(0), OK, false), ProofOutcome::Verified);
        assert_eq!(classify(None, OK, true), ProofOutcome::Timeout);
        // A timed-out run is never verified, even with an exit-0 successful log.
        assert_eq!(classify(Some(0), OK, true), ProofOutcome::Timeout);
        assert_eq!(
            classify(
                Some(0),
                "** 1 of 2 cover properties satisfied\nVERIFICATION:- SUCCESSFUL",
                false
            ),
            ProofOutcome::VacuousCover
        );
        assert_eq!(
            classify(Some(0), "VERIFICATION:- SUCCESSFUL", false),
            ProofOutcome::VacuousCover
        );
        assert_eq!(
            classify(
                Some(1),
                "Failed Checks: unwinding assertion loop 0\nVERIFICATION:- FAILED",
                false
            ),
            ProofOutcome::UnwindingFailed
        );
        assert_eq!(
            classify(
                Some(1),
                "x is not currently supported by Kani\nVERIFICATION:- FAILED",
                false
            ),
            ProofOutcome::UnsupportedConstruct
        );
        assert_eq!(
            classify(
                Some(1),
                "Failed Checks: assertion failed\nVERIFICATION:- FAILED",
                false
            ),
            ProofOutcome::AssertionFailed
        );
        // A genuine assertion failure wins over an unwinding failure in one log.
        assert_eq!(
            classify(
                Some(1),
                "Failed Checks: unwinding assertion loop 0\nFailed Checks: assertion failed: x\nVERIFICATION:- FAILED",
                false
            ),
            ProofOutcome::AssertionFailed
        );
        assert_eq!(classify(Some(137), "", false), ProofOutcome::OutOfMemory);
        assert_eq!(
            classify(Some(101), "error: compile", false),
            ProofOutcome::ToolError
        );
    }

    #[test]
    fn cover_counts_are_read_from_the_result_line() {
        assert_eq!(parse_covers(OK), Some((3, 3)));
        assert_eq!(
            parse_covers(" ** 4 of 5 cover properties satisfied (1 unreachable)"),
            Some((4, 5))
        );
        assert_eq!(parse_covers("no covers"), None);
    }

    // Trace: TC-121, FR-023-AC-3
    #[test]
    fn records_round_trip_and_refuse_inconsistency() {
        let good = proof(ProofOutcome::Verified, 3, 3);
        let parsed = parse_record(&text(&good)).unwrap();
        assert_eq!(serde_json::to_string(&parsed).unwrap(), text(&good));
        let refuse_code = |p: ProofRecord| parse_record(&text(&p)).unwrap_err().code;
        assert_eq!(
            refuse_code(proof(ProofOutcome::Verified, 2, 3)),
            KaniRecordErrorCode::Inconsistent
        );
        assert_eq!(
            refuse_code(proof(ProofOutcome::Verified, 0, 0)),
            KaniRecordErrorCode::Inconsistent
        );
        assert_eq!(
            refuse_code(proof(ProofOutcome::VacuousCover, 3, 3)),
            KaniRecordErrorCode::Inconsistent
        );
        assert_eq!(
            refuse_code(proof(ProofOutcome::Verified, 4, 3)),
            KaniRecordErrorCode::Inconsistent
        );
        let mut no_version = proof(ProofOutcome::Verified, 3, 3);
        no_version.kani_version.clear();
        assert_eq!(refuse_code(no_version), KaniRecordErrorCode::Inconsistent);
        assert_eq!(
            parse_record("{}").unwrap_err().code,
            KaniRecordErrorCode::Malformed
        );
        let wrong = text(&good).replace("kani-proof-record/v1", "kani-proof-record/v0");
        assert_eq!(
            parse_record(&wrong).unwrap_err().code,
            KaniRecordErrorCode::Schema
        );
        let extra = text(&good).replace("\"harness\"", "\"extra\":1,\"harness\"");
        assert_eq!(
            parse_record(&extra).unwrap_err().code,
            KaniRecordErrorCode::Malformed
        );
    }

    // Trace: TC-121, FR-023-AC-3
    #[test]
    fn a_built_record_carries_versions_bounds_and_a_reproducing_command() {
        let run = ProofRun {
            harness: "m::h",
            exit_code: Some(0),
            log: OK,
            timed_out: false,
            kani_version: "Kani 0.68.0",
            cbmc_version: "6.11.0",
            elapsed_seconds: 5,
            bounds: "Proves x.",
        };
        let record = build_record(&run);
        assert_eq!(record.outcome, ProofOutcome::Verified);
        assert_eq!(record.kani_version, "Kani 0.68.0");
        assert_eq!(record.cbmc_version, "6.11.0");
        assert_eq!(record.bounds, "Proves x.");
        assert_eq!(record.harness, "m::h");
        assert_eq!(record.elapsed_seconds, 5);
        assert_eq!((record.covers_satisfied, record.covers_total), (3, 3));
        assert!(record
            .reproduce
            .ends_with(&["--harness".into(), "m::h".into(), "--exact".into()]));
        assert!(parse_record(&text(&record)).is_ok());
    }

    #[test]
    fn harness_docs_are_extracted_without_markers() {
        let source = "/// Proves a.\n///\n/// Does not claim b.\n#[kani::proof]\nfn h() {}\n\nfn other() {}\n";
        assert_eq!(
            harness_bounds(source, "h").as_deref(),
            Some("Proves a.  Does not claim b.")
        );
        assert_eq!(harness_bounds(source, "other"), None);
        assert_eq!(harness_bounds(source, "missing"), None);
    }
}
