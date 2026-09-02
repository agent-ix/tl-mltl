//! Replay the shared temporal corpus through the reference semantics (FR-006-AC-2).
//!
//! This is a producer. It runs the real crate over the real corpus bytes and
//! writes one declared structured row per obligation to stdout. It computes no
//! aggregate verdict, retains nothing, and knows nothing about Quoin: the
//! assurance chain reads these rows, and a row this file does not emit is a
//! result nothing downstream can invent.
//!
//! Every expectation comes from `corpus/tl-syntax-v1/manifest.json`, which is
//! the upstream corpus declaration rather than anything this repository derived.
//! A fixture whose expectation the manifest does not state is refused rather
//! than skipped, because a skipped obligation and a discharged one must not
//! print the same thing.
//!
//! Row vocabulary, all of which the chain enumerates:
//!
//! - `pass`      the obligation was discharged
//! - `fail`      the crate disagreed with the corpus declaration
//! - `malformed` a fixture the manifest declares invalid was rejected as invalid
//!
//! `malformed` is not a defect and it is not a pass of the input. Three of the
//! eight shared fixtures are malformed by design, so reporting them as `fail`
//! would report a permanently failing proof for a permanently correct
//! evaluator. The word survives into the row, into the bytes Quoin retains, and
//! into a chain scenario whose count oracle is the corpus manifest.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use serde::Deserialize;
use serde_json::{json, Value};
use tl_mltl::{analyze_horizon, evaluate_closed, EvaluationLimits, TruthValue};
use tl_syntax::{FormulaDocument, PropositionId};

const PROTOCOL: &str = "tl-mltl.reference-conformance/v1";

#[derive(Deserialize)]
struct Manifest {
    corpus_revision: String,
    fixtures: Vec<Fixture>,
}

#[derive(Deserialize)]
struct Fixture {
    id: String,
    class: String,
    formula: String,
    expected_validation: String,
    #[serde(default)]
    trace: Vec<Vec<PropositionId>>,
    expected_horizon: Option<u64>,
    expected_closed_trace: Option<bool>,
    expected_error: Option<String>,
}

/// The rejection stage, and the discriminating marker, a declared corpus error
/// must be reached by.
///
/// Two facts per declared error, and both have to hold.
///
/// The stage is structural. `serde` and `Formula::validate` are two different
/// refusals, and a producer that could not tell them apart would report a
/// document rejected for the wrong reason as correctly rejected. All three of
/// this corpus's malformed fixtures are refused at the wire boundary, because
/// `tl_syntax::FormulaDocument`'s own deserializer enforces interval order,
/// operand order and profile identity. That is stated here as an observed fact
/// rather than assumed: if a future upstream moved one of them to
/// `validate`, the row would fail loudly and this table would have to be
/// updated deliberately.
///
/// The marker exists because a stage alone cannot tell three deserializer
/// refusals apart, and "rejected" without "for the declared reason" is a
/// materially weaker claim. `FormulaDocument`'s deserializer surfaces its cause
/// only in the message, so the message is the only discriminator available.
/// This is a stated limitation, not a preference: it is named here rather than
/// resolved by re-implementing upstream's validation locally, which is the kind
/// of second implementation this migration exists to remove.
fn declared_rejection(expected_error: &str) -> Option<(&'static str, &'static str)> {
    match expected_error {
        "interval_inverted" => Some(("deserialize", "interval start")),
        "operand_not_preceding" => Some(("deserialize", "must identify a preceding node")),
        "unsupported_semantic_profile" => Some(("deserialize", "unknown variant")),
        _ => None,
    }
}

/// Read a declared corpus file, or stop.
///
/// Deliberately not `unwrap_or_default`. An absent fixture would yield empty
/// bytes, empty bytes fail to deserialize, and a failure to deserialize is
/// exactly what a malformed fixture is supposed to produce — so a deleted
/// fixture would have been reported as a correctly rejected one.
fn read_declared(corpus: &Path, relative: &str) -> Result<Vec<u8>, String> {
    let path = corpus.join(relative);
    fs::read(&path).map_err(|error| {
        format!(
            "the corpus manifest declares {}, which could not be read: {error}. \
             A declared fixture that is not on disk is an error, not a rejection.",
            path.display()
        )
    })
}

