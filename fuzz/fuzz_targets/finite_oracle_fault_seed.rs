#![no_main]

#[path = "../support/finite_oracle.rs"]
mod finite_oracle;

use libfuzzer_sys::fuzz_target;

// Trace: TL-230, FR-043-AC-2. Deliberate, narrowly scoped evaluator fault
// used only to prove that the differential lane turns a real mismatch red.
fuzz_target!(|data: &[u8]| {
    if let Err(difference) = finite_oracle::check(data, true) {
        panic!("seeded evaluator fault detected: {difference:?}");
    }
});
