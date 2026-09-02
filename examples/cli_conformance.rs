//! Drive the built CLI over its declared request documents (FR-006-AC-2).
//!
//! This is a producer. It executes the real `tl-mltl` binary — not a
//! reimplementation of it, and not the library the binary happens to call — over
//! the request documents `tests/fixtures/cli-requests/manifest.json` declares,
//! and writes one structured row per case.
//!
//! It does not build the binary. `make cli-conformance` and
//! `make assurance-inputs` build it first and this example refuses to run when
//! it is absent, because an example that can build its own subject can report a
//! green run against a stale one, and a producer that silently rebuilds is a
//! producer that can make its own inputs.
//!
//! Determinism is measured, not asserted: every case is run twice and the two
//! stdout byte strings have to be identical.
//!
//! Row vocabulary, all of which the chain enumerates:
//!
//! - `pass` the obligation was discharged
//! - `fail` the CLI did not do what the manifest declares

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use serde::Deserialize;
use serde_json::{json, Value};

const PROTOCOL: &str = "tl-mltl.cli-conformance/v1";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    cases: Vec<Case>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Case {
    id: String,
    request: String,
    expected_exit: i32,
    #[serde(default)]
    required_fields: Vec<String>,
    stderr_marker: Option<String>,
    trace_ids: Vec<String>,
}

struct Row {
    symbol: String,
    outcome: &'static str,
    trace_ids: Vec<String>,
    detail: Value,
}

impl Row {
    fn emit(&self) -> String {
        json!({
            "protocol": PROTOCOL,
            "symbol": self.symbol,
            "family": "cli",
            "outcome": self.outcome,
            "traceIds": self.trace_ids,
            "detail": self.detail,
        })
        .to_string()
    }
}

fn argument(arguments: &[String], name: &str) -> Option<String> {
    let mut iterator = arguments.iter();
    while let Some(value) = iterator.next() {
        if value == name {
            return iterator.next().cloned();
        }
    }
    None
}

/// Where cargo put the CLI binary for this same profile.
///
/// Examples are built into `<target>/<profile>/examples/`, so the binary is one
/// directory up. Derived from this process's own path rather than from a
/// hard-coded `target/debug`, because `CARGO_TARGET_DIR` moves both together.
fn default_binary() -> Result<PathBuf, String> {
    let current = std::env::current_exe()
        .map_err(|error| format!("this example could not locate itself: {error}"))?;
    let profile = current
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| "this example is not inside a cargo profile directory".to_owned())?;
    Ok(profile.join("tl-mltl"))
}

/// Are two runs' outputs the same bytes?
///
/// One function, used both for the determinism comparison and for the
/// `determinism-discrimination` row that requires two *different* requests to
/// produce different output. Weakening it to a constant makes every case look
/// deterministic and simultaneously makes the discrimination row fail, which is
/// the only way a check that currently passes can be given a failure direction.
fn same_output(left: &[u8], right: &[u8]) -> bool {
    left == right
}

/// Does a refusal name this declared cause?
///
/// One function, used both for a case's own outcome and for the cross-case
/// discrimination row. If it is ever weakened to a constant, the case's own
/// check still passes but every marker starts matching every other case's
/// stderr and `marker-discrimination` fails. Two separate implementations would
/// let one be neutered while the other agreed with itself.
fn marker_hits(stderr: &str, marker: &str) -> bool {
    stderr.contains(marker)
}

fn run_once(binary: &Path, request: &Path) -> Result<(i32, Vec<u8>, String), String> {
    let output = Command::new(binary)
        .arg(request)
        .output()
        .map_err(|error| format!("could not run {}: {error}", binary.display()))?;
    Ok((
        output.status.code().unwrap_or(-1),
        output.stdout,
        String::from_utf8_lossy(&output.stderr).into_owned(),
    ))
}

