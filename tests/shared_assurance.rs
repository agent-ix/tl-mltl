//! Tests for the shared assurance intake path (FR-006).
//!
//! These follow this repository's own binding idiom: a `// Trace:` comment above
//! each `#[test]`, which is what Quire's census reads. They invoke the gates
//! rather than reimplementing them, because a test that recomputes what a gate
//! computes is a second implementation that can agree with itself while both are
//! wrong.
//!
//! A missing prerequisite is a failure here, never a skip. A gate that stands
//! down when its dependency is absent reports the same green as one that ran.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use serde_json::Value;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// The interpreter `make assurance-env` builds. Its absence is an error.
fn assurance_python() -> PathBuf {
    let path = std::env::var_os("ASSURANCE_PYTHON")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join(".venv-assurance/bin/python"));
    assert!(
        path.is_file(),
        "the pinned assurance interpreter is missing at {}. Run `make assurance-env`. \
         This is a failure and not a skip: a gate that stands down when its dependency \
         is absent reports the same green as one that ran.",
        path.display()
    );
    path
}

fn run(program: &Path, arguments: &[&str]) -> (i32, String, String) {
    let output = Command::new(program)
        .args(arguments)
        .current_dir(root())
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", program.display()));
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn json_gate(program: &Path, arguments: &[&str]) -> Value {
    let (code, stdout, stderr) = run(program, arguments);
    assert_eq!(code, 0, "{arguments:?} exited {code}\n{stdout}\n{stderr}");
    serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{arguments:?} did not emit JSON: {error}\n{stdout}"))
}

fn head_revision() -> String {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(root())
        .output()
        .expect("git rev-parse failed");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

fn digest_of(path: &Path) -> String {
    let output = Command::new("sha256sum")
        .arg(path)
        .output()
        .expect("sha256sum failed");
    String::from_utf8_lossy(&output.stdout)
        .split_whitespace()
        .next()
        .expect("sha256sum output")
        .to_owned()
}

/// The chain is expensive and several tests read it. It runs once per test
/// binary, and every reader sees the same run rather than a different one.
static CHAIN: OnceLock<Value> = OnceLock::new();

fn chain_report() -> &'static Value {
    CHAIN.get_or_init(|| {
        // The chain runs under the system interpreter: it only shells out to
        // quoin and never imports engineering-assurance.
        let revision = head_revision();
        let (code, stdout, stderr) = run(
            Path::new("python3"),
            &[
                "scripts/assurance_chain.py",
                "--candidate-revision",
                &revision,
                "--json",
            ],
        );
        assert_eq!(code, 0, "the assurance chain exited {code}\n{stderr}");
        serde_json::from_str(&stdout).expect("the assurance chain did not emit JSON")
    })
}

// Trace: TC-018, FR-006-AC-1
#[test]
fn every_shared_pin_is_classified_by_the_packaged_matrix() {
    let python = assurance_python();
    let report = json_gate(&python, &["scripts/check_shared_pins.py", "--json"]);

    let components = report["components"].as_array().expect("components array");
    assert_eq!(
        components.len(),
        4,
        "the matrix pins four components; this run classified {}",
        components.len()
    );
    for component in components {
        assert_eq!(
            component["verdict"], "compatible",
            "{} is {} ({})",
            component["component"], component["verdict"], component["reason"]
        );
    }
    assert_eq!(report["accepted"], true);
    assert!(report["artifact_mismatches"].as_array().unwrap().is_empty());
    assert!(report["mirror_references"].as_array().unwrap().is_empty());
    assert!(
        report["upstream_pin_mismatches"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the tl-syntax pins disagree across the files that name them: {}",
        report["upstream_pin_mismatches"]
    );

    // Acceptance is reported and never gated on: the pinned release records
    // `pending_human_acceptance` and ships no predicate for it
    // (agent-ix/engineering-assurance#20). Reading an absent field as approval,
    // in either direction, is the mistake this asserts against.
    assert_eq!(report["acceptance_recorded_here"], false);
    assert!(report["acceptance_state"].is_string());

    // The mirror check must be seen to refuse. Without this it is indistinguishable
    // from a check that matches nothing.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['engineering_assurance']['requirement']+=' --registry=https://npm.ix/';\
             print(json.dumps(m.mirror_references(pins)))",
        ],
    );
    assert_eq!(code, 0, "the mirror probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        !offenders.is_empty(),
        "a mirror registry reference was not detected; the check matches nothing"
    );

    // The two tl-syntax revisions are two facts. The compiled pin moved onto
    // main; the retained corpus basis did not. Collapsing them is refused, and
    // the refusal is exercised rather than assumed.
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             pins=json.load(open('assurance/pins.json'));\
             pins['upstream_dependency']['corpus_basis']=\
             pins['upstream_dependency']['compiled_revision'];\
             print(json.dumps(m.upstream_pin_mismatches(pins)))",
        ],
    );
    assert_eq!(code, 0, "the collapsed-revision probe failed: {stderr}");
    let problems: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        problems.iter().any(|item| item.contains("same string")),
        "collapsing the compiled revision and the corpus basis was not detected: {problems:?}"
    );
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-1, SUITE-004, SUITE-005, SUITE-006
#[test]
fn the_chain_reaches_quoin_without_quoin_or_quire_executing_a_producer() {
    let report = chain_report();
    assert_eq!(report["matched"], true, "{report:#}");

    for group in ["scenarios", "controls", "adapter_probes"] {
        let items = report[group]
            .as_array()
            .unwrap_or_else(|| panic!("{group}"));
        assert!(!items.is_empty(), "{group} is empty");
        for item in items {
            assert_eq!(
                item["matched"], true,
                "{group} entry did not match: {item:#}"
            );
        }
    }

    // Every attested result is read out of the producer's bytes. Asserting the
    // values here means a chain that reverted to sealing a literal "passed"
    // would still have to agree with what the producers actually wrote — and a
    // chain that attested `inconclusive`, `not_computed` or `unavailable` while
    // still matching every byte-identity scenario would fail here rather than
    // reporting exit 0.
    let attested = report["attested_results"]
        .as_object()
        .expect("attested_results");
    assert_eq!(
        attested.len(),
        7,
        "seven proof obligations are declared; {} were attested",
        attested.len()
    );
    for (proof, result) in attested {
        assert_eq!(result, "passed", "{proof} was attested {result}");
    }

    // The adapter transcribes one named protocol and refuses another, rather than
    // guessing. A verdict recovered from an unrecognised stream is a verdict
    // recovered from nothing.
    let probes = report["adapter_probes"].as_array().unwrap();
    for required in [
        "refuses-a-foreign-protocol",
        "refuses-an-unnamed-outcome",
        "refuses-an-empty-stream",
        "accepts-the-real-run",
    ] {
        assert!(
            probes.iter().any(|probe| probe["probe"] == required),
            "adapter probe {required} is missing"
        );
    }
}