struct Row {
    symbol: String,
    family: &'static str,
    outcome: &'static str,
    trace_ids: Vec<&'static str>,
    detail: Value,
}

impl Row {
    fn emit(&self) -> String {
        json!({
            "protocol": PROTOCOL,
            "symbol": self.symbol,
            "family": self.family,
            "outcome": self.outcome,
            "traceIds": self.trace_ids,
            "detail": self.detail,
        })
        .to_string()
    }
}

fn truth_name(value: TruthValue) -> &'static str {
    match value {
        TruthValue::True => "true",
        TruthValue::False => "false",
        TruthValue::Pending => "pending",
    }
}

fn manifest_path(arguments: &[String]) -> Result<PathBuf, String> {
    let mut iterator = arguments.iter();
    while let Some(argument) = iterator.next() {
        if argument == "--manifest" {
            let value = iterator
                .next()
                .ok_or_else(|| "--manifest requires a path".to_owned())?;
            return Ok(PathBuf::from(value));
        }
    }
    Err("usage: reference_conformance --manifest PATH".to_owned())
}

/// Deserialize and validate a fixture the manifest declares valid.
///
/// A declared-valid fixture that does not load is a failure of the fixture or
/// of the crate, and either way it is not something to work around, so it is
/// reported as an error rather than folded into a row outcome.
fn load_valid(fixture: &Fixture, corpus: &Path) -> Result<FormulaDocument, String> {
    let bytes = read_declared(corpus, &fixture.formula)?;
    serde_json::from_slice(&bytes).map_err(|error| {
        format!(
            "fixture {} is declared valid but did not deserialize: {error}",
            fixture.id
        )
    })
}

fn horizon_row(fixture: &Fixture, corpus: &Path) -> Result<Row, String> {
    let symbol = format!("{}/horizon", fixture.id);
    let expected = fixture.expected_horizon.ok_or_else(|| {
        format!(
            "fixture {} is declared valid but the manifest states no expected horizon; \
             an obligation with no declared expectation is refused rather than skipped",
            fixture.id
        )
    })?;
    let document = load_valid(fixture, corpus)?;
    let formula = document.validate().map_err(|error| {
        format!(
            "fixture {} is declared valid but did not validate: {error}",
            fixture.id
        )
    })?;
    Ok(match analyze_horizon(formula, fixture.id.clone()) {
        Ok(report) => Row {
            symbol,
            family: "horizon",
            outcome: if report.lookahead == expected {
                "pass"
            } else {
                "fail"
            },
            trace_ids: vec!["FR-002-AC-1", "FR-002-AC-3", "TC-005"],
            detail: json!({
                "declaredByCorpusManifest": expected,
                "lookahead": report.lookahead,
                "propagationDelay": report.propagation_delay,
                "requiredBuffer": report.required_buffer,
                "unit": report.unit,
                "semanticProfile": report.semantic_profile,
                "corpusRevision": report.corpus_revision,
            }),
        },
        Err(error) => Row {
            symbol,
            family: "horizon",
            outcome: "fail",
            trace_ids: vec!["FR-002-AC-1", "TC-005"],
            detail: json!({
                "declaredByCorpusManifest": expected,
                "why": "horizon analysis refused a fixture the corpus declares valid",
                "error": format!("{error}"),
            }),
        },
    })
}

