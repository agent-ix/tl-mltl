//! TL-357: generated inputs never count as conformance, and a generated
//! disagreement enters the canonical population only through the promotion rule
//! documented in `corpus/README.md` ("Promotion of generated counterexamples").
//!
//! Generated inputs are the committed fuzz seeds and any libFuzzer artifact
//! (`fuzz/corpus`, `fuzz/artifacts`) and any proptest persistence file
//! (`proptest-regressions`). A corpus manifest names the canonical population;
//! nothing generated may be named by one, nor be a byte copy of a canonical file.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use serde_json::{json, Value};
use tl_mltl::campaign::sha256_hex;
use tl_oracle::population::CaseResult;

#[path = "support/finite_harness.rs"]
mod harness;
use harness::{formula_from_json, reference, word_from_json, Profile};

const GENERATED_MARKERS: [&str; 3] = ["fuzz/corpus", "fuzz/artifacts", "proptest-regressions"];

fn repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn files_under(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            files_under(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn all_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    files_under(root, &mut out);
    out.sort();
    out
}

/// Every committed or locally produced generated-input file.
fn generated_files() -> Vec<PathBuf> {
    let root = repo();
    let mut out = all_files(&root.join("fuzz/corpus"));
    out.extend(all_files(&root.join("fuzz/artifacts")));
    let mut pending = vec![root.clone()];
    while let Some(dir) = pending.pop() {
        let Ok(entries) = fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            if !path.is_dir() || name == "target" || name == ".git" {
                continue;
            }
            if name == "proptest-regressions" {
                out.extend(all_files(&path));
            } else {
                pending.push(path);
            }
        }
    }
    out
}

/// Every corpus manifest the repository reads as a canonical population.
fn manifests() -> Vec<PathBuf> {
    let mut out = vec![Path::new(tl_syntax::CORPUS_DIR).join("manifest.json")];
    for root in [repo().join("corpus"), repo().join("tests/fixtures")] {
        out.extend(
            all_files(&root)
                .into_iter()
                .filter(|p| p.file_name().is_some_and(|n| n == "manifest.json")),
        );
    }
    out
}

fn strings(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|v| strings(v, out)),
        Value::Object(map) => map.iter().for_each(|(k, v)| {
            out.push(k.clone());
            strings(v, out);
        }),
        _ => {}
    }
}

/// Names in `manifest` that point at generated input.
fn generated_references(manifest: &Value) -> Vec<String> {
    let mut all = Vec::new();
    strings(manifest, &mut all);
    all.into_iter()
        .filter(|s| GENERATED_MARKERS.iter().any(|m| s.contains(m)))
        .collect()
}

// Trace: TC-108, FR-020-AC-5, TC-099, FR-009-AC-4
#[test]
fn no_corpus_manifest_names_a_generated_input() {
    let manifests = manifests();
    assert!(manifests.len() >= 3, "expected the declared manifests");
    for path in manifests {
        let manifest: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            generated_references(&manifest),
            Vec::<String>::new(),
            "{} counts a generated input",
            path.display()
        );
    }
}

// Trace: TC-108, FR-020-AC-5, TC-099, FR-009-AC-4
#[test]
fn no_generated_input_is_a_byte_copy_of_a_canonical_corpus_file() {
    let generated = generated_files();
    assert!(
        generated
            .iter()
            .any(|p| p.starts_with(repo().join("fuzz/corpus"))),
        "the committed fuzz seeds must be found"
    );
    let mut canonical = BTreeSet::new();
    for root in [PathBuf::from(tl_syntax::CORPUS_DIR), repo().join("corpus")] {
        for file in all_files(&root) {
            canonical.insert(sha256_hex(&fs::read(&file).unwrap()));
        }
    }
    assert!(!canonical.is_empty());
    for file in generated {
        let digest = sha256_hex(&fs::read(&file).unwrap());
        assert!(
            !canonical.contains(&digest),
            "{} is a byte copy of a canonical corpus file",
            file.display()
        );
    }
}

