//! TL-367: a canonical case or retained observation with generated provenance
//! or non-canonical shape makes its owning corpus check fail and names the
//! affected cell or fixture. TL-357 only catches byte-identical copies of
//! canonical files in generated directories; these detectors catch a canonical
//! entry stamped with generated provenance or carrying a non-canonical shape.
//!
//! What is detected: generated provenance keys, generated-input names,
//! non-canonical fields, a missing hand-written rationale, a label or shape that
//! disagrees with the formula, a report that is not a live run or whose tool or
//! revision is not the pin, a missing or non-raw R2U2 output, and an output
//! group the recorded report does not list.
//!
//! Non-claims: there is no detector for a hand-authored case whose `expected` or
//! `trace` was edited without touching labels or provenance (the oracle replay in
//! `boundary_corpus.rs` owns that); `corpus/r2u2-v4.2` is not covered; and there
//! is no byte-level re-serialisation detector. These tests therefore back
//! NFR-004-AC-2, TC-099 and FR-009-AC-4 partly, not in full.
//!
//! Each detector reads a corpus root, so it is proven on a temporary copy: a
//! substituted entry is injected, the detector must fail and name it, and the
//! retained bytes are never written.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};

const REPO: &str = env!("CARGO_MANIFEST_DIR");

/// Keys a hand-authored boundary case carries; anything else is not canonical.
/// tl-syntax exposes no schema or typed struct for `boundary-v1`, so this list
/// is read off `boundary-v1/cases.json`; a legitimate new corpus field must be
/// added here.
const CASE_KEYS: [&str; 14] = [
    "id",
    "profile",
    "operator",
    "shape",
    "kind",
    "edges",
    "formula",
    "trace",
    "at",
    "expected",
    "rationale",
    "closed",
    "checkedBy",
    "note",
];
const GENERATED_KEYS: [&str; 6] = [
    "provenance",
    "found_by",
    "found_input",
    "generator",
    "minimised",
    "seed",
];
const GENERATED_MARKERS: [&str; 3] = ["fuzz/corpus", "fuzz/artifacts", "proptest-regressions"];

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap_or_else(|e| panic!("{}: {e}", path.display())))
        .unwrap()
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

fn mentions_generated(value: &Value) -> bool {
    match value {
        Value::String(s) => GENERATED_MARKERS.iter().any(|m| s.contains(m)),
        Value::Array(items) => items.iter().any(mentions_generated),
        Value::Object(map) => map.values().any(mentions_generated),
        _ => false,
    }
}

/// The interval shape a boundary case's formula actually has.
fn derived_shape(formula: &Value) -> &'static str {
    let Some(bounds) = formula.get("interval").and_then(Value::as_array) else {
        return "not-applicable";
    };
    let (a, b) = (bounds[0].as_u64().unwrap(), bounds[1].as_u64().unwrap());
    match (a, b) {
        (0, 0) => "point-zero",
        (_, b) if b == u64::from(u32::MAX) => "max",
        (a, b) if a == b => "point",
        (0, _) => "from-zero",
        _ => "interior",
    }
}

/// Why `case` is not a hand-authored boundary case, or nothing if it is.
fn case_origin_problems(case: &Value) -> Vec<String> {
    let mut problems = Vec::new();
    let object = case.as_object().expect("a case is an object");
    for key in object.keys() {
        if GENERATED_KEYS.contains(&key.as_str()) {
            problems.push(format!("carries generated provenance `{key}`"));
        } else if !CASE_KEYS.contains(&key.as_str()) {
            problems.push(format!("carries non-canonical field `{key}`"));
        }
    }
    if mentions_generated(case) {
        problems.push("names a generated input".to_owned());
    }
    if object
        .get("rationale")
        .and_then(Value::as_str)
        .is_none_or(|r| r.trim().is_empty())
    {
        problems.push("has no hand-written rationale".to_owned());
    }
    if let (Some(op), Some(formula)) = (object.get("operator"), object.get("formula")) {
        if formula["op"] != *op {
            problems.push("operator label does not match its formula".to_owned());
        }
        if object.get("shape").and_then(Value::as_str) != Some(derived_shape(formula)) {
            problems.push("shape label does not match its interval".to_owned());
        }
    }
    problems
}

/// Every substituted case in the boundary corpus under `root`, each named by
/// its cell (profile, operator, shape) and id.
fn boundary_substitutions(root: &Path) -> Vec<String> {
    let doc = read_json(&root.join("boundary-v1/cases.json"));
    let mut found = Vec::new();
    if mentions_generated(&json!(doc["note"])) || doc["note"].as_str().is_none() {
        found.push("boundary-v1: header note missing or generated".to_owned());
    }
    for case in doc["cases"].as_array().unwrap() {
        let problems = case_origin_problems(case);
        if !problems.is_empty() {
            found.push(format!(
                "boundary case {} ({} {} {}): {}",
                case["id"].as_str().unwrap_or("<no id>"),
                case["profile"].as_str().unwrap_or("?"),
                case["operator"].as_str().unwrap_or("?"),
                case["shape"].as_str().unwrap_or("?"),
                problems.join("; ")
            ));
        }
    }
    found
}