fn closed_row(fixture: &Fixture, corpus: &Path) -> Result<Option<Row>, String> {
    let Some(expected) = fixture.expected_closed_trace else {
        return Ok(None);
    };
    let symbol = format!("{}/closed", fixture.id);
    let document = load_valid(fixture, corpus)?;
    let formula = document.validate().map_err(|error| {
        format!(
            "fixture {} is declared valid but did not validate: {error}",
            fixture.id
        )
    })?;
    Ok(Some(
        match evaluate_closed(
            formula,
            fixture.id.clone(),
            &fixture.trace,
            format!("{}-trace", fixture.id),
            EvaluationLimits::default(),
        ) {
            Ok(report) => {
                let observed = match report.verdict {
                    TruthValue::True => Some(true),
                    TruthValue::False => Some(false),
                    TruthValue::Pending => None,
                };
                Row {
                    symbol,
                    family: "closed",
                    outcome: if observed == Some(expected) {
                        "pass"
                    } else {
                        "fail"
                    },
                    trace_ids: vec!["FR-001-AC-1", "FR-001-AC-2", "TC-001", "TC-003"],
                    detail: json!({
                        "declaredByCorpusManifest": expected,
                        "verdict": truth_name(report.verdict),
                        "verdictTime": report.verdict_time,
                        "traceLength": report.trace_length,
                        "traceClosed": report.trace_closed,
                        "semanticProfile": report.semantic_profile,
                        "schemaVersion": report.schema_version,
                    }),
                }
            }
            Err(error) => Row {
                symbol,
                family: "closed",
                outcome: "fail",
                trace_ids: vec!["FR-001-AC-1", "TC-001"],
                detail: json!({
                    "declaredByCorpusManifest": expected,
                    "why": "closed evaluation refused a fixture the corpus declares valid",
                    "error": format!("{error}"),
                }),
            },
        },
    ))
}

/// Does a rejection message name this declared cause?
///
/// One function, used both for a fixture's own outcome and for the
/// cross-fixture discrimination row below. That is deliberate: if this is ever
/// weakened to a constant, the diagonal still passes but the off-diagonal
/// collapses and `malformed-marker-discrimination` fails. A separate
/// implementation for each purpose would let one be neutered while the other
/// went on agreeing with itself.
fn marker_hits(message: &str, marker: &str) -> bool {
    message.contains(marker)
}

/// The rejection a declared-invalid fixture actually produced.
fn observe_rejection(fixture: &Fixture, corpus: &Path) -> Result<(&'static str, String), String> {
    let bytes = read_declared(corpus, &fixture.formula)?;
    Ok(match serde_json::from_slice::<FormulaDocument>(&bytes) {
        Err(error) => ("deserialize", error.to_string()),
        Ok(document) => match document.validate() {
            Err(error) => ("validate", format!("{error}")),
            Ok(_) => ("accepted", String::new()),
        },
    })
}

/// Every declared marker must name its own fixture's refusal and no other's.
///
/// Without this, "rejected for the declared reason" is satisfied by a marker
/// that matches everything, and an evaluator that refused every document for one
/// reason would look correct. Measured rather than assumed: disabling the
/// per-row cause check on a tree where every cause matches is invisible, so the
/// check needs an off-diagonal that must come out false.
fn marker_discrimination_row(fixtures: &[&Fixture], corpus: &Path) -> Result<Row, String> {
    let mut observed = Vec::new();
    for fixture in fixtures {
        let declared = fixture
            .expected_error
            .as_deref()
            .ok_or_else(|| format!("fixture {} declares no expected error", fixture.id))?;
        let (_, marker) = declared_rejection(declared)
            .ok_or_else(|| format!("fixture {} declares an unnamed rejection", fixture.id))?;
        let (_, message) = observe_rejection(fixture, corpus)?;
        observed.push((fixture.id.clone(), marker, message));
    }
    let mut collisions = Vec::new();
    let mut missing = Vec::new();
    for (id, marker, message) in &observed {
        if !marker_hits(message, marker) {
            missing.push(id.clone());
        }
        for (other_id, _, other_message) in &observed {
            if other_id == id {
                continue;
            }
            if marker_hits(other_message, marker) {
                collisions.push(format!("{marker} also matches {other_id}"));
            }
        }
    }
    let matched = collisions.is_empty() && missing.is_empty() && observed.len() > 1;
    Ok(Row {
        symbol: "malformed-marker-discrimination".to_owned(),
        family: "malformed",
        outcome: if matched { "pass" } else { "fail" },
        trace_ids: vec!["FR-001-AC-3", "NFR-002-AC-1", "TC-012"],
        detail: json!({
            "why": "each declared cause must name its own fixture's refusal and no other's",
            "fixtures": observed.len(),
            "markersThatMatchedNothing": missing,
            "collisions": collisions,
        }),
    })
}

