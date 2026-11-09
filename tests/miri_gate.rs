//! TL-233: the Miri undefined-behavior check is a declared, pinned `ci` gate.

use std::fs;

fn makefile() -> String {
    fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Makefile")).unwrap()
}

// Trace: TC-177, NFR-001-AC-5
#[test]
fn miri_is_a_pinned_nightly_gate_in_the_ci_prerequisites() {
    let text = makefile();
    let ci = text
        .lines()
        .skip_while(|line| !line.starts_with("ci:"))
        .take_while(|line| line.starts_with("ci:") || line.starts_with('\t'))
        .collect::<Vec<_>>()
        .join(" ");
    assert!(
        ci.split_whitespace().any(|gate| gate == "miri"),
        "ci prerequisites must include miri: {ci}"
    );
    assert!(text.contains("MIRI_TOOLCHAIN ?= nightly-"));
    assert!(text.contains("cargo miri test --all-features --lib"));
    assert!(text.contains("-Zmiri-disable-isolation"));
    assert!(text.contains("$(CI_GUARD) record miri"));
}