/// A line of raw R2U2 output: `<step>:<formula>,<T|F>`.
fn is_raw_r2u2_line(line: &str) -> bool {
    let Some((position, rest)) = line.split_once(':') else {
        return false;
    };
    let Some((formula, verdict)) = rest.split_once(',') else {
        return false;
    };
    let digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    digits(position) && digits(formula) && (verdict == "T" || verdict == "F")
}

fn pinned_revision() -> String {
    let pin = read_json(&Path::new(REPO).join("campaign/tool-versions.json"));
    pin["r2u2"]["revision"].as_str().unwrap().to_owned()
}

/// Every retained observation under `suite` (one `r2u2-live-v1` suite
/// directory) whose provenance or content is not a recorded live R2U2 run,
/// each named by suite and group.
fn observation_substitutions(suite: &Path, pinned: &str) -> Vec<String> {
    let name = suite.file_name().unwrap().to_string_lossy().into_owned();
    let report = read_json(&suite.join("report.json"));
    let mut found = Vec::new();
    if report["mode"] != "ran" {
        found.push(format!(
            "{name}: report is not a live run ({})",
            report["mode"]
        ));
    }
    if !report["toolDetail"].is_null() {
        found.push(format!("{name}: report carries a tool-unavailable detail"));
    }
    for revision in [
        &report["pin"]["revision"],
        &report["observed"]["r2u2Revision"],
    ] {
        if revision != pinned {
            found.push(format!(
                "{name}: observation revision {revision} is not the pin"
            ));
        }
    }
    let groups = report["groups"].as_array().unwrap();
    let recorded: Vec<&str> = groups
        .iter()
        .map(|g| g["group"].as_str().unwrap())
        .collect();
    // A group with no recorded stdout digest was refused at export and has no
    // observation; every other group must have its raw output.
    let observed: Vec<&str> = groups
        .iter()
        .filter(|g| !g["stdoutSha256"].is_null())
        .map(|g| g["group"].as_str().unwrap())
        .collect();
    for group in &observed {
        let stdout = suite.join("outputs").join(group).join("r2u2.stdout");
        match fs::read_to_string(&stdout) {
            Ok(text) => {
                if let Some(line) = text.lines().find(|l| !is_raw_r2u2_line(l)) {
                    found.push(format!(
                        "{name}/{group}: stdout is not raw R2U2 output (`{line}`)"
                    ));
                }
            }
            Err(_) => found.push(format!("{name}/{group}: r2u2.stdout is missing")),
        }
    }
    let mut on_disk: Vec<String> = fs::read_dir(suite.join("outputs"))
        .unwrap()
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    on_disk.sort();
    for group in on_disk {
        if !recorded.contains(&group.as_str()) {
            found.push(format!(
                "{name}/{group}: observation is not in the recorded report"
            ));
        }
    }
    found
}

fn live_copy() -> tempfile::TempDir {
    let copy = tempfile::tempdir().unwrap();
    copy_dir(&Path::new(REPO).join("corpus/r2u2-live-v1"), copy.path());
    copy
}

fn boundary_copy() -> tempfile::TempDir {
    let copy = tempfile::tempdir().unwrap();
    copy_dir(
        &Path::new(tl_syntax::CORPUS_DIR).join("boundary-v1"),
        &copy.path().join("boundary-v1"),
    );
    copy
}

fn mutate_cases(root: &Path, edit: impl FnOnce(&mut Vec<Value>)) {
    let path = root.join("boundary-v1/cases.json");
    let mut doc = read_json(&path);
    edit(doc["cases"].as_array_mut().unwrap());
    fs::write(path, serde_json::to_vec_pretty(&doc).unwrap()).unwrap();
}

fn canonical_root() -> PathBuf {
    PathBuf::from(tl_syntax::CORPUS_DIR)
}

// Trace: TC-099, FR-009-AC-4, NFR-004-AC-2 (partly: see the file header non-claims)
#[test]
fn the_canonical_boundary_corpus_has_no_generated_origin() {
    assert_eq!(
        boundary_substitutions(&canonical_root()),
        Vec::<String>::new()
    );
}

