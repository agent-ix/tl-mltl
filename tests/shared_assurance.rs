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
        6,
        "six proof obligations are declared; {} were attested",
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
///
/// `provenance` is answered for the same reason: it is how the driver observes
/// Quire's version, because `quire provenance` reports the CLI and engine
/// identity as JSON and `quire --version` reports only the CLI. Answering it
/// lets `quire` be shimmed at all, which matters — `quire coverage` is a
/// producer and would otherwise be invisible to this test. That gap was real
/// until an injected `quire coverage` in the driver went undetected here.
fn producer_shims(directory: &Path, names: &[&str]) -> PathBuf {
    // Removed and recreated, not merely topped up. `target/` survives between
    // runs, so a shim written by an earlier version of this test would still be
    // on the shimmed PATH and would silently change what is being measured —
    // which is exactly what happened while this test was being written.
    let _ = fs::remove_dir_all(directory);
    fs::create_dir_all(directory).unwrap();
    let log = directory.join("invocations.log");
    for name in names {
        let path = directory.join(name);
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n\
                 for argument in \"$@\"; do\n\
                 case \"$argument\" in\n\
                 --version|-V) echo \"{name} 9.9.9 (shim)\"; exit 0 ;;\n\
                 provenance) echo '{{\"cli\":{{\"version\":\"9.9.9\"}},\
                 \"engine\":{{\"version\":\"9.9.9\"}}}}'; exit 0 ;;\n\
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
    // Run A replaces every producer — cargo, rustup, rustc, quire, and the
    // external monitor and its compiler — with a stub that logs and fails. The
    // chain must finish, and the log must be empty: not one producer was
    // invoked.
    //
    // `quire` is deliberately NOT in this list, and the reason is worth stating
    // because the obvious reading is that it was forgotten.
    //
    // `quire coverage` is a producer of one of the seven inputs, so a PATH shim
    // looks like the right instrument. It is not: `quoin evidence record`
    // invokes `quire coverage` itself, inside the store it is writing to. That
    // is Quoin using the static exporter, which is exactly what the architecture
    // says Quire is for, and a PATH shim cannot tell it apart from the driver
    // regenerating its own input. Shimming `quire` makes run A fail on a clean
    // tree — measured, not assumed.
    //
    // The property is instead tested directly, and more strongly, by run C
    // below: every declared input is moved aside in turn and the driver is
    // required to refuse rather than recreate it. That covers `quire coverage`
    // and the other six producers by name.
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

    // Run C. Every declared input, one at a time: move it aside, run the driver,
    // and require it to refuse with exit 2 naming the target that writes the
    // file. A driver that can produce its own inputs can produce a green run out
    // of nothing, and this is the direct measurement of that — it covers `quire
    // coverage`, which cannot be PATH-shimmed for the reason given above, and it
    // covers the other five producers by name rather than by absence of evidence.
    let assurance = root().join("target/assurance");
    let inputs = [
        "reference-conformance.jsonl",
        "r2u2-differential.jsonl",
        "cli-conformance.jsonl",
        "test-census.json",
        "quire-static-export.json",
        "msrv.jsonl",
    ];
    for name in inputs {
        let present = assurance.join(name);
        assert!(
            present.is_file(),
            "{name} is absent before the probe even starts; run `make assurance-inputs`"
        );
        let stashed = assurance.join(format!("{name}.stashed-by-test"));
        fs::rename(&present, &stashed).unwrap();
        let revision = head_revision();
        let output = Command::new("python3")
            .args([
                "scripts/assurance_chain.py",
                "--candidate-revision",
                &revision,
            ])
            .current_dir(root())
            .output()
            .expect("failed to run the assurance chain");
        fs::rename(&stashed, &present).unwrap();
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        assert_eq!(
            output.status.code(),
            Some(2),
            "with {name} absent the driver exited {:?}; a driver that carries on \
             without a producer's output is a driver that can report a result nobody \
             produced\n{stderr}",
            output.status.code()
        );
        assert!(
            stderr.contains("make assurance-inputs"),
            "the refusal for an absent {name} did not name the target that writes \
             it: {stderr}"
        );
        assert!(
            present.is_file(),
            "the driver recreated {name} instead of refusing; it is a producer"
        );
    }
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-1
#[test]
fn an_unobservable_tool_version_is_refused_rather_than_defaulted() {
    // A sealed attestation names the version of the tool that produced the bytes.
    // A version nobody measured, filled in with a plausible-looking default, is
    // worse than an absent one: a reader cannot tell it apart from a real
    // observation. The driver raises rather than defaulting, and that raise is
    // on a branch the honest path never takes — measured, not assumed: a
    // mutation inserting `observed = "0.0.0"` before the raise was applied and
    // no gate detected it, because on a working toolchain the branch is dead.
    //
    // So the branch is made live. `rustup` is replaced by a stub that fails for
    // every argument including `--version`, which is how the MSRV proof's cargo
    // version is observed. The chain must refuse with exit 2 and say why.
    let directory = root().join("target/unobservable-version-shim");
    let _ = fs::remove_dir_all(&directory);
    fs::create_dir_all(&directory).unwrap();
    let shim = directory.join("rustup");
    fs::write(&shim, "#!/bin/sh\nexit 1\n").unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&shim, fs::Permissions::from_mode(0o755)).unwrap();
    }

    let inherited = std::env::var("PATH").unwrap_or_default();
    let revision = head_revision();
    let output = Command::new("python3")
        .args([
            "scripts/assurance_chain.py",
            "--candidate-revision",
            &revision,
        ])
        .current_dir(root())
        .env("PATH", format!("{}:{inherited}", directory.display()))
        .output()
        .expect("failed to run the assurance chain");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "the chain did not refuse an unobservable tool version; it exited {:?} \
         and would have sealed an attestation naming a version nobody \
         measured\n{stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("could not be observed"),
        "the refusal did not name the cause: {stderr}"
    );
}