/// Write an executable shim for each name that records every invocation.
///
/// The log is the point. A shim that is never consulted and a producer that is
/// never run look identical from the outside, so the shims write down every call
/// and the test reads the file rather than assuming.
///
/// A version query is answered rather than refused, and deliberately so. Asking
/// a tool its version is an observation — it is what the compatibility matrix's
/// own `observe` column does — and it is not the thing this test forbids. What
/// is forbidden is asking a tool to build, compile, test, evaluate, or replay
/// anything. Every such invocation is logged and the log must be empty.
///
/// `--version` is matched anywhere in the argv, not just in `$1`, because the
/// MSRV attestation observes `rustup run 1.75.0 cargo --version`: its declared
/// command runs cargo through the pinned toolchain, so the version sealed into
/// the attestation has to come from that toolchain rather than from ambient
/// cargo. That is still a version observation. Anything without a version flag
/// — `cargo build`, `cargo run`, `rustup run … check` — is logged and fails the
/// test, which is what keeps it able to fail.
fn producer_shims(directory: &Path, names: &[&str]) -> PathBuf {
    fs::create_dir_all(directory).unwrap();
    let log = directory.join("invocations.log");
    let _ = fs::remove_file(&log);
    for name in names {
        let path = directory.join(name);
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n\
                 for argument in \"$@\"; do\n\
                 case \"$argument\" in\n\
                 --version|-V) echo \"{name} 9.9.9 (shim)\"; exit 0 ;;\n\
                 esac\n\
                 done\n\
                 echo \"$0 $@\" >> {}\n\
                 exit 97\n",
                log.display()
            ),
        )
        .unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
    }
    log
}