// Trace: TC-099, FR-009-AC-4, NFR-004-AC-2 (partly: see the file header non-claims)
#[test]
fn a_boundary_case_with_generated_provenance_or_non_canonical_shape_is_named() {
    type Edit = fn(&mut Value);
    let substitutions: [(&str, Edit); 6] = [
        ("carries generated provenance", |c| {
            c["provenance"] = json!({
                "found_by": "fuzz target evaluator_differential",
                "found_input": "fuzz/artifacts/evaluator_differential/crash-x",
                "minimised": true
            });
        }),
        ("names a generated input", |c| {
            c["rationale"] = json!("copied from fuzz/corpus/c2po_map/seed-1");
        }),
        ("has no hand-written rationale", |c| {
            c.as_object_mut().unwrap().remove("rationale");
        }),
        ("non-canonical field", |c| {
            c["generatedBy"] = json!("proptest");
        }),
        ("operator label does not match", |c| {
            c["formula"]["op"] = json!("globally");
        }),
        ("shape label does not match", |c| {
            c["formula"]["interval"] = json!([2, 3]);
        }),
    ];
    for (expected, edit) in substitutions {
        let copy = boundary_copy();
        assert_eq!(boundary_substitutions(copy.path()), Vec::<String>::new());
        let mut id = String::new();
        mutate_cases(copy.path(), |cases| {
            let case = cases
                .iter_mut()
                .find(|c| c["id"] == "bc-cl-001")
                .expect("bc-cl-001 exists");
            edit(case);
            id = case["id"].as_str().unwrap().to_owned();
        });
        let found = boundary_substitutions(copy.path());
        assert!(
            found
                .iter()
                .any(|f| f.contains(&id) && f.contains(expected)),
            "substitution `{expected}` was not named: {found:?}"
        );
    }
}

// Trace: TC-099, FR-009-AC-4, NFR-004-AC-2 (partly: see the file header non-claims)
#[test]
fn the_retained_observations_are_recorded_live_runs() {
    let live = Path::new(REPO).join("corpus/r2u2-live-v1");
    for suite in ["past", "future"] {
        assert_eq!(
            observation_substitutions(&live.join(suite), &pinned_revision()),
            Vec::<String>::new()
        );
    }
}

// Trace: TC-099, FR-009-AC-4, NFR-004-AC-2 (partly: see the file header non-claims)
#[test]
fn a_retained_observation_with_generated_provenance_or_non_raw_output_is_named() {
    let pinned = pinned_revision();
    type Edit = fn(&Path);
    let substitutions: [(&str, Edit); 6] = [
        ("not a live run", |dir| {
            let path = dir.join("report.json");
            let mut report = read_json(&path);
            report["mode"] = json!("generated");
            fs::write(path, serde_json::to_vec(&report).unwrap()).unwrap();
        }),
        ("is not the pin", |dir| {
            let path = dir.join("report.json");
            let mut report = read_json(&path);
            report["observed"]["r2u2Revision"] = json!("generated-locally");
            fs::write(path, serde_json::to_vec(&report).unwrap()).unwrap();
        }),
        ("not raw R2U2 output", |dir| {
            let stdout = dir.join("outputs/00-00-00/r2u2.stdout");
            let rows: Vec<Value> = fs::read_to_string(&stdout)
                .unwrap()
                .lines()
                .map(|l| json!({"row": l}))
                .collect();
            fs::write(stdout, serde_json::to_vec(&rows).unwrap()).unwrap();
        }),
        ("not in the recorded report", |dir| {
            copy_dir(
                &dir.join("outputs/00-00-00"),
                &dir.join("outputs/99-generated"),
            );
        }),
        ("r2u2.stdout is missing", |dir| {
            fs::remove_file(dir.join("outputs/00-00-00/r2u2.stdout")).unwrap();
        }),
        ("tool-unavailable detail", |dir| {
            let path = dir.join("report.json");
            let mut report = read_json(&path);
            report["toolDetail"] = json!("synthesised without R2U2");
            fs::write(path, serde_json::to_vec(&report).unwrap()).unwrap();
        }),
    ];
    for (expected, edit) in substitutions {
        let copy = live_copy();
        let suite = copy.path().join("future");
        assert_eq!(
            observation_substitutions(&suite, &pinned),
            Vec::<String>::new()
        );
        edit(&suite);
        let found = observation_substitutions(&suite, &pinned);
        assert!(
            found
                .iter()
                .any(|f| f.contains(expected) && f.starts_with("future")),
            "substitution `{expected}` was not detected: {found:?}"
        );
        if expected == "not raw R2U2 output" || expected == "not in the recorded report" {
            assert!(
                found
                    .iter()
                    .any(|f| f.contains("future/00-00-00") || f.contains("future/99-generated")),
                "the affected fixture was not named: {found:?}"
            );
        }
    }
}