fn malformed_row(fixture: &Fixture, corpus: &Path) -> Result<Row, String> {
    let symbol = format!("{}/malformed", fixture.id);
    let declared = fixture.expected_error.as_deref().ok_or_else(|| {
        format!(
            "fixture {} is declared invalid but the manifest states no expected error; \
             an obligation with no declared expectation is refused rather than skipped",
            fixture.id
        )
    })?;
    let expectation = declared_rejection(declared).ok_or_else(|| {
        format!(
            "fixture {} declares the error {declared:?}, which this replay does not name. \
             An unnamed rejection is refused rather than defaulted, because a defaulted \
             unknown is how a document rejected for the wrong reason reads as correct.",
            fixture.id
        )
    })?;
    let (expected_stage, marker) = expectation;
    let bytes = read_declared(corpus, &fixture.formula)?;
    // The outcome is decided by WHERE the refusal happened and by whether the
    // refusal names the declared cause. The stage is structural. The marker is
    // the only discriminator `FormulaDocument`'s deserializer offers between
    // three different wire-boundary refusals, and it is used deliberately and
    // named in `declared_rejection`'s comment rather than left implicit.
    let (observed_stage, message) = match serde_json::from_slice::<FormulaDocument>(&bytes) {
        Err(error) => ("deserialize", error.to_string()),
        Ok(document) => match document.validate() {
            Err(error) => ("validate", format!("{error}")),
            Ok(_) => ("accepted", String::new()),
        },
    };
    let stage_matched = observed_stage == expected_stage;
    let cause_matched = marker_hits(&message, marker);
    Ok(Row {
        symbol,
        family: "malformed",
        outcome: if stage_matched && cause_matched {
            "malformed"
        } else {
            "fail"
        },
        trace_ids: vec!["FR-001-AC-3", "NFR-002-AC-1", "TC-012"],
        detail: json!({
            "declaredByCorpusManifest": declared,
            "declaredStage": expected_stage,
            "observedStage": observed_stage,
            "declaredCauseMarker": marker,
            "declaredCauseObserved": cause_matched,
            "rejection": message,
        }),
    })
}

fn run(arguments: &[String]) -> Result<Vec<Row>, String> {
    let manifest_file = manifest_path(arguments)?;
    let corpus = manifest_file
        .parent()
        .ok_or_else(|| "the manifest has no parent directory".to_owned())?
        .to_path_buf();
    let raw = fs::read(&manifest_file)
        .map_err(|error| format!("read {}: {error}", manifest_file.display()))?;
    let manifest: Manifest = serde_json::from_slice(&raw)
        .map_err(|error| format!("parse {}: {error}", manifest_file.display()))?;
    if manifest.corpus_revision.is_empty() {
        return Err("the corpus manifest declares no revision".to_owned());
    }
    if manifest.fixtures.is_empty() {
        return Err(
            "the corpus manifest declares no fixtures; there is nothing to replay".to_owned(),
        );
    }

    let mut classes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut rows = Vec::new();
    let mut invalid: Vec<&Fixture> = Vec::new();
    for fixture in &manifest.fixtures {
        *classes.entry(fixture.class.as_str()).or_insert(0) += 1;
        match fixture.expected_validation.as_str() {
            "valid" => {
                rows.push(horizon_row(fixture, &corpus)?);
                if let Some(row) = closed_row(fixture, &corpus)? {
                    rows.push(row);
                }
            }
            "invalid" => {
                rows.push(malformed_row(fixture, &corpus)?);
                invalid.push(fixture);
            }
            other => {
                return Err(format!(
                    "fixture {} declares expected_validation {other:?}, which this replay \
                     does not name; an unnamed expectation is refused rather than skipped",
                    fixture.id
                ))
            }
        }
    }
    let _ = classes;
    rows.push(marker_discrimination_row(&invalid, &corpus)?);
    Ok(rows)
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(rows) => {
            for row in &rows {
                println!("{}", row.emit());
            }
            // A producer that reported a failing row exits non-zero. The rows are
            // the structured result the chain reads; this is the status the shell
            // reads, so `make conformance` cannot report success over a failure.
            if rows.iter().any(|row| row.outcome == "fail") {
                eprintln!("reference_conformance: at least one obligation was not discharged");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("reference_conformance: {error}");
            ExitCode::from(2)
        }
    }
}