fn run_chain_with_path(shims: &Path) -> std::process::Output {
    let inherited = std::env::var("PATH").unwrap_or_default();
    let revision = head_revision();
    Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(root())
        .env("PATH", format!("{}:{inherited}", shims.display()))
        .output()
        .expect("failed to run the assurance chain")
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-2
#[test]
fn the_chain_never_executes_a_producer_and_the_probe_can_prove_it() {
    // Two runs, because one proves nothing.
    //
    // Run A replaces every producer — cargo, rustup, rustc, and the external
    // monitor and its compiler — with a stub that logs and fails. The chain must
    // finish, and the log must be empty: not one producer was invoked.
    //
    // Run B is the control. It stubs `quoin`, which the chain is supposed to run,
    // and requires the chain to fail and the log to be non-empty. Without it, an
    // empty log in run A would be equally consistent with PATH never being
    // consulted at all.
    let producers = root().join("target/producer-shims");
    let producer_log = producer_shims(&producers, &["cargo", "rustup", "rustc", "r2u2", "c2po"]);
    let output = run_chain_with_path(&producers);
    let logged = fs::read_to_string(&producer_log).unwrap_or_default();
    assert!(
        output.status.success(),
        "the assurance chain failed with producers stubbed, which means it ran one:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        logged.trim().is_empty(),
        "the assurance driver asked a producer to do work, not just to name its version:\n{logged}"
    );

    let tools = root().join("target/tool-shims");
    let tool_log = producer_shims(&tools, &["quoin"]);
    let control = run_chain_with_path(&tools);
    let tool_logged = fs::read_to_string(&tool_log).unwrap_or_default();
    assert!(
        !tool_logged.trim().is_empty(),
        "stubbing quoin produced no invocation, so PATH is not being consulted by \
         the subprocess and the run above proves nothing"
    );
    assert!(
        !control.status.success(),
        "the chain succeeded with quoin stubbed out, so it is not actually using it"
    );
}

// Trace: TC-020, FR-006-AC-3, SUITE-003
#[test]
fn the_sealed_records_impact_snapshot_is_the_quire_export() {
    let report = chain_report();
    let export = root().join(report["quire_export"].as_str().expect("quire_export"));
    let bytes =
        fs::read(&export).unwrap_or_else(|error| panic!("{} is absent: {error}", export.display()));

    assert_eq!(
        report["impact_snapshot_digest"],
        digest_of(&export),
        "the sealed record's impact snapshot does not name the Quire export it claims"
    );
    // An empty object has a digest too. The snapshot is only worth its content,
    // so the export is required to actually carry the coverage facts the record
    // claims it snapshotted, and to name every requirement this repository has.
    let parsed: Value = serde_json::from_slice(&bytes).expect("the Quire export is JSON");
    let text = String::from_utf8_lossy(&bytes);
    for requirement in [
        "FR-001", "FR-002", "FR-003", "FR-004", "FR-005", "FR-006", "NFR-001", "NFR-002",
        "NFR-003", "StR-001", "StR-002",
    ] {
        assert!(
            text.contains(requirement),
            "the Quire export does not mention {requirement}; it is not a coverage \
             export of this repository"
        );
    }
    assert!(
        parsed.is_object() && !parsed.as_object().unwrap().is_empty(),
        "the Quire export is not a populated document"
    );

    // The measured coverage, pinned. `derive_result` refuses an export that
    // measured nothing or carries a status lie, but the figures themselves are
    // asserted too: an export reporting different totals has to move a number in
    // this file rather than only a threshold the driver applies.
    let totals = &parsed["totals"];
    assert_eq!(totals["total"], 68, "matrix row count changed: {totals}");
    assert_eq!(
        totals["backed"], 66,
        "backed-row count changed: {totals}. Exactly two rows are unbacked on \
         purpose — SUITE-001 (`make ci`, the composite that contains every other \
         suite) and SUITE-002 (the `quire validate` half of `make spec`, which \
         writes no structured result) — and spec/evidence/suites.md says why. If \
         that number moved, update the registry deliberately rather than \
         adjusting this assertion."
    );
    assert!(
        parsed["status_lies"].as_array().unwrap().is_empty(),
        "Quire reported a row whose declared status disagrees with its evidence: {}",
        parsed["status_lies"]
    );

    // And the chain must have read it as such rather than as a not-computed run.
    assert_eq!(
        report["attested_results"]["PROOF-quire-static-export"], "passed",
        "the Quire export was attested as {}",
        report["attested_results"]["PROOF-quire-static-export"]
    );
}

fn walk(directory: &Path) -> u64 {
    let mut count = 0;
    for entry in fs::read_dir(directory).expect("evidence directory") {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            count += walk(&path);
        } else {
            count += 1;
        }
    }
    count
}