// Trace: TC-019, FR-006-AC-2, NFR-003-AC-2
#[test]
fn the_driver_refuses_to_start_any_child_that_is_not_quoin_or_a_version_probe() {
    // The PATH-shim runs cannot establish this and an adversarial review showed
    // exactly why: `quoin evidence record` runs `quire coverage` itself, so
    // `quire` cannot be shimmed; and a driver that ran `quire coverage` and
    // discarded the output left no shim invocation, no recreated input file and
    // no trace of any kind. The isolation test passed with that injection in.
    //
    // So the boundary is enforced inside the driver by an audit hook, and here
    // it is exercised: a copy of the driver with the review's exact injection
    // must exit 2 and name the argv it refused.
    let report = chain_report();
    let children = report["child_processes"]
        .as_array()
        .expect("child_processes");
    assert!(
        !children.is_empty(),
        "the driver recorded no child processes at all, so this test is measuring \
         nothing: {report:#}"
    );
    for child in children {
        let command = child.as_str().unwrap_or("");
        let permitted = command.starts_with("quoin ")
            || command == "quoin"
            || command.starts_with("quire provenance")
            || command.contains(" --version")
            || command.contains(" -V");
        assert!(
            permitted,
            "the driver started `{command}`, which is neither the pinned Quoin CLI \
             nor a version observation"
        );
    }

    let scratch = root().join("target/execution-boundary-probe");
    let _ = fs::remove_dir_all(&scratch);
    fs::create_dir_all(scratch.join("scripts")).unwrap();
    let driver = fs::read_to_string(root().join("scripts/assurance_chain.py")).unwrap();
    let marker = "def run_chain(candidate_revision: str, workspace: Path) -> dict[str, Any]:\n";
    assert!(driver.contains(marker), "the driver's entry point moved");
    let mutated = driver.replacen(
        marker,
        &format!(
            "{marker}    subprocess.run([\"quire\", \"coverage\", \"--scope\", \".\", \
             \"--json\"], cwd=ROOT, check=False, capture_output=True)\n"
        ),
        1,
    );
    assert_ne!(mutated, driver, "the injection did not apply");
    fs::write(scratch.join("scripts/assurance_chain.py"), &mutated).unwrap();
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
        .expect("failed to run the injected chain");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(2),
        "a driver that ran `quire coverage` was not refused; it exited {:?}\n{stderr}",
        output.status.code()
    );
    assert!(
        stderr.contains("not permitted to start"),
        "the refusal did not name the cause: {stderr}"
    );
    assert!(
        stderr.contains("quire"),
        "the refusal did not name the argv it refused: {stderr}"
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
    // 64 is every row Quire mints from `spec/`: 33 acceptance criteria, 23
    // test-matrix rows and 8 suite-registry rows. Naming the population matters
    // — "matrix rows" would have been wrong, since the test matrix contributes
    // 23 of them. It was 35 + 24 + 9 before FR-006-AC-4, NFR-003-AC-4, TC-021
    // and SUITE-007 were deleted with the retained evidence they measured.
    assert_eq!(
        totals["total"], 64,
        "the declared-row population changed: {totals}. It is 33 acceptance \
         criteria + 23 test-matrix rows + 8 suite-registry rows."
    );
    assert_eq!(
        totals["backed"], 62,
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
    // Each state is bound to the NAMED case that owns it, not merely to the
    // set of state strings the run happened to emit. An adversarial review
    // deleted the only real `suspect` demonstration and relabelled an unrelated
    // probe `suspect`; every gate stayed green, because `states_demonstrated`
    // is built from a free-text label the author types next to the assertion.
    // Requiring a named owner means a relabelled bystander no longer stands in
    // for a demonstration that was removed.
    const REQUIRED: [(&str, &str); 12] = [
        ("pass", "retain-producer-output"),
        ("fail", "attested-failed"),
        ("unavailable", "attested-unavailable"),
        (
            "unsupported",
            "declared-unsupported-case-is-reported-unsupported",
        ),
        ("inconclusive", "declared-unknowns-are-carried-not-dropped"),
        ("not-computed", "attested-not_computed"),
        ("malformed", "malformed-formula-is-reported-as-malformed"),
        ("partial", "receipt-reports-the-absent-human-decision"),
        ("stale", "stale-candidate-binding"),
        ("suspect", "audit-reports-a-suspect-link"),
        ("vacuous", "audit-reports-a-vacuous-run"),
        ("tampered", "refuse-an-edited-receipt"),
    ];

    let report = chain_report();

    // Only MEASURED outcomes count. The chain's `states_demonstrated` is built
    // from cases that ran and matched — never from a free-text label typed next
    // to an assertion, which would let a state stop being demonstrated while
    // this test stayed green. That is the exact failure mode this test exists to
    // rule out.
    let demonstrated: BTreeSet<String> = report["states_demonstrated"]
        .as_array()
        .unwrap()
        .iter()
        .map(|value| value.as_str().unwrap().to_owned())
        .collect();

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

    // And each one is demonstrated by the case that is supposed to demonstrate
    // it. Without this, deleting a demonstration and relabelling any other case
    // with its state keeps the set complete and the gate green.
    let mut owners: BTreeSet<(String, String)> = BTreeSet::new();
    for group in ["scenarios", "adapter_probes"] {
        for item in report[group].as_array().unwrap() {
            if item["matched"] != serde_json::Value::Bool(true) {
                continue;
            }
            let name = item
                .get("scenario")
                .or_else(|| item.get("probe"))
                .and_then(|value| value.as_str())
                .unwrap_or("");
            if let Some(state) = item["state"].as_str() {
                owners.insert((state.to_owned(), name.to_owned()));
            }
        }
    }
    for (state, owner) in REQUIRED {
        assert!(
            owners.contains(&(state.to_owned(), owner.to_owned())),
            "the state `{state}` is not demonstrated by `{owner}`, which is the case \
             that owns it. A state carried by some other case is a label, not a \
             demonstration. Observed owners: {owners:?}"
        );
    }

    // The two states Quoin's audit produces are additionally required to carry
    // the finding that produced them, so the label cannot outlive the finding.
    let probes = report["adapter_probes"].as_array().unwrap();
    for (probe_name, kind) in [
        ("audit-reports-a-suspect-link", "suspect-link"),
        ("audit-reports-a-vacuous-run", "vacuous-evidence"),
    ] {
        let probe = probes
            .iter()
            .find(|item| item["probe"] == probe_name)
            .unwrap_or_else(|| panic!("the probe {probe_name} did not run"));
        let count = probe["detail"][kind].as_u64().unwrap_or(0);
        assert!(
            count > 0,
            "{probe_name} reports no `{kind}` finding, so the state it claims to \
             demonstrate was not produced by anything: {probe:#}"
        );
    }

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
        let name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        // Three exclusions: Git's own store, build output, and the assurance
        // interpreter. Everything a reader could hide in is walked, including
        // `build.rs` and the workflow, because an adversarial review put a
        // reference in `build.rs` — which runs on every cargo invocation — and
        // an earlier census that walked named directories did not see it.
        if matches!(name, ".git" | "target" | ".venv-assurance") {
            continue;
        }
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
fn no_local_evidence_framework_remains() {
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
        // Released for the pre-stable phase by the owner decision recorded in
        // agent-ix/engineering-assurance#7 and deleted under
        // agent-ix/tl-mltl#16. The reader, its fixtures, the two frozen schemas
        // and the retained records themselves go together: a tree that still
        // holds any one of them has not made the deletion it claims to have.
        "evidence",
        "schemas",
        "scripts/legacy_evidence_view.py",
        "tests/fixtures/legacy-compat",
    ] {
        assert!(
            !root.join(removed).exists(),
            "{removed} is still present; the generic evidence machinery was not removed"
        );
    }

    // And nothing may quietly reintroduce a reader for them. The census walks
    // the whole tree — a reference one directory down, in `build.rs`, or in a CI
    // step would otherwise not be caught — and a census this small would be
    // vacuous, so its size is asserted too.
    let mut sources = Vec::new();
    collect_sources(&root, &mut sources);
    let mut inspected = 0;
    for path in &sources {
        inspected += 1;
        let Ok(source) = fs::read_to_string(path) else {
            continue;
        };
        // This test names the deleted machinery on purpose; so does the
        // change-assurance declaration, which records what the release covered.
        let file_name = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("");
        let parent = path
            .parent()
            .and_then(|value| value.file_name())
            .and_then(|value| value.to_str())
            .unwrap_or("");
        if file_name == "shared_assurance.rs"
            || (file_name == "change-assurance.json" && parent == "assurance")
            || parent == "reviews"
            || parent == "tasks"
            || parent.starts_with("PLAN-")
        {
            continue;
        }
        for name in [
            "legacy_evidence_view",
            "tl-mltl-evidence-input-v1.schema.json",
            "tl-mltl-evidence-manifest-v1.schema.json",
        ] {
            assert!(
                !source.contains(name),
                "{} references {name}, which was deleted with the retained evidence",
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

    // And `ci` still names the work it claims to run. Dropping a prerequisite
    // from a composite removes a whole enforcement layer while every remaining
    // check stays green, which is a false green nothing else here would catch.
    //
    // The graph is expanded with `make -n` and read for the COMMANDS, not for
    // target names. `cargo test`, `cargo clippy`, `cargo fmt` and the MSRV
    // toolchain invocation are named specifically: checking only for the
    // producer scripts would prove nothing, because `assurance-inputs` supplies
    // those whether or not `test` is still a prerequisite of `ci`.
    let expansion = Command::new("make")
        .args(["-n", "ci"])
        .current_dir(&root)
        .output()
        .expect("make -n ci failed");
    let graph = String::from_utf8_lossy(&expansion.stdout).into_owned();
    assert!(
        expansion.status.success(),
        "make -n ci did not expand: {}",
        String::from_utf8_lossy(&expansion.stderr)
    );
    for required in [
        "cargo test --all-targets --all-features",
        "cargo clippy --all-targets --all-features",
        "cargo fmt --all -- --check",
        "cargo deny check licenses",
        // Split, because `$(CARGO)` expands to an absolute rustup path when
        // `make` runs under `cargo test` — which is exactly the environment
        // this assertion runs in.
        "rustup run 1.75.0",
        "cargo check --locked --all-targets --all-features",
        "scripts/assurance_chain.py",
        "scripts/check_shared_pins.py",
        "scripts/rust_test_census.py",
        "quire validate",
        "quire coverage",
        "check_unsafe_comments.sh",
        "sha256sum --check",
        "example reference_conformance",
        "example r2u2_differential",
        "example cli_conformance",
        // The build line that keeps `cli_conformance` from replaying a stale
        // binary. The producer cannot check this for itself without becoming a
        // builder, so the graph is what holds it.
        "build --quiet --bin tl-mltl",
        "RUSTDOCFLAGS=-Dwarnings",
    ] {
        assert!(
            graph.contains(required),
            "`make ci` no longer runs `{required}`; a composite that loses a \
             prerequisite loses an enforcement layer while everything else stays \
             green. Expansion was:\n{graph}"
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
