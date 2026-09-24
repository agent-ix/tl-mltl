#![no_main]

#[allow(
    dead_code,
    reason = "the paired fuzz target uses the other shared entry point"
)]
#[path = "../target_logic.rs"]
mod target_logic;

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    let _ = target_logic::exercise_wire_cli(data);
});