// Trace: TC-021, FR-006-AC-4, NFR-002-AC-2, NFR-003-AC-4, SUITE-007
#[test]
fn retained_evidence_is_read_through_the_shared_mapping_without_moving_a_byte() {
    let python = assurance_python();
    let census = json_gate(&python, &["scripts/legacy_evidence_view.py", "--json"]);

    // Two different claims, kept apart. The first is that this run wrote nothing;
    // the second is that the retained bytes are the bytes that were committed.
    // Only Git can answer the second, and it is asked rather than assumed.
    assert!(census["evidence_bytes_moved_during_this_run"]
        .as_array()
        .unwrap()
        .is_empty());
    assert!(
        census["uncommitted_evidence_changes"]
            .as_array()
            .unwrap()
            .is_empty(),
        "retained evidence differs from what was committed: {}",
        census["uncommitted_evidence_changes"]
    );
    assert!(census["misattributed_records"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(census["matched"], true);

    let files = census["evidence_files_read"].as_u64().unwrap();
    let on_disk = walk(&root().join("evidence"));
    assert_eq!(
        files, on_disk,
        "the compatibility view read {files} evidence files but {on_disk} are present"
    );
    assert_eq!(
        on_disk, 283,
        "this repository retains 283 evidence files; if a byte moved under \
         evidence/ that is the migration's one hard prohibition"
    );

    let retained = &census["retained"];
    assert_eq!(
        retained["count"].as_u64().unwrap(),
        6,
        "this repository retains six evidence records"
    );
    // The honest answer for this repository, measured here rather than inherited.
    // Its retained family is quire.derivation-evidence/v1, which the pinned
    // mapping does not cover, so every envelope is refused. That refusal is
    // reported as it stands and is not converted into a pass. Filed as
    // agent-ix/engineering-assurance#21.
    assert_eq!(
        retained["outcomes"],
        serde_json::json!(["incompatible"]),
        "the retained-evidence outcome changed; if the shared mapping gained a \
         derivation-evidence reader this assertion should be updated deliberately"
    );
    assert_eq!(
        retained["declared_schema_versions"],
        serde_json::json!(["quire.derivation-evidence/v1"])
    );

    // The mapping must be seen to accept, or a refusal proves nothing.
    let accepted = census["accepted_positive_controls"].as_array().unwrap();
    assert!(
        !accepted.is_empty(),
        "no positive control was accepted; a mapping only ever seen refusing is \
         indistinguishable from a step that never worked"
    );

    let (code, stdout, stderr) = run(
        &python,
        &["scripts/legacy_evidence_view.py", "--mutation-probes"],
    );
    assert_eq!(
        code, 0,
        "a load-bearing compatibility check was removed and the census did not \
         notice\n{stdout}\n{stderr}"
    );
}

// Trace: TC-022, FR-006-AC-5, NFR-003-AC-3
#[test]
fn all_twelve_verification_outcomes_are_demonstrated_and_paired_with_controls() {
    // The twelve states this migration must keep distinguishable, and the gate
    // that owns each. A state nobody demonstrates is a state nobody would notice
    // the loss of.
    //
    // `malformed` is owned by the shared temporal corpus: three of its eight
    // fixtures are malformed by design. `unsupported` is owned by the R2U2
    // exchange, whose manifest declares one case outside the adapter's profile.
    // Both are this repository's own domain behaviour rather than a compatibility
    // artefact.
    const REQUIRED: [(&str, &str); 12] = [
        ("pass", "chain"),
        ("fail", "chain"),
        ("unavailable", "chain"),
        ("unsupported", "chain/r2u2-corpus"),
        ("inconclusive", "chain"),
        ("not-computed", "chain"),
        ("malformed", "chain/shared-corpus"),
        ("partial", "chain"),
        ("stale", "chain"),
        ("suspect", "chain"),
        ("vacuous", "chain"),
        ("tampered", "chain"),
    ];

    let python = assurance_python();
    let report = chain_report();
    let census = json_gate(&python, &["scripts/legacy_evidence_view.py", "--json"]);

    // Only MEASURED outcomes count. The chain's `states_demonstrated` is already
    // built from cases that ran and matched. The compatibility lane contributes
    // the outcome the mapping actually returned and the states it actually
    // mapped — never the case's `kind`, which is a free-text label in
    // expectations.json. Counting the label meant a state could stop being
    // demonstrated while this test stayed green, which is the exact failure mode
    // this test exists to rule out.
    let mut demonstrated: BTreeSet<String> = report["states_demonstrated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();
    for case in census["cases"].as_array().unwrap() {
        if case["matched"] != serde_json::Value::Bool(true) {
            continue;
        }
        for state in case["mapped_states"].as_array().unwrap() {
            demonstrated.insert(state.as_str().unwrap().to_owned());
        }
    }

    let missing: Vec<&str> = REQUIRED
        .iter()
        .filter(|(state, _)| !demonstrated.contains(*state))
        .map(|(state, _)| *state)
        .collect();
    assert!(
        missing.is_empty(),
        "these verification outcomes were never demonstrated: {missing:?}; \
         demonstrated: {demonstrated:?}"
    );

    // The compatibility lane's own six-state vocabulary, measured the same way:
    // the census reports which states it observed and which it did not.
    assert!(
        census["undemonstrated_states"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the compatibility mapping's state vocabulary is not fully demonstrated: {}",
        census["undemonstrated_states"]
    );
    assert!(
        census["undemonstrated_outcomes"]
            .as_array()
            .unwrap()
            .is_empty(),
        "the compatibility mapping's outcome vocabulary is not fully demonstrated: {}",
        census["undemonstrated_outcomes"]
    );

    // Every negative names the positive control that proves the step it refuses
    // is a step that works.
    let controls = report["controls"].as_array().unwrap();
    assert!(!controls.is_empty(), "no positive controls were run");
    let negatives: BTreeSet<&str> = controls
        .iter()
        .map(|control| control["pairs_with"].as_str().unwrap())
        .collect();
    for required in [
        "retained-bytes-changed-after-sealing",
        "refuse-an-edited-receipt",
        "stale-candidate-binding",
        "attested-failed",
        "malformed-formula-is-reported-as-malformed",
        "declared-unsupported-case-is-reported-unsupported",
        "differential-comparison-is-not-a-boolean",
    ] {
        assert!(
            negatives.contains(required),
            "the negative {required} has no positive control"
        );
    }
}

// Trace: TC-023, FR-006-AC-6, StR-002-VC-1, StR-002-VC-2, SUITE-006
#[test]
fn the_r2u2_differential_is_a_comparison_and_never_a_boolean() {
    let report = chain_report();

    // The counts come from the two corpus manifests, so a producer that stopped
    // reporting a state cannot also move the number it is checked against.
    let declared_malformed = report["declared_malformed_fixtures"].as_u64().unwrap();
    let declared_unsupported = report["declared_unsupported_cases"].as_u64().unwrap();
    let declared_supported = report["declared_supported_cases"].as_u64().unwrap();
    assert_eq!(
        declared_malformed, 3,
        "the shared corpus manifest declares {declared_malformed} invalid fixtures; if a \
         fixture was added or removed this expectation should move deliberately"
    );
    assert_eq!(
        declared_unsupported, 1,
        "the R2U2 corpus manifest declares {declared_unsupported} unsupported cases"
    );
    assert_eq!(
        declared_supported, 8,
        "the R2U2 corpus manifest declares {declared_supported} comparable cases"
    );
    assert_eq!(
        report["malformed_rows"].as_u64().unwrap(),
        declared_malformed
    );
    assert_eq!(
        report["unsupported_rows"].as_u64().unwrap(),
        declared_unsupported
    );

    // Three comparison classifications and four external-monitor states, all
    // observed. This is the assertion that stops the differential collapsing
    // into a bit: a producer that only ever emitted `agreement` would satisfy
    // every count above and fail here.
    let comparisons: BTreeSet<&str> = report["observed_comparisons"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(
        comparisons,
        BTreeSet::from(["agreement", "mismatch", "non_conclusive"]),
        "the differential reported {comparisons:?}; agreement, mismatch and \
         non-conclusive are three answers and all three have to have been observed"
    );
    let external: BTreeSet<&str> = report["observed_external_states"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap())
        .collect();
    assert_eq!(
        external,
        BTreeSet::from(["conclusive", "pending", "tool_error", "unsupported"]),
        "the differential observed {external:?}; pending, unsupported and tool_error \
         are three different reasons to be non-conclusive and none may be folded away"
    );

    // The facts the chain asserts, each named, so that dropping any one of them
    // is visible here rather than only inside the driver.
    let scenarios = report["scenarios"].as_array().unwrap();
    for required in [
        "malformed-formula-is-reported-as-malformed",
        "malformed-does-not-fail-its-proof",
        "declared-unsupported-case-is-reported-unsupported",
        "differential-comparison-is-not-a-boolean",
        "external-monitor-states-stay-separate",
        "every-supported-case-was-compared",
        "differential-states-survive-into-retained-bytes",
    ] {
        let found = scenarios
            .iter()
            .find(|item| item["scenario"] == required)
            .unwrap_or_else(|| panic!("the scenario {required} did not run"));
        assert_eq!(
            found["matched"], true,
            "{required} did not match: {found:#}"
        );
    }

    // Neither state is a failure: the proofs they belong to are attested `passed`.
    for proof in ["PROOF-reference-conformance", "PROOF-r2u2-differential"] {
        assert_eq!(
            report["attested_results"][proof], "passed",
            "{proof} was attested {}",
            report["attested_results"][proof]
        );
    }

    // And they are not silent passes either: the producer's own rows say
    // `unsupported`, and the adapter carries that word alongside Quoin's
    // three-valued entry outcome rather than discarding it.
    let (code, stdout, stderr) = run(
        Path::new("python3"),
        &[
            "scripts/assurance_chain.py",
            "--adapt",
            "target/assurance/r2u2-differential.jsonl",
        ],
    );
    assert_eq!(code, 0, "the adapter refused the real stream: {stderr}");
    let adapted: Value = serde_json::from_str(&stdout).expect("the adapter emits JSON");
    let carried = adapted["entries"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|entry| entry["domainOutcome"] == "unsupported")
        .count() as u64;
    assert_eq!(
        carried, declared_unsupported,
        "the adapter dropped the unsupported domain outcome; Quoin's entry vocabulary \
         is three-valued, so the twelve-state word has to survive alongside it"
    );
}

// Trace: TC-017, NFR-002-AC-3, SUITE-008
#[test]
fn every_requirement_tagged_test_is_a_test_cargo_compiles_and_runs() {
    let report = json_gate(
        Path::new("python3"),
        &["scripts/rust_test_census.py", "--json"],
    );
    assert_eq!(report["matched"], true, "{report:#}");
    let tagged = report["tagged"].as_array().unwrap();
    let compiled = report["compiled"].as_array().unwrap();
    assert!(
        report["ignored"].as_array().unwrap().is_empty(),
        "a requirement-tagged Rust test is ignored: {}",
        report["ignored"]
    );
    // A census over an empty tagged set compares nothing and passes. The floor
    // is asserted so the gate cannot become vacuous by deletion.
    assert!(
        tagged.len() >= 20,
        "the requirement-tagged test set is unexpectedly small ({}); a census over \
         too few tests asserts almost nothing",
        tagged.len()
    );
    assert_eq!(
        tagged.len(),
        compiled.len(),
        "tagged and compiled test sets differ in size"
    );

    // And the census must be seen to refuse. Without this the comparison is
    // indistinguishable from one that always agrees with itself.
    let (code, stdout, stderr) = run(
        Path::new("python3"),
        &[
            "-c",
            "import sys,json;sys.path.insert(0,'scripts');\
             import rust_test_census as m;\
             print(json.dumps(sorted(m.listed_test_names('a::b: test\\nc::d: test\\n')[0])))",
        ],
    );
    assert_eq!(code, 0, "the census parser probe failed: {stderr}");
    let parsed: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert_eq!(
        parsed,
        vec!["b".to_owned(), "d".to_owned()],
        "the census does not parse cargo's own --list output; it would compare an \
         empty observed set against an empty expected set and agree"
    );
}

/// Collect every readable source file under `directory`, recursively.
fn collect_sources(directory: &Path, into: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries {
        let path = entry.expect("directory entry").path();
        if path.is_dir() {
            collect_sources(&path, into);
            continue;
        }
        let extension = path.extension().and_then(|value| value.to_str());
        if matches!(
            extension,
            Some("py" | "sh" | "rs" | "txt" | "toml" | "yml" | "md" | "json")
        ) {
            into.push(path);
        }
    }
}

// Trace: TC-024, FR-006-AC-7
#[test]
fn no_local_evidence_framework_remains_and_the_frozen_schemas_are_referenced_by_nothing() {
    let root = root();

    // The generic machinery is gone, by name.
    for removed in [
        "scripts/build_evidence_envelope.py",
        "scripts/collect_evidence.sh",
        "scripts/finalize_collection.py",
        "scripts/verify_evidence.sh",
        "scripts/evidence_profile.py",
        "scripts/check_failure_propagation.py",
        "scripts/parameter_identity.py",
        "scripts/run_policy_tests.py",
        "scripts/tool_identity.py",
        "scripts/validate_json_schema.py",
        "scripts/test_evidence_tool.py",
        "scripts/test_failure_propagation.py",
        "scripts/test_tool_identity.py",
        "tools.lock",
        "tests/evidence_contract.rs",
    ] {
        assert!(
            !root.join(removed).exists(),
            "{removed} is still present; the generic evidence machinery was not removed"
        );
    }

    // The two evidence schemas are frozen, not deleted. They are NOT in the same
    // position and schemas/README.md says so: the manifest schema's current
    // bytes are exactly what all six retained envelopes name, and the input
    // schema's are not — the records name two earlier revisions of it. All three
    // digests are pinned so neither the freeze nor the divergence can move
    // silently.
    let frozen = [
        (
            "schemas/tl-mltl-evidence-input-v1.schema.json",
            "7b7e4725bc05d1aafdda7af1586449dbaec6dae2e0893d204acf188347daff24",
        ),
        (
            "schemas/tl-mltl-evidence-manifest-v1.schema.json",
            "8744bfe233f10f2dd6fe3a9d2948d2424802eda0489e4874b79428e6bf73cca1",
        ),
    ];
    for (path, expected) in frozen {
        let file = root.join(path);
        assert!(
            file.is_file(),
            "{path} was deleted; it is frozen, not removed"
        );
        assert_eq!(
            digest_of(&file),
            expected,
            "{path} changed; a frozen schema is immutable"
        );
    }

    // What the six retained envelopes actually name, read out of the immutable
    // bytes rather than restated. The manifest digest has to be the current
    // file; the input digests have to be the two the records carry, and neither
    // may become the current file without a deliberate change here.
    let mut input_digests: BTreeSet<String> = BTreeSet::new();
    let mut output_digests: BTreeSet<String> = BTreeSet::new();
    let mut envelopes = 0;
    for entry in fs::read_dir(root.join("evidence")).expect("evidence directory") {
        let envelope = entry
            .expect("directory entry")
            .path()
            .join("evidence-envelope.json");
        if !envelope.is_file() {
            continue;
        }
        envelopes += 1;
        let value: Value =
            serde_json::from_slice(&fs::read(&envelope).unwrap()).expect("envelope is JSON");
        for (key, into) in [
            ("inputs", &mut input_digests),
            ("outputs", &mut output_digests),
        ] {
            for item in value[key].as_array().into_iter().flatten() {
                if let Some(digest) = item["schema"]["digest"]["value"].as_str() {
                    into.insert(digest.to_owned());
                }
            }
        }
    }
    assert_eq!(envelopes, 6, "six retained envelopes were expected");
    assert_eq!(
        output_digests,
        BTreeSet::from([frozen[1].1.to_owned()]),
        "the retained envelopes no longer name the frozen manifest schema's current bytes"
    );
    assert_eq!(
        input_digests,
        BTreeSet::from([
            "808fd9f33720066e136188722daf0d4ce254fb846fd16f1e9073d1d3175138e2".to_owned(),
            "d763369e194bc9b908b456f6da0f39266720cc4bb77a5102d21c62b51c0b2d3a".to_owned(),
        ]),
        "the input-schema digests the retained envelopes name changed"
    );
    assert!(
        !input_digests.contains(frozen[0].1),
        "the input schema on disk is now one of the digests the records name; that \
         would be good news, but schemas/README.md documents the opposite and has to \
         be corrected deliberately rather than by this assertion quietly passing"
    );

    // Nothing validates against them any more. The census walks recursively and
    // covers the build and workflow files too, because a reintroduced validator
    // one directory down, or a CI step, would otherwise not be caught. A census
    // this small would be vacuous, so its size is asserted as well.
    let mut sources = Vec::new();
    for directory in [
        "scripts",
        "tests",
        "examples",
        "src",
        "spec",
        "assurance",
        ".github",
    ] {
        collect_sources(&root.join(directory), &mut sources);
    }
    for file in [
        "Makefile",
        "Cargo.toml",
        "requirements-assurance.txt",
        "README.md",
    ] {
        let path = root.join(file);
        if path.is_file() {
            sources.push(path);
        }
    }
    let mut inspected = 0;
    for path in &sources {
        inspected += 1;
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        // Three files name the frozen schemas on purpose: this test pins their
        // digests, schemas/README.md documents the freeze, and the
        // change-assurance declaration states the preservation constraint.
        // Everything else must not mention them at all.
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let parent = path
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let permitted = match file_name {
            "shared_assurance.rs" => true,
            "README.md" => parent == "schemas",
            "change-assurance.json" => parent == "assurance",
            _ => false,
        };
        for (schema, _) in frozen {
            let name = Path::new(schema).file_name().unwrap().to_str().unwrap();
            if permitted {
                continue;
            }
            assert!(
                !source.contains(name),
                "{} references the frozen schema {name}; nothing may validate against it",
                path.display()
            );
        }
    }
    assert!(
        inspected > 30,
        "the source census is unexpectedly small ({inspected}) to make this claim"
    );

    // The Makefile is orchestration, not a trust root, and carries no gate that
    // polices its own execution.
    let makefile = fs::read_to_string(root.join("Makefile")).unwrap();
    for gone in [
        "check-failure-propagation",
        "check-tool-identities",
        "ci-for-evidence",
        "verify-evidence",
        "evidence-tool",
    ] {
        assert!(
            !makefile.contains(gone),
            "the Makefile still carries the {gone} self-attestation target"
        );
    }
}

// Trace: TC-022, FR-006-AC-5, NFR-003-AC-3
#[test]
fn a_control_naming_a_scenario_that_does_not_exist_is_refused() {
    // NFR-003-AC-3 claims this guard is checked. The driver has it; nothing
    // exercised it, which is the same shape of gap the guard itself is there to
    // catch.
    //
    // The driver is copied and one `pairs_with` — and only that one — is
    // renamed. Renaming the scenario as well would leave the pairing consistent
    // and prove nothing.
    let scratch = root().join("target/dangling-probe");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    let driver = fs::read_to_string(root().join("scripts/assurance_chain.py")).unwrap();

    let control_marker =
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt\",";
    assert!(
        driver.contains(control_marker),
        "the control this probe renames is no longer present in the driver"
    );
    let mutated = driver.replacen(
        control_marker,
        "        \"verify-accepts-an-unedited-receipt\",\n        \"refuse-an-edited-receipt-typo\",",
        1,
    );
    assert_ne!(mutated, driver, "the mutation did not apply");
    fs::write(scratch.join("scripts/assurance_chain.py"), &mutated).unwrap();

    // Everything else the driver reads comes from the real tree. Every root
    // entry except `scripts` is symlinked, rather than an enumerated list, so
    // that a driver which starts reading a new directory does not turn this
    // probe into one that fails for an unrelated reason.
    for entry in fs::read_dir(root()).expect("repository root") {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|v| v.to_str())
            .unwrap_or("")
            .to_owned();
        if name == "scripts" || name == ".git" {
            continue;
        }
        let _ = std::os::unix::fs::symlink(&path, scratch.join(&name));
    }

    let revision = head_revision();
    let output = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(&scratch)
        .output()
        .expect("failed to run the mutated chain");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a control naming a non-existent scenario was not refused\n{stderr}"
    );
    assert!(
        stderr.contains("name a scenario that does not exist"),
        "the refusal did not name the cause: {stderr}"
    );
}

// Trace: TC-018, FR-006-AC-1, SUITE-009
#[test]
fn the_mirror_scan_refuses_a_registry_reference_in_a_real_file() {
    // The structural branch of `mirror_references` (pins.json) already has a
    // control. The file-scan branch did not: it was never observed to fire, so
    // it was indistinguishable from a loop over files that never match.
    let python = assurance_python();
    let (code, stdout, stderr) = run(
        &python,
        &[
            "-c",
            "import json,sys,pathlib;sys.path.insert(0,'scripts');\
             import check_shared_pins as m;\
             original=pathlib.Path('requirements-assurance.txt').read_text();\
             pathlib.Path('requirements-assurance.txt').write_text(\
             original+'\\n--registry=https://npm.ix/\\n');\
             pins=json.load(open('assurance/pins.json'));\
             found=m.mirror_references(pins);\
             pathlib.Path('requirements-assurance.txt').write_text(original);\
             print(json.dumps(found))",
        ],
    );
    assert_eq!(code, 0, "the mirror file-scan probe failed: {stderr}");
    let offenders: Vec<String> = serde_json::from_str(stdout.trim()).unwrap();
    assert!(
        offenders
            .iter()
            .any(|entry| entry.starts_with("requirements-assurance.txt:")),
        "a mirror reference written into a scanned FILE was not detected; the \
         file-scan branch matches nothing. Detected: {offenders:?}"
    );

    // And the file must be restored, or this test has dirtied the tree.
    let restored = fs::read_to_string(root().join("requirements-assurance.txt")).unwrap();
    assert!(
        !restored.contains("npm.ix/"),
        "the probe left a mirror reference in requirements-assurance.txt"
    );
}
