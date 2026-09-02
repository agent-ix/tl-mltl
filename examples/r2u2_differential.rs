//! Replay the retained R2U2 4.2 exchange through the comparison layer (FR-006-AC-2).
//!
//! This is a producer, and it is the one that carries this repository's external
//! compatibility claim. It runs the real crate over the real retained R2U2
//! artifacts and writes one declared structured row per obligation.
//!
//! It does not execute R2U2. The external monitor was run once, out of band, and
//! its exact stdout, spec binary, signal map, trace and tool identities are
//! retained under `corpus/r2u2-v4.2/` and pinned by SHA-256 in that directory's
//! own manifest. This replay reads those bytes and compares. Executing the
//! external monitor from an assurance gate would make the gate a producer of the
//! thing it is checking.
//!
//! **A differential result is never a boolean here.** Every row retains, as
//! separate fields, the comparison classification, the reference truth value,
//! the reference verdict time, the external tool's execution status, its value
//! and its verdict time. `agreement` and `mismatch` and `non_conclusive` are
//! three answers, and `pending`, `unsupported` and `tool_error` are three
//! different reasons to be non-conclusive. Collapsing any of them into one bit
//! is the exact loss this producer exists to prevent, so each is emitted by its
//! own row family and each is asserted separately by the chain.
//!
//! Row vocabulary, all of which the chain enumerates:
//!
//! - `pass` — the obligation was discharged
//! - `fail` — the crate disagreed with what the retained exchange declares
//! - `unsupported` — a case the corpus manifest declares outside the adapter's
//!   profile was reported unsupported and was never compared
//!
//! `unsupported` maps to a passing proof because the OBLIGATION — a case the
//! adapter does not support is reported as unsupported and is not silently
//! compared — was discharged. It is not a pass of the case. The word survives
//! into the row, into the bytes Quoin retains, and into a chain scenario whose
//! count oracle is the corpus manifest rather than this producer's own output.

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

use serde::Deserialize;
use serde_json::{json, Value};
use tl_mltl::{
    analyze_horizon, compare_external, evaluate_prefix, evaluate_prefix_at, map_to_c2po,
    ComparisonStatus, EvaluationLimits, ExternalStatus, ExternalVerdict, MappingError,
    MappingSourceIdentity, MappingSourceState, ToolIdentity, TruthValue,
};
use tl_syntax::{FormulaDocument, PropositionId};

const PROTOCOL: &str = "tl-mltl.r2u2-differential/v1";
const TRACE_ID: &str = "r2u2-v4.2-trace";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    source: Source,
    tools: Tools,
    artifacts: BTreeMap<String, String>,
    trace: Vec<Vec<PropositionId>>,
    cases: Vec<Case>,
    unsupported_cases: Vec<UnsupportedCase>,
}

#[derive(Deserialize)]
struct Source {
    revision: String,
    license: String,
}

#[derive(Deserialize)]
struct Tools {
    r2u2: R2u2,
}

