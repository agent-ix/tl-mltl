//! TL-223: every committed seed of the two infinite-trace fuzz targets runs
//! through the target body without cargo-fuzz, so the seeds and the property
//! checks stay executable in `cargo test`. The campaign itself is recorded in
//! `spec/reviews/SR-223-infinite-fuzz-and-mutation-pilot.md`.

#![cfg(feature = "infinite-trace")]

#[path = "support/lasso_kit.rs"]
mod kit;
#[path = "../fuzz/support/infinite.rs"]
mod support;

use std::fs;
use std::path::Path;

fn seeds(target: &str) -> Vec<Vec<u8>> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz/corpus")
        .join(target);
    let mut names: Vec<_> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("{}: {e}", dir.display()))
        .map(|entry| entry.unwrap().path())
        .collect();
    names.sort();
    names.iter().map(|path| fs::read(path).unwrap()).collect()
}

// Trace: TC-141, TC-143, FR-030-AC-1, FR-030-AC-2
#[test]
fn lasso_differential_seeds_agree_with_both_references() {
    let all = seeds("lasso_differential");
    assert_eq!(all.len(), 12);
    for seed in &all {
        support::lasso_differential(seed);
    }
}

// Trace: TC-143, FR-030-AC-2
#[test]
fn partial_valuation_monotone_seeds_keep_conclusive_verdicts() {
    let all = seeds("partial_valuation_monotone");
    assert_eq!(all.len(), 12);
    for seed in &all {
        support::partial_valuation_monotone(seed);
    }
}

#[test]
fn the_decoder_is_total_and_deterministic() {
    for len in [0usize, 1, 7, 64] {
        let data: Vec<u8> = (0..len).map(|i| (i * 37 + 11) as u8).collect();
        support::lasso_differential(&data);
        support::partial_valuation_monotone(&data);
    }
}
