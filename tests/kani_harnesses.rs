//! TL-372: the Kani harness list is checked by exact name, every harness states
//! its bounds and non-claims, and verifier-only code adds no production
//! dependency (FR-023-AC-1, FR-023-AC-4).
//!
//! The harnesses themselves run only under `cargo kani`; this suite checks the
//! source and the list the `kani-check` gate iterates, so a renamed, removed or
//! unlisted harness fails `cargo test` and `make kani-check` before any proof
//! starts.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const ROOT: &str = env!("CARGO_MANIFEST_DIR");

/// Source files that carry `kani::proof` harnesses.
const PROOF_SOURCES: [(&str, &str); 3] = [
    ("src/future/horizon.rs", "future::horizon::kani_proofs"),
    (
        "src/future/kani_evaluate.rs",
        "future::evaluate::kani_proofs",
    ),
    ("src/wire/kani_wire.rs", "wire::common::kani_proofs"),
];

fn read(relative: &str) -> String {
    let path = Path::new(ROOT).join(relative);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {relative} ({}): {error}", path.display()))
}

/// The harness names and their doc comments found in `source`.
fn harnesses(source: &str) -> Vec<(String, String)> {
    let lines: Vec<&str> = source.lines().collect();
    let mut found = Vec::new();
    for (index, line) in lines.iter().enumerate() {
        if line.trim() != "#[kani::proof]" {
            continue;
        }
        let name = lines[index + 1..]
            .iter()
            .find_map(|next| {
                let next = next.trim();
                next.strip_prefix("fn ")
                    .and_then(|rest| rest.split('(').next())
                    .map(str::to_string)
            })
            .expect("a harness function follows #[kani::proof]");
        let doc: String = lines[..index]
            .iter()
            .rev()
            .map(|above| above.trim())
            .take_while(|above| {
                above.starts_with("///") || above.starts_with("#[") || above.starts_with("//")
            })
            .collect::<Vec<_>>()
            .join("\n");
        found.push((name, doc));
    }
    found
}

fn listed() -> Vec<String> {
    read("scripts/kani-harnesses.txt")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect()
}

/// Names in exactly one of the two sets: the list and the source disagree.
fn mismatch(listed: &[String], found: &[String]) -> Vec<String> {
    let listed: BTreeSet<&String> = listed.iter().collect();
    let found: BTreeSet<&String> = found.iter().collect();
    listed
        .symmetric_difference(&found)
        .map(|name| (*name).clone())
        .collect()
}

fn all_found() -> Vec<(String, String)> {
    PROOF_SOURCES
        .iter()
        .flat_map(|(path, module)| {
            harnesses(&read(path))
                .into_iter()
                .map(move |(name, doc)| (format!("{module}::{name}"), doc))
        })
        .collect()
}

// Trace: TC-120, FR-023-AC-1
#[test]
fn listed_harnesses_match_the_source_by_exact_name() {
    let found: Vec<String> = all_found().into_iter().map(|(name, _)| name).collect();
    let unique: BTreeSet<&String> = found.iter().collect();
    assert_eq!(unique.len(), found.len(), "harness names must be unique");
    let list = listed();
    assert_eq!(
        mismatch(&list, &found),
        Vec::<String>::new(),
        "scripts/kani-harnesses.txt and the #[kani::proof] functions disagree"
    );
}

// Trace: TC-120, FR-023-AC-1
#[test]
fn name_check_fails_for_a_renamed_or_missing_harness() {
    let found: Vec<String> = all_found().into_iter().map(|(name, _)| name).collect();
    let mut renamed = found.clone();
    renamed[0].push_str("_renamed");
    assert_eq!(mismatch(&found, &renamed).len(), 2);
    let missing = &found[1..];
    assert_eq!(mismatch(&found, missing), vec![found[0].clone()]);
    assert!(mismatch(&found, &found).is_empty());
}

// Trace: TC-120, FR-023-AC-1
#[test]
fn every_harness_states_its_bounds_and_non_claims() {
    for (name, doc) in all_found() {
        assert!(doc.contains("Proves"), "{name}: doc lacks 'Proves'");
        assert!(
            doc.contains("Does not claim") || doc.contains("Does not prove"),
            "{name}: doc lacks a non-claim"
        );
    }
}

// Trace: TC-120, FR-023-AC-1
#[test]
fn combinator_harness_name_and_doc_say_the_evaluator_entry_is_stubbed() {
    let (name, doc) = all_found()
        .into_iter()
        .find(|(name, _)| name.contains("::evaluator_combinators"))
        .unwrap();
    assert!(name.contains("stubbed_at"), "{name} must name the stub");
    assert!(doc.contains("no-panic") && doc.contains("typed-error"));
}