#[derive(Deserialize)]
struct R2u2 {
    version: String,
    sha256: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    formula_index: u32,
    formula: FormulaDocument,
    expected: Expected,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Expected {
    verdict: bool,
    verdict_time: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UnsupportedCase {
    id: String,
    reason: String,
    /// The out-of-profile formula the adapter must refuse.
    ///
    /// Required, not optional. An adversarial review found this case carrying
    /// only an id and a reason, so the producer printed `unsupported` by
    /// iterating a list rather than by anything refusing anything — deleting
    /// the profile check from `map_to_c2po` changed nothing. The formula makes
    /// the row falsifiable.
    formula: FormulaDocument,
    expected_refusal: String,
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

fn comparison_name(status: ComparisonStatus) -> &'static str {
    match status {
        ComparisonStatus::Agreement => "agreement",
        ComparisonStatus::Mismatch => "mismatch",
        ComparisonStatus::NonConclusive => "non_conclusive",
    }
}

fn external_status_name(status: ExternalStatus) -> &'static str {
    match status {
        ExternalStatus::Conclusive => "conclusive",
        ExternalStatus::Pending => "pending",
        ExternalStatus::Unsupported => "unsupported",
        ExternalStatus::ToolError => "tool_error",
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
    Err("usage: r2u2_differential --manifest PATH".to_owned())
}

/// The external monitor's own retained stdout, indexed by (formula, time).
///
/// The external values come from the retained run and from nowhere else. There
/// is no fallback: a case whose external record is absent is reported as a
/// failure to compare rather than compared against a value this producer chose.
fn time_indexed_verdicts(stdout: &str) -> BTreeMap<(u32, u64), bool> {
    stdout
        .lines()
        .filter_map(|line| {
            let (identity_time, value) = line.split_once(',')?;
            let (identity, time) = identity_time.split_once(':')?;
            Some(((identity.parse().ok()?, time.parse().ok()?), value == "T"))
        })
        .collect()
}

fn tool_identity(manifest: &Manifest) -> ToolIdentity {
    ToolIdentity {
        name: "r2u2".to_owned(),
        version: manifest.tools.r2u2.version.clone(),
        executable_sha256: manifest.tools.r2u2.sha256.clone(),
        configuration_sha256: manifest
            .artifacts
            .get("spec.bin")
            .cloned()
            .unwrap_or_default(),
    }
}

#[allow(clippy::too_many_lines)]
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
    if manifest.cases.is_empty() {
        return Err("the R2U2 manifest declares no cases; there is nothing to replay".to_owned());
    }
    if manifest.source.license != "Apache-2.0" {
        return Err(format!(
            "the retained R2U2 source declares license {:?}; the retained copy is Apache-2.0",
            manifest.source.license
        ));
    }
    let stdout = fs::read_to_string(corpus.join("r2u2.stdout"))
        .map_err(|error| format!("read the retained R2U2 stdout: {error}"))?;
    let external = time_indexed_verdicts(&stdout);
    if external.is_empty() {
        return Err("the retained R2U2 stdout carries no verdict rows".to_owned());
    }
    let tool = tool_identity(&manifest);
    let source = MappingSourceIdentity {
        revision: env!("TL_MLTL_SOURCE_REVISION").to_owned(),
        state: MappingSourceState::parse(env!("TL_MLTL_SOURCE_STATE"))
            .ok_or_else(|| "the build script emitted an unknown source state".to_owned())?,
    };

    let mut rows: Vec<Row> = Vec::new();

    for case in &manifest.cases {
        let formula = case
            .formula
            .validate()
            .map_err(|error| format!("case {} does not validate: {error}", case.id))?;

        // -- artifact exchange: the C2PO expression the adapter would hand over --
        let formula_bytes = serde_json::to_vec(&case.formula)
            .map_err(|error| format!("case {} did not serialize: {error}", case.id))?;
        let mapping = map_to_c2po(
            formula,
            case.id.clone(),
            &formula_bytes,
            source.clone(),
            Some(tool.clone()),
            100_000,
        );
        rows.push(match mapping {
            Ok(manifest_row) => Row {
                symbol: format!("{}/mapping", case.id),
                family: "mapping",
                outcome: "pass",
                trace_ids: vec!["FR-004-AC-1", "FR-004-AC-3", "TC-011", "TC-013"],
                detail: json!({
                    "expression": manifest_row.expression,
                    "inputSha256": manifest_row.input_sha256,
                    "outputSha256": manifest_row.output_sha256,
                    "propositionIds": manifest_row.proposition_ids,
                    "adapterVersion": manifest_row.adapter_version,
                    "syntaxRevision": manifest_row.syntax_revision,
                    "sourceRevision": manifest_row.source_revision,
                    "sourceState": manifest_row.source_state,
                    "externalTool": manifest_row.external_tool,
                    "limitation": manifest_row.limitation,
                }),
            },
            Err(error) => Row {
                symbol: format!("{}/mapping", case.id),
                family: "mapping",
                outcome: "fail",
                trace_ids: vec!["FR-004-AC-1", "TC-011"],
                detail: json!({ "error": format!("{error:?}") }),
            },
        });

        // -- the reference verdict at the declared verdict time ------------------
        let reference = evaluate_prefix_at(
            formula,
            case.id.clone(),
            &manifest.trace,
            TRACE_ID,
            false,
            case.expected.verdict_time,
            EvaluationLimits::default(),
        )
        .map_err(|error| format!("case {} did not evaluate: {error}", case.id))?;

        // -- the differential comparison against the retained external record ----
        let observed = external.get(&(case.formula_index, case.expected.verdict_time));
        let symbol = format!("{}/differential", case.id);
        match observed {
            None => rows.push(Row {
                symbol,
                family: "differential",
                outcome: "fail",
                trace_ids: vec!["FR-005-AC-2", "TC-015"],
                detail: json!({
                    "why": "the retained R2U2 stdout carries no record for this case",
                    "formulaIndex": case.formula_index,
                    "verdictTime": case.expected.verdict_time,
                }),
            }),
            Some(value) => {
                let verdict = ExternalVerdict {
                    schema_version: "tl-mltl.external-verdict/v1".to_owned(),
                    tool: tool.clone(),
                    formula_id: case.id.clone(),
                    trace_id: TRACE_ID.to_owned(),
                    status: ExternalStatus::Conclusive,
                    value: Some(*value),
                    verdict_time: Some(case.expected.verdict_time),
                    detail: None,
                };
                let report = compare_external(&reference, verdict.clone());
                let expected_reference = if case.expected.verdict {
                    TruthValue::True
                } else {
                    TruthValue::False
                };
                let matched = report.status == ComparisonStatus::Agreement
                    && reference.verdict == expected_reference
                    && reference.verdict_time == case.expected.verdict_time;
                rows.push(Row {
                    symbol,
                    family: "differential",
                    outcome: if matched { "pass" } else { "fail" },
                    trace_ids: vec!["FR-005-AC-2", "StR-002-VC-1", "TC-015"],
                    detail: json!({
                        "comparison": comparison_name(report.status),
                        "referenceVerdict": truth_name(report.reference_verdict),
                        "referenceVerdictTime": report.reference_verdict_time,
                        "declaredByCorpusManifest": {
                            "verdict": case.expected.verdict,
                            "verdictTime": case.expected.verdict_time,
                        },
                        "externalStatus": external_status_name(verdict.status),
                        "externalValue": verdict.value,
                        "externalVerdictTime": verdict.verdict_time,
                        "externalTool": verdict.tool,
                        "detail": report.detail,
                        "schemaVersion": report.schema_version,
                    }),
                });

                // -- negative control: the mismatch direction has to be live -----
                //
                // The retained exchange agrees, and it has to keep agreeing. A
                // producer that only ever emits `agreement` is indistinguishable
                // from one whose comparison always returns it, so each case is
                // also compared against the same external record moved one
                // instant later, which must classify as a mismatch.
                let mut moved = verdict;
                moved.verdict_time = moved.verdict_time.and_then(|time| time.checked_add(1));
                let control = compare_external(&reference, moved.clone());
                rows.push(Row {
                    symbol: format!("{}/differential-control", case.id),
                    family: "differential-control",
                    outcome: if control.status == ComparisonStatus::Mismatch {
                        "pass"
                    } else {
                        "fail"
                    },
                    trace_ids: vec!["FR-005-AC-2", "TC-015"],
                    detail: json!({
                        "why": "the same external record moved one instant later must not agree",
                        "comparison": comparison_name(control.status),
                        "externalVerdictTime": moved.verdict_time,
                        "referenceVerdictTime": control.reference_verdict_time,
                        "detail": control.detail,
                    }),
                });
            }
        }

        // -- the open-prefix obligation: pending is decided by the horizon -------
        //
        // The expectation is not hand-written. `analyze_horizon` states how far
        // ahead the formula has to see; a prefix shorter than that cannot be
        // decided, and one at least that long must be. The oracle is the
        // formula's own declared lookahead, so a producer that stopped
        // reporting `pending` cannot also move the number it is checked
        // against.
        let horizon = analyze_horizon(formula, case.id.clone())
            .map_err(|error| format!("case {} horizon analysis failed: {error}", case.id))?;
        let empty: Vec<Vec<PropositionId>> = Vec::new();
        let prefix = evaluate_prefix(
            formula,
            case.id.clone(),
            &empty,
            TRACE_ID,
            false,
            EvaluationLimits::default(),
        )
        .map_err(|error| format!("case {} prefix evaluation failed: {error}", case.id))?;
        let must_be_pending = horizon.lookahead > 0;
        let is_pending = prefix.verdict == TruthValue::Pending;
        rows.push(Row {
            symbol: format!("{}/prefix", case.id),
            family: "prefix",
            outcome: if must_be_pending == is_pending {
                "pass"
            } else {
                "fail"
            },
            trace_ids: vec!["FR-003-AC-1", "FR-003-AC-3", "TC-009"],
            detail: json!({
                "why": "an empty open prefix is undecided exactly when the formula must look ahead",
                "lookahead": horizon.lookahead,
                "observedInstants": prefix.trace_length,
                "traceClosed": prefix.trace_closed,
                "verdict": truth_name(prefix.verdict),
                "expectedPending": must_be_pending,
            }),
        });
    }

    // -- every non-conclusive external state, kept apart ------------------------
    //
    // Three different reasons an external monitor produces no comparable answer.
    // They are all `non_conclusive` to the comparison layer, and they must stay
    // three distinct retained facts rather than one. The reference side is a
    // real evaluation of a real case; only the external side is constructed,
    // because these are states the retained run does not contain and the
    // alternative is not demonstrating them at all.
    let probe_case = manifest
        .cases
        .first()
        .ok_or_else(|| "no case to build the external-state controls from".to_owned())?;
    let probe_formula = probe_case
        .formula
        .validate()
        .map_err(|error| format!("probe case does not validate: {error}"))?;
    let probe_reference = evaluate_prefix_at(
        probe_formula,
        probe_case.id.clone(),
        &manifest.trace,
        TRACE_ID,
        false,
        probe_case.expected.verdict_time,
        EvaluationLimits::default(),
    )
    .map_err(|error| format!("probe case did not evaluate: {error}"))?;
    for status in [
        ExternalStatus::Pending,
        ExternalStatus::Unsupported,
        ExternalStatus::ToolError,
    ] {
        let verdict = ExternalVerdict {
            schema_version: "tl-mltl.external-verdict/v1".to_owned(),
            tool: tool.clone(),
            formula_id: probe_case.id.clone(),
            trace_id: TRACE_ID.to_owned(),
            status,
            value: None,
            verdict_time: None,
            detail: Some(format!(
                "external monitor reported {}",
                external_status_name(status)
            )),
        };
        let report = compare_external(&probe_reference, verdict.clone());
        rows.push(Row {
            symbol: format!("external-state/{}", external_status_name(status)),
            family: "external-state",
            outcome: if report.status == ComparisonStatus::NonConclusive {
                "pass"
            } else {
                "fail"
            },
            trace_ids: vec!["FR-005-AC-2", "NFR-002-AC-1", "TC-015"],
            detail: json!({
                "why": "a non-conclusive external state is never folded into agreement",
                "externalStatus": external_status_name(status),
                "externalValue": verdict.value,
                "externalVerdictTime": verdict.verdict_time,
                "externalDetail": verdict.detail,
                "comparison": comparison_name(report.status),
                "referenceVerdict": truth_name(report.reference_verdict),
                "detail": report.detail,
            }),
        });
    }

    // -- the cases the corpus declares outside the adapter's profile ------------
    //
    // The refusal is performed, not asserted. Each declared case carries a real
    // out-of-profile formula and it is handed to the adapter; the row is
    // `unsupported` only when the adapter actually refuses it with the declared
    // error. Deleting the profile check from `map_to_c2po` turns this row
    // `fail`, which is the whole point: before the formula was added, the row
    // was an echo of a list the producer had just iterated and nothing could
    // make it false.
    for case in &manifest.unsupported_cases {
        let formula = match case.formula.validate() {
            Ok(value) => value,
            Err(error) => {
                rows.push(Row {
                    symbol: format!("{}/declared-unsupported", case.id),
                    family: "declared-unsupported",
                    outcome: "fail",
                    trace_ids: vec!["FR-004-AC-2", "TC-012"],
                    detail: json!({
                        "why": "the declared out-of-profile formula does not validate",
                        "error": format!("{error}"),
                    }),
                });
                continue;
            }
        };
        let formula_bytes = serde_json::to_vec(&case.formula)
            .map_err(|error| format!("case {} did not serialize: {error}", case.id))?;
        let refusal = map_to_c2po(
            formula,
            case.id.clone(),
            &formula_bytes,
            source.clone(),
            Some(tool.clone()),
            100_000,
        );
        let (refused, observed) = match &refusal {
            Err(MappingError::UnsupportedProfile { actual }) => (
                case.expected_refusal == "unsupported_profile",
                format!("unsupported_profile:{actual}"),
            ),
            Err(other) => (false, format!("{other:?}")),
            Ok(manifest_row) => (
                false,
                format!("accepted and produced {:?}", manifest_row.expression),
            ),
        };
        rows.push(Row {
            symbol: format!("{}/declared-unsupported", case.id),
            family: "declared-unsupported",
            outcome: if refused { "unsupported" } else { "fail" },
            trace_ids: vec!["FR-004-AC-2", "FR-005-AC-2", "TC-012", "TC-015"],
            detail: json!({
                "why": "the corpus manifest declares this case outside the adapter's \
                        profile; the adapter is asked and must refuse, and the case is \
                        never handed to the external monitor",
                "reason": case.reason,
                "declaredRefusal": case.expected_refusal,
                "observedRefusal": observed,
                "declaredProfile": "mltl.closed-trace/v1",
                "sourceRevision": manifest.source.revision,
            }),
        });
    }

    Ok(rows)
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(rows) => {
            for row in &rows {
                println!("{}", row.emit());
            }
            if rows.iter().any(|row| row.outcome == "fail") {
                eprintln!("r2u2_differential: at least one obligation was not discharged");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("r2u2_differential: {error}");
            ExitCode::from(2)
        }
    }
}