fn case_row(binary: &Path, fixtures: &Path, case: &Case) -> Result<Row, String> {
    let request = fixtures.join(&case.request);
    if !request.is_file() {
        return Err(format!(
            "the manifest declares the request {}, which is not on disk. A declared \
             request that is absent is an error, not a refusal to report.",
            request.display()
        ));
    }
    let (first_code, first_stdout, first_stderr) = run_once(binary, &request)?;
    let (second_code, second_stdout, _) = run_once(binary, &request)?;

    let deterministic = same_output(&first_stdout, &second_stdout) && first_code == second_code;
    let exit_matched = first_code == case.expected_exit;

    let mut missing: Vec<&str> = Vec::new();
    let mut parsed_ok = true;
    if case.expected_exit == 0 {
        match serde_json::from_slice::<Value>(&first_stdout) {
            Ok(value) => {
                for field in &case.required_fields {
                    if value.get(field).is_none() {
                        missing.push(field.as_str());
                    }
                }
            }
            Err(_) => parsed_ok = false,
        }
    }

    // A refusal has no structured output, so the exit status is the primary
    // fact and the declared marker is the only discriminator between four
    // different refusals. The manifest says so explicitly; it is used here and
    // not hidden.
    let marker_observed = match case.stderr_marker.as_deref() {
        None => true,
        Some(marker) => marker_hits(&first_stderr, marker),
    };

    let matched =
        deterministic && exit_matched && parsed_ok && missing.is_empty() && marker_observed;
    Ok(Row {
        symbol: case.id.clone(),
        outcome: if matched { "pass" } else { "fail" },
        trace_ids: case.trace_ids.clone(),
        detail: json!({
            "declaredByRequestManifest": {
                "expectedExit": case.expected_exit,
                "requiredFields": case.required_fields,
                "stderrMarker": case.stderr_marker,
            },
            "observedExit": first_code,
            "repeatedExit": second_code,
            "stdoutBytes": first_stdout.len(),
            "deterministicAcrossTwoRuns": deterministic,
            "structuredOutputParsed": parsed_ok,
            "missingRequiredFields": missing,
            "declaredMarkerObserved": marker_observed,
            "stderr": first_stderr.trim(),
        }),
    })
}