// Trace: TC-108, FR-020-AC-5
#[test]
fn the_shared_manifest_total_counts_only_files_inside_the_canonical_root() {
    let root = PathBuf::from(tl_syntax::CORPUS_DIR);
    let manifest: Value =
        serde_json::from_slice(&fs::read(root.join("manifest.json")).unwrap()).unwrap();
    let fixtures = manifest["fixtures"].as_array().unwrap();
    assert!(!fixtures.is_empty());
    let ids: BTreeSet<_> = fixtures.iter().map(|f| f["id"].as_str().unwrap()).collect();
    assert_eq!(ids.len(), fixtures.len(), "fixture ids are unique");
    for fixture in fixtures {
        let named = root.join(fixture["formula"].as_str().unwrap());
        assert!(named.is_file(), "{} must exist", named.display());
        assert!(
            fs::canonicalize(&named)
                .unwrap()
                .starts_with(fs::canonicalize(&root).unwrap()),
            "{} escapes the canonical root",
            named.display()
        );
    }
}

// Trace: TC-108, FR-020-AC-5
#[test]
fn the_detector_flags_a_manifest_that_names_a_generated_input() {
    for name in [
        "fuzz/corpus/c2po_map/seed-1",
        "fuzz/artifacts/lasso_differential/crash-abc",
        "tests/proptest-regressions/property.txt",
    ] {
        let tainted = json!({"fixtures": [{"id": "x", "formula": name}]});
        assert_eq!(generated_references(&tainted), vec![name.to_owned()]);
    }
    let clean = json!({"fixtures": [{"id": "x", "formula": "formulas/a.json"}]});
    assert!(generated_references(&clean).is_empty());
}

/// The promotion rule: a reproducer (profile, formula, trace, position), an
/// expected verdict that `tl-oracle` independently confirms, and provenance
/// naming where the case was found. Returns why a record is refused.
fn check_promotion(record: &Value) -> Result<(), String> {
    for key in ["id", "profile", "formula", "trace", "at", "expected"] {
        if record.get(key).is_none() {
            return Err(format!("reproducer field `{key}` missing"));
        }
    }
    let provenance = record
        .get("provenance")
        .and_then(Value::as_object)
        .ok_or("provenance missing")?;
    for key in ["found_by", "found_input", "minimised"] {
        let ok = match provenance.get(key) {
            Some(Value::String(s)) => !s.trim().is_empty(),
            Some(Value::Bool(true)) => key == "minimised",
            _ => false,
        };
        if !ok {
            return Err(format!("provenance `{key}` missing or empty"));
        }
    }
    let token = record["profile"].as_str().ok_or("profile")?;
    let profile = [Profile::ClosedFuture, Profile::OriginPast]
        .into_iter()
        .find(|p| p.token() == token)
        .ok_or("unknown profile")?;
    let verdict = reference(
        profile,
        &formula_from_json(&record["formula"]),
        &word_from_json(&record["trace"]),
        record["at"].as_u64().ok_or("at")? as usize,
    );
    match (&record["expected"], verdict) {
        (Value::Bool(want), CaseResult::Admitted(got)) if *want == got => Ok(()),
        (want, got) => Err(format!("oracle says {got:?}, record expects {want}")),
    }
}

fn candidate() -> Value {
    json!({
        "id": "promoted-example",
        "profile": Profile::ClosedFuture.token(),
        "formula": {"op": "globally", "interval": [0, 1], "arg": {"op": "prop", "id": 0}},
        "trace": [[0], [0]],
        "at": 0,
        "expected": true,
        "provenance": {
            "found_by": "fuzz target evaluator_differential",
            "found_input": "fuzz/artifacts/evaluator_differential/crash-example",
            "minimised": true
        }
    })
}

// Trace: TC-108, FR-020-AC-5, TC-099, FR-009-AC-4
#[test]
fn promotion_requires_reproducer_oracle_verdict_and_provenance() {
    check_promotion(&candidate()).expect("a complete record is admitted");

    for key in ["formula", "trace", "at", "expected", "provenance"] {
        let mut r = candidate();
        r.as_object_mut().unwrap().remove(key);
        assert!(check_promotion(&r).is_err(), "missing `{key}` was admitted");
    }
    for key in ["found_by", "found_input", "minimised"] {
        let mut r = candidate();
        r["provenance"].as_object_mut().unwrap().remove(key);
        assert!(
            check_promotion(&r).is_err(),
            "provenance without `{key}` was admitted"
        );
    }
    let mut blank = candidate();
    blank["provenance"]["found_by"] = json!("  ");
    assert!(check_promotion(&blank).is_err());

    // The expected verdict must be the oracle's, not the code under test's.
    let mut wrong = candidate();
    wrong["expected"] = json!(false);
    assert!(check_promotion(&wrong).unwrap_err().contains("oracle says"));
}