// Partly backs FR-023-AC-2: the gate reports one generic failure; it does not
// give unwinding failure, timeout or unsupported construct separate statuses.
// Trace: TC-120, FR-023-AC-2
#[test]
fn gate_runs_each_listed_harness_exactly_and_every_proof_is_covered() {
    let makefile = read("Makefile");
    assert!(makefile.contains("scripts/kani-harnesses.txt"));
    assert!(makefile.contains("scripts/kani-gate.sh"));
    let gate = read("scripts/kani-gate.sh");
    assert!(gate.contains("--harness") && gate.contains("--exact"));
    // Kani exits 0 on an unsatisfied cover; the gate must read the result.
    assert!(gate.contains("cover properties satisfied"));
    assert!(gate.contains("VERIFICATION:- SUCCESSFUL"));
    // Every harness reaches a reachability check: in its own body or in a
    // helper function of the same file it calls.
    for (path, _) in PROOF_SOURCES {
        let source = read(path);
        for (name, _) in harnesses(&source) {
            let body = fn_body(&source, &name);
            let covered = body.contains("kani::cover!")
                || source
                    .split("\nfn ")
                    .skip(1)
                    .filter(|helper| {
                        let helper_name = helper.split('(').next().unwrap();
                        body.contains(&format!("{helper_name}("))
                    })
                    .any(|helper| helper.contains("kani::cover!"));
            assert!(covered, "{path}: harness {name} has no cover check");
        }
    }
}

/// The text from `fn <name>(` up to the next top-level item.
fn fn_body(source: &str, name: &str) -> String {
    let start = source.find(&format!("fn {name}(")).unwrap();
    let rest = &source[start..];
    let end = rest.find("\n}\n").map_or(rest.len(), |end| end + 3);
    rest[..end].to_string()
}

// Partly backs FR-023-AC-4: it checks the manifest, lockfile and cfg(kani)
// confinement only; MSRV divergence detection and Java, Node or Electron
// dependencies are not checked here.
// Trace: TC-123, FR-023-AC-4
#[test]
fn no_verifier_dependency_in_the_manifest_and_cfg_kani_is_confined() {
    let manifest = read("Cargo.toml");
    assert!(
        !manifest.to_ascii_lowercase().contains("kani"),
        "Cargo.toml must not mention kani"
    );
    let lock = read("Cargo.lock").to_ascii_lowercase();
    for verifier in ["kani", "loom", "verus", "crux"] {
        assert!(
            !lock.contains(&format!("name = \"{verifier}")),
            "{verifier} in Cargo.lock"
        );
    }
    let allowed: BTreeSet<&str> = [
        "src/future/horizon.rs",
        "src/future/evaluate.rs",
        "src/wire/common.rs",
    ]
    .into_iter()
    .collect();
    for entry in walk(&Path::new(ROOT).join("src")) {
        let relative = entry.strip_prefix(ROOT).unwrap().trim_start_matches('/');
        let text = fs::read_to_string(&entry).unwrap();
        if mentions_kani_cfg(&text)
            && !relative.ends_with("ci_guard.rs")
            && !relative.contains("/kani_")
        {
            assert!(
                allowed.contains(relative),
                "unexpected cfg(kani) in {relative}"
            );
        }
    }
}

fn walk(dir: &Path) -> Vec<String> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            out.extend(walk(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path.to_string_lossy().into_owned());
        }
    }
    out
}

/// True when a `cfg` or `cfg_attr` attribute in `text` names `kani` as a token,
/// in any form: `cfg(kani)`, `cfg(all(kani, ..))`, `cfg(any(.., kani))`,
/// `cfg(not(kani))` or `cfg_attr(kani, ..)`.
fn mentions_kani_cfg(text: &str) -> bool {
    let mut rest = text;
    while let Some(start) = rest.find('#') {
        let attribute = &rest[start..];
        if !(attribute.starts_with("#[") || attribute.starts_with("#![")) {
            rest = &attribute[1..];
            continue;
        }
        let end = attribute.find(']').map_or(attribute.len(), |end| end + 1);
        let body = &attribute[..end];
        let names: Vec<&str> = body
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|token| !token.is_empty())
            .collect();
        let is_cfg = names.iter().any(|t| *t == "cfg" || *t == "cfg_attr");
        if is_cfg && names.contains(&"kani") {
            return true;
        }
        rest = &attribute[1..];
    }
    false
}

// Trace: TC-123, FR-023-AC-4
#[test]
fn kani_cfg_detection_catches_every_attribute_form() {
    for form in [
        "#[cfg(kani)]",
        "#![cfg(kani)]",
        "#[cfg(all(kani, test))]",
        "#[cfg(any(test, kani))]",
        "#[cfg(not(kani))]",
        "#[cfg_attr(kani, derive(Debug))]",
        "#[cfg(all(not(miri),\n  kani))]",
    ] {
        assert!(mentions_kani_cfg(form), "missed {form}");
    }
    for form in [
        "#[cfg(test)]",
        "#[derive(Debug)]\nstruct Kanidog;",
        "// cfg(kani) in a comment is not an attribute",
        "let kani = 1;",
    ] {
        assert!(!mentions_kani_cfg(form), "false hit on {form}");
    }
}

// Trace: TC-120, FR-023-AC-1
#[test]
#[should_panic(expected = "cannot read src/no/such/file.rs")]
fn unreadable_proof_source_path_fails_with_a_clear_message() {
    read("src/no/such/file.rs");
}
