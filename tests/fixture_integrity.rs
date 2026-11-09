//! Fixture-integrity tests (TL-365): a record with a missing or unknown field is
//! refused, a removed or changed case names its fixture, and the retained corpus
//! bytes still match the identities recorded with them. Every mutation is made to
//! an in-memory value or a temporary copy; the retained bytes are never written.

use std::{fs, path::Path};

use serde_json::{json, Value};
use tl_mltl::campaign::sha256_hex;
use tl_mltl::r2u2_future::run_future;
use tl_mltl::r2u2_live::{CaseStatus, Source};
use tl_mltl::{ClockBinding, ExternalVerdict, PositionHistoryDocument, PositionObservation};
use tl_syntax::{FormulaDocument, PropositionId};

const REPO: &str = env!("CARGO_MANIFEST_DIR");

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .unwrap()
}

/// Removes each non-null top-level field in turn and adds one unknown field;
/// every variant must be refused. Returns the number of removals checked.
fn assert_missing_and_unknown_fields_refused(
    what: &str,
    optional: &[&str],
    value: &Value,
    accepts: &dyn Fn(Value) -> bool,
) -> usize {
    assert!(
        accepts(value.clone()),
        "{what}: the unmutated record must be accepted"
    );
    let object = value.as_object().unwrap();
    let mut removals = 0;
    for (key, field) in object {
        if field.is_null() || optional.contains(&key.as_str()) {
            continue;
        }
        let mut missing = value.clone();
        missing.as_object_mut().unwrap().remove(key);
        assert!(
            !accepts(missing),
            "{what}: a record missing `{key}` was accepted"
        );
        removals += 1;
    }
    let mut unknown = value.clone();
    unknown
        .as_object_mut()
        .unwrap()
        .insert("unexpectedField".to_owned(), json!(true));
    assert!(
        !accepts(unknown),
        "{what}: a record with an unknown field was accepted"
    );
    removals
}

fn history_value() -> Value {
    let history = PositionHistoryDocument::new(
        "event-main",
        1,
        0,
        1,
        Some(ClockBinding::EventPosition),
        vec![
            PositionObservation::new(0, vec![PropositionId(0)], None),
            PositionObservation::new(1, vec![], None),
        ],
    )
    .unwrap();
    serde_json::to_value(&history).unwrap()
}

fn formula_value() -> Value {
    let cases = read_json(&Path::new(tl_syntax::CORPUS_DIR).join("past-history/cases.json"));
    cases["formulas"][0]["document"].clone()
}

fn verdict_value() -> Value {
    let manifest = read_json(&Path::new(REPO).join("corpus/r2u2-v4.2/manifest.json"));
    json!({
        "schemaVersion": "tl-mltl.external-verdict/v1",
        "tool": {"name": "r2u2", "version": manifest["tools"]["r2u2"]["version"]},
        "formulaId": "r2u2-future-witness-v1",
        "traceId": "r2u2-v4.2-trace",
        "status": "conclusive",
        "value": true,
        "verdictTime": 0,
        "detail": null,
    })
}

// Trace: TC-098, TC-099, FR-009-AC-1
#[test]
fn canonical_and_retained_records_with_a_missing_or_unknown_field_are_refused() {
    let history = history_value();
    let removed =
        assert_missing_and_unknown_fields_refused("position history", &[], &history, &|v| {
            serde_json::from_value::<PositionHistoryDocument>(v).is_ok()
        });
    assert!(removed >= 6, "history checked {removed} fields");

    let formula = formula_value();
    let removed = assert_missing_and_unknown_fields_refused("formula", &[], &formula, &|v| {
        serde_json::from_value::<FormulaDocument>(v).is_ok_and(|d| d.validate().is_ok())
    });
    assert!(removed >= 3, "formula checked {removed} fields");

    let verdict = verdict_value();
    let removed = assert_missing_and_unknown_fields_refused(
        "retained observation",
        &["value", "verdictTime", "detail"],
        &verdict,
        &|v| serde_json::from_value::<ExternalVerdict>(v).is_ok(),
    );
    assert!(removed >= 5, "observation checked {removed} fields");

    // A nested record is held to the same rule.
    let mut nested = history_value();
    nested["observations"][0]["unexpectedField"] = json!(1);
    assert!(serde_json::from_value::<PositionHistoryDocument>(nested).is_err());
    let mut tool = verdict_value();
    tool["tool"]["unexpectedField"] = json!(1);
    assert!(serde_json::from_value::<ExternalVerdict>(tool).is_err());
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dest = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_dir(&entry.path(), &dest);
        } else {
            fs::copy(entry.path(), dest).unwrap();
        }
    }
}

fn future_copy() -> tempfile::TempDir {
    let copy = tempfile::tempdir().unwrap();
    copy_dir(
        &Path::new(REPO).join("corpus/r2u2-live-v1/future"),
        copy.path(),
    );
    copy
}

fn replay(dir: &Path) -> Result<tl_mltl::r2u2_live::Report, String> {
    run_future(Path::new(REPO), &Source::Recorded, dir)
}

