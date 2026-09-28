#![no_main]

#[path = "../support/finite_oracle.rs"]
mod finite_oracle;

use libfuzzer_sys::fuzz_target;

// Trace: TL-230, FR-043-AC-1, FR-046-AC-1. Every byte string becomes a
// bounded, well-formed formula and complete finite trace in one applicable
// profile; the candidate and independent oracle are compared at every cell.
fuzz_target!(|data: &[u8]| {
    if let Err(difference) = finite_oracle::check(data, false) {
        panic!("production/oracle disagreement: {difference:?}");
    }
});
