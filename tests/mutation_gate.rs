//! TL-371: behavior of the mutation gate this crate runs (`tl-gate gate`, built by `tools/gate`) on
//! synthetic cargo-mutants outcomes, and the configuration that decides what is mutated. The gate's
//! own logic lives in tl-syntax; these tests pin the behavior this crate's FR-022 depends on.
//! The mutation configuration skips this file because it rebuilds the gate and tests no crate code.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

struct Run {
    status: i32,
    stdout: String,
}

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tl-mltl-mutation-gate-{}-{name}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn outcome(file: &str, summary: &str, name: &str) -> String {
    format!(
        r#"{{"scenario":{{"Mutant":{{"file":"{file}","name":"{file}:1:1: {name}"}}}},"summary":"{summary}"}}"#
    )
}

fn write_outcomes(dir: &Path, baseline: &str, outcomes: &[String]) {
    let first = format!(r#"{{"scenario":"Baseline","summary":"{baseline}"}}"#);
    let all = std::iter::once(first)
        .chain(outcomes.iter().cloned())
        .collect::<Vec<_>>()
        .join(",");
    fs::write(
        dir.join("outcomes.json"),
        format!(r#"{{"outcomes":[{all}]}}"#),
    )
    .unwrap();
}

fn thresholds(dir: &Path, core: f64, crate_floor: f64, survivors: &str) -> PathBuf {
    let path = dir.join("thresholds.toml");
    fs::write(
        &path,
        format!(
            "diff_min_kill_percent = 100.0\nrequire_survivor_reasons = true\n\
             exclude = [\"src/ci_guard.rs\"]\n\
             [[group]]\nname = \"evaluator-core\"\nglobs = [\"src/future/*.rs\"]\nmin_kill_percent = {core}\n\
             [[group]]\nname = \"crate\"\nglobs = [\"src/*.rs\", \"src/*/*.rs\"]\nmin_kill_percent = {crate_floor}\n\
             {survivors}"
        ),
    )
    .unwrap();
    path
}

fn gate(dir: &Path, thresholds: &Path) -> Run {
    let output = Command::new(env!("CARGO"))
        .args([
            "run",
            "--quiet",
            "--manifest-path",
            "tools/gate/Cargo.toml",
            "--",
            "gate",
        ])
        .arg(dir)
        .arg(thresholds)
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    Run {
        status: output.status.code().unwrap(),
        stdout: String::from_utf8(output.stdout).unwrap(),
    }
}

const LISTED: &str = "[[survivor]]\nfile = \"src/lib.rs\"\nreason = \"equivalent\"\nnames = [\"replace a with b in f\"]\n";

fn passing_population() -> Vec<String> {
    vec![
        outcome("src/future/eval.rs", "CaughtMutant", "m1"),
        outcome("src/lib.rs", "CaughtMutant", "m2"),
        outcome("src/lib.rs", "CaughtMutant", "m3"),
    ]
}

// Trace: TC-113, FR-022-AC-1, FR-022
#[test]
fn the_campaign_mutates_the_core_and_the_crate_and_excludes_the_plumbing_by_configuration() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let thresholds = fs::read_to_string(root.join("mutation-thresholds.toml")).unwrap();
    assert!(thresholds.contains("name = \"evaluator-core\""));
    assert!(thresholds.contains("globs = [\"src/future/*.rs\"]"));
    assert!(thresholds.contains("name = \"crate\""));
    for plumbing in [
        "src/ci_guard.rs",
        "src/bin/ci_guard.rs",
        "src/bin/kani_gate.rs",
    ] {
        assert!(thresholds.contains(plumbing), "{plumbing} is not excluded");
    }
    let mutants = fs::read_to_string(root.join(".cargo/mutants.toml")).unwrap();
    assert!(mutants.contains("exclude_globs"));
    assert!(mutants.contains("ci_guard"));
}

// Trace: TC-114, FR-022-AC-2, NFR-005-AC-1
#[test]
fn mutation_gate_refuses_a_failing_baseline() {
    let dir = scratch("baseline");
    write_outcomes(&dir, "Failure", &passing_population());
    let toml = thresholds(&dir, 0.0, 0.0, "");
    let run = gate(&dir, &toml);
    assert_eq!(run.status, 1, "{}", run.stdout);
    assert!(run.stdout.contains("FAIL baseline"), "{}", run.stdout);
    write_outcomes(&dir, "Success", &passing_population());
    assert_eq!(gate(&dir, &toml).status, 0);
}

// Trace: TC-115, FR-022-AC-3
#[test]
fn mutation_gate_reports_each_outcome_class_and_fails_an_unknown_one() {
    let dir = scratch("classes");
    write_outcomes(
        &dir,
        "Success",
        &[
            outcome("src/future/eval.rs", "CaughtMutant", "core"),
            outcome("src/lib.rs", "CaughtMutant", "a"),
            outcome("src/lib.rs", "MissedMutant", "replace a with b in f"),
            outcome("src/lib.rs", "Timeout", "b"),
            outcome("src/lib.rs", "Unviable", "c"),
        ],
    );
    let toml = thresholds(
        &dir,
        0.0,
        0.0,
        &format!(
            "{LISTED}[[survivor]]\nfile = \"src/lib.rs\"\nreason = \"slow\"\nnames = [\"b\"]\n"
        ),
    );
    let run = gate(&dir, &toml);
    assert_eq!(run.status, 0, "{}", run.stdout);
    for part in ["caught=2", "missed=1", "timeout=1", "unviable=1"] {
        assert!(
            run.stdout.contains(part),
            "{part} missing from {}",
            run.stdout
        );
    }
    write_outcomes(&dir, "Success", &[outcome("src/lib.rs", "ToolError", "d")]);
    assert_eq!(gate(&dir, &toml).status, 1);
}

// Trace: TC-116, FR-022-AC-3, FR-022-AC-4
#[test]
fn mutation_gate_computes_the_kill_rate_and_enforces_a_floor() {
    let dir = scratch("rate");
    write_outcomes(
        &dir,
        "Success",
        &[
            outcome("src/future/eval.rs", "CaughtMutant", "core"),
            outcome("src/lib.rs", "CaughtMutant", "a"),
            outcome("src/lib.rs", "CaughtMutant", "a2"),
            outcome("src/lib.rs", "MissedMutant", "replace a with b in f"),
            outcome("src/lib.rs", "Unviable", "u"),
        ],
    );
    // 3 / (3 + 1 + 0) = 75%; the unviable mutant counts on neither side.
    let met = thresholds(&dir, 0.0, 75.0, LISTED);
    let run = gate(&dir, &met);
    assert_eq!(run.status, 0, "{}", run.stdout);
    assert!(run.stdout.contains("75.0"), "{}", run.stdout);
    let above = thresholds(&dir, 0.0, 75.1, LISTED);
    assert_eq!(gate(&dir, &above).status, 1);
}

// Trace: TC-117, FR-022-AC-5
#[test]
fn mutation_gate_requires_a_reason_for_each_survivor() {
    let dir = scratch("reason");
    write_outcomes(
        &dir,
        "Success",
        &[
            outcome("src/future/eval.rs", "CaughtMutant", "core"),
            outcome("src/lib.rs", "CaughtMutant", "a"),
            outcome("src/lib.rs", "MissedMutant", "replace a with b in f"),
        ],
    );
    assert_eq!(gate(&dir, &thresholds(&dir, 0.0, 0.0, "")).status, 1);
    assert_eq!(gate(&dir, &thresholds(&dir, 0.0, 0.0, LISTED)).status, 0);
    let blank = LISTED.replace("equivalent", " ");
    assert_eq!(gate(&dir, &thresholds(&dir, 0.0, 0.0, &blank)).status, 1);
}

// Trace: TC-118, FR-022-AC-4, NFR-005-AC-2
#[test]
fn mutation_gate_fails_a_weakened_test_selection() {
    let dir = scratch("weakened");
    let toml = thresholds(&dir, 100.0, 90.0, "");
    write_outcomes(&dir, "Success", &passing_population());
    assert_eq!(gate(&dir, &toml).status, 0);
    // The same mutants with the killing test removed: the core mutant now survives.
    let mut weakened = passing_population();
    weakened[0] = outcome("src/future/eval.rs", "MissedMutant", "m1");
    write_outcomes(&dir, "Success", &weakened);
    let run = gate(&dir, &toml);
    assert_eq!(run.status, 1, "{}", run.stdout);
    assert!(
        run.stdout
            .contains("FAIL survivor without a reason: src/future/eval.rs: m1"),
        "{}",
        run.stdout
    );
    assert!(
        run.stdout
            .contains("FAIL evaluator-core: kill 0.00% (min 100.00%)"),
        "{}",
        run.stdout
    );
}