// Trace: TC-099, TC-101, FR-009-AC-3
#[test]
fn a_removed_or_changed_recorded_result_fails_replay_and_names_the_fixture() {
    let clean = future_copy();
    assert_eq!(replay(clean.path()).unwrap().outcome, "pass");
    let group = "00-00-00";

    // A removed observation is refused and names its group.
    let removed = future_copy();
    fs::remove_file(
        removed
            .path()
            .join("outputs")
            .join(group)
            .join("r2u2.stdout"),
    )
    .unwrap();
    let error = replay(removed.path()).unwrap_err();
    assert!(error.contains(group), "{error}");

    // A changed observation no longer matches the recorded digest.
    let changed = future_copy();
    let stdout = changed
        .path()
        .join("outputs")
        .join(group)
        .join("r2u2.stdout");
    let mut bytes = fs::read(&stdout).unwrap();
    let flip = bytes.iter().position(|b| *b == b'T' || *b == b'F').unwrap();
    bytes[flip] = if bytes[flip] == b'T' { b'F' } else { b'T' };
    fs::write(&stdout, &bytes).unwrap();
    let error = replay(changed.path()).unwrap_err();
    assert!(error.contains(group), "{error}");

    // A removed case: the recorded report no longer lists the group.
    let dropped = future_copy();
    let path = dropped.path().join("report.json");
    let mut report = read_json(&path);
    report["groups"]
        .as_array_mut()
        .unwrap()
        .retain(|g| g["group"] != group);
    fs::write(&path, serde_json::to_vec(&report).unwrap()).unwrap();
    let error = replay(dropped.path()).unwrap_err();
    assert!(error.contains(group), "{error}");

    // A changed result that is re-recorded consistently still fails the
    // comparison, and the failing row names its case.
    let rerecorded = future_copy();
    let stdout = rerecorded
        .path()
        .join("outputs")
        .join(group)
        .join("r2u2.stdout");
    let mut bytes = fs::read(&stdout).unwrap();
    let flip = bytes.iter().position(|b| *b == b'T' || *b == b'F').unwrap();
    bytes[flip] = if bytes[flip] == b'T' { b'F' } else { b'T' };
    fs::write(&stdout, &bytes).unwrap();
    let path = rerecorded.path().join("report.json");
    let mut report = read_json(&path);
    for record in report["groups"].as_array_mut().unwrap() {
        if record["group"] == group {
            record["stdoutSha256"] = json!(sha256_hex(&bytes));
        }
    }
    fs::write(&path, serde_json::to_vec(&report).unwrap()).unwrap();
    let report = replay(rerecorded.path()).unwrap();
    assert_eq!(report.outcome, "fail");
    let bad: Vec<_> = report
        .cases
        .iter()
        .filter(|c| c.status != CaseStatus::Agree)
        .collect();
    assert!(!bad.is_empty());
    assert!(bad
        .iter()
        .all(|c| c.group == group && c.id.ends_with(group)));
}

// Trace: TC-099, FR-009-AC-5
#[test]
fn retained_corpus_bytes_match_the_identities_recorded_with_them() {
    let root = Path::new(REPO).join("corpus/r2u2-live-v1");
    for suite in ["past", "future"] {
        let dir = root.join(suite);
        let report = read_json(&dir.join("report.json"));
        let replay = read_json(&dir.join("replay-report.json"));
        assert_eq!(report["mode"], "ran", "{suite}");
        assert_eq!(report["suite"], suite);
        assert_eq!(
            replay["groups"], report["groups"],
            "{suite}: replay identities"
        );
        let groups = report["groups"].as_array().unwrap();
        assert!(!groups.is_empty());
        for group in groups {
            let name = group["group"].as_str().unwrap();
            let digest = |relative: String| sha256_hex(&fs::read(dir.join(relative)).unwrap());
            assert_eq!(
                group["specSha256"],
                digest(format!("inputs/{name}/spec.c2po")),
                "{suite}/{name} spec"
            );
            assert_eq!(
                group["traceSha256"],
                digest(format!("inputs/{name}/trace.csv")),
                "{suite}/{name} trace"
            );
            if !group["stdoutSha256"].is_null() {
                assert_eq!(
                    group["stdoutSha256"],
                    digest(format!("outputs/{name}/r2u2.stdout")),
                    "{suite}/{name} stdout"
                );
            }
        }
    }

    // The retained exchange and mapping corpus keep their corpus identity.
    let exchange = read_json(&Path::new(REPO).join("corpus/r2u2-v4.2/manifest.json"));
    assert_eq!(exchange["schemaVersion"], "tl-mltl.r2u2-corpus/v1");
    assert_eq!(exchange["source"]["tag"], "4.2-release");
    assert_eq!(exchange["tools"]["r2u2"]["version"], "4.2-release");
    let mapping = read_json(&Path::new(REPO).join("corpus/past-c2po-v1/manifest.json"));
    assert_eq!(mapping["schemaVersion"], "tl-mltl.past-c2po-corpus/v1");
}