fn run(arguments: &[String]) -> Result<Vec<Row>, String> {
    let requests = argument(arguments, "--requests")
        .ok_or_else(|| "usage: cli_conformance --requests PATH [--binary PATH]".to_owned())?;
    let manifest_file = PathBuf::from(requests);
    let fixtures = manifest_file
        .parent()
        .ok_or_else(|| "the request manifest has no parent directory".to_owned())?
        .to_path_buf();
    let binary = match argument(arguments, "--binary") {
        Some(value) => PathBuf::from(value),
        None => default_binary()?,
    };
    if !binary.is_file() {
        return Err(format!(
            "the CLI binary is not at {}. Run `cargo build --bin tl-mltl` first. This \
             producer executes the CLI and never builds it: a producer that can build \
             its own subject can report a green run against a stale one.",
            binary.display()
        ));
    }
    let raw = fs::read(&manifest_file)
        .map_err(|error| format!("read {}: {error}", manifest_file.display()))?;
    let manifest: Manifest = serde_json::from_slice(&raw)
        .map_err(|error| format!("parse {}: {error}", manifest_file.display()))?;
    if manifest.cases.is_empty() {
        return Err("the request manifest declares no cases; there is nothing to drive".to_owned());
    }
    let mut rows: Vec<Row> = manifest
        .cases
        .iter()
        .map(|case| case_row(&binary, &fixtures, case))
        .collect::<Result<Vec<Row>, String>>()?;

    // -- the markers have to discriminate -----------------------------------
    //
    // Every refusal exits 2, so the markers are what tell five different
    // refusals apart. A marker that also appears in another case's stderr tells
    // nothing apart, and a set of markers that all match everything would leave
    // a CLI which refused every request for one reason looking correct. So the
    // cross-product is measured rather than assumed.
    let mut collisions: Vec<Value> = Vec::new();
    let mut markers = 0;
    for case in &manifest.cases {
        let Some(marker) = case.stderr_marker.as_deref() else {
            continue;
        };
        markers += 1;
        for other in &manifest.cases {
            if other.id == case.id {
                continue;
            }
            let (_, _, stderr) = run_once(&binary, &fixtures.join(&other.request))?;
            if marker_hits(&stderr, marker) {
                collisions.push(json!({ "marker": marker, "alsoMatched": other.id }));
            }
        }
    }
    // -- the determinism comparison has to be able to say "different" -------
    //
    // Every accepted case is run twice and the two outputs compared. On a
    // deterministic CLI that comparison always says "same", so a comparison that
    // could only ever say "same" would be indistinguishable from a working one.
    // Two different requests are compared here and must differ. Measured rather
    // than assumed: hardcoding the determinism result to `true` on a
    // deterministic tree is otherwise invisible.
    let accepted: Vec<&Case> = manifest
        .cases
        .iter()
        .filter(|case| case.expected_exit == 0)
        .collect();
    let mut distinguishable = false;
    let mut compared = 0;
    if accepted.len() > 1 {
        let (_, left, _) = run_once(&binary, &fixtures.join(&accepted[0].request))?;
        let (_, right, _) = run_once(&binary, &fixtures.join(&accepted[1].request))?;
        compared = 2;
        distinguishable = !same_output(&left, &right) && !left.is_empty() && !right.is_empty();
    }
    rows.push(Row {
        symbol: "determinism-discrimination".to_owned(),
        outcome: if distinguishable { "pass" } else { "fail" },
        trace_ids: vec!["FR-005-AC-1".to_owned(), "NFR-001-AC-1".to_owned()],
        detail: json!({
            "why": "a comparison that can only ever report `same` measures nothing",
            "requestsCompared": compared,
            "outputsDiffer": distinguishable,
        }),
    });

    // -- the binary under test has to be the binary this revision builds ----
    //
    // `current_exe()` finds whatever is in the profile directory. If a build was
    // skipped, that is a binary from an older revision, and every row above
    // would report on it happily. The CLI stamps the revision it was built from
    // into its C2PO mapping manifest, so the two are compared: a stale binary
    // names a different revision than `git rev-parse HEAD` and fails here.
    let head = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(
            fixtures
                .parent()
                .and_then(Path::parent)
                .unwrap_or(&fixtures),
        )
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_owned())
        .unwrap_or_default();
    let mapping_request = fixtures.join("map-c2po.json");
    let (_, stdout, _) = run_once(&binary, &mapping_request)?;
    let stamped = serde_json::from_slice::<Value>(&stdout)
        .ok()
        .and_then(|value| value["sourceRevision"].as_str().map(str::to_owned))
        .unwrap_or_default();
    rows.push(Row {
        symbol: "binary-identity".to_owned(),
        outcome: if !head.is_empty() && stamped == head {
            "pass"
        } else {
            "fail"
        },
        trace_ids: vec!["FR-005-AC-1".to_owned(), "NFR-002-AC-2".to_owned()],
        detail: json!({
            "why": "the CLI under test must be the one this revision builds, not a \
                    leftover in the profile directory",
            "headRevision": head,
            "revisionStampedIntoTheBinary": stamped,
        }),
    });

    rows.push(Row {
        symbol: "marker-discrimination".to_owned(),
        outcome: if collisions.is_empty() && markers > 1 {
            "pass"
        } else {
            "fail"
        },
        trace_ids: vec!["FR-005-AC-1".to_owned(), "NFR-002-AC-1".to_owned()],
        detail: json!({
            "why": "each declared refusal marker must match its own case and no other",
            "markersDeclared": markers,
            "collisions": collisions,
        }),
    });
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
                eprintln!("cli_conformance: at least one obligation was not discharged");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("cli_conformance: {error}");
            ExitCode::from(2)
        }
    }
}
