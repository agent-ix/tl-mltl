//! Bounded Kani proofs for the wire decode scanners.
//!
//! Compiled only under `cargo kani` (`cfg(kani)`). The scanners proved here are
//! the hand-written lexical stage every owner-wire read runs before `serde_json`
//! sees a byte (`preflight`) and the identity-preimage scanners (`scan_string`,
//! `scan_value`). `serde_json` itself is third-party code and is not modelled.

use super::{preflight, scan_string, scan_value, OwnerLimits};

/// Maximum input length covered by the `preflight` proof. Longer inputs make
/// CBMC fork on every byte class and exhaust memory (measured: 8 bytes did not
/// finish in 15 minutes and exceeded 14 GB).
const PREFLIGHT_BYTES: usize = 4;

/// Maximum input length covered by the identity-scanner proof.
const SCANNER_BYTES: usize = 16;

macro_rules! any_bytes {
    ($name:ident, $max:expr) => {
        let storage: [u8; $max] = kani::any();
        let len: usize = kani::any();
        kani::assume(len <= $max);
        let $name = &storage[..len];
    };
}

/// Proves: for every input of 0..=4 bytes, under both the input and output byte
/// ceilings and a depth ceiling of 1, `preflight` returns `Ok(usage)` or a typed `OwnerReadError` and
/// never panics, overflows, or indexes out of bounds.
///
/// Does not claim: non-ASCII input (UTF-8 validation is standard-library code;
/// an unconstrained-byte variant needed more than 29 GB and did not finish),
/// anything about inputs longer than 4 bytes, about
/// `serde_json` decoding that follows the preflight, or that accepted bytes are
/// a valid document. The cover checks show both the `Ok` and `Err` outcomes are
/// reachable.
#[kani::proof]
#[kani::unwind(6)]
fn preflight_never_panics_on_short_input() {
    any_bytes!(bytes, PREFLIGHT_BYTES);
    // `str::from_utf8` is standard-library code whose loops need a much larger
    // unwind bound and memory than the lexer under test; the proof covers ASCII
    // input and leaves UTF-8 validation to the standard library.
    kani::assume(bytes.iter().all(u8::is_ascii));
    let input: bool = kani::any();
    let limits = OwnerLimits {
        max_depth: 1,
        ..OwnerLimits::owner_max()
    };
    let result = preflight(bytes, limits, input);
    kani::cover!(
        result.is_ok() && !bytes.is_empty(),
        "non-empty accepted input reachable"
    );
    kani::cover!(
        matches!(&result, Err(error) if error.field() == "depth"),
        "depth refusal reachable"
    );
    assert!(result.map_or(true, |usage| usage.wire_bytes == bytes.len()));
}

/// Proves: `scan_string` and `scan_value` over every input of 0..=16 bytes and
/// every start offset inside it return an in-bounds index or a typed encoding
/// error and never panic.
///
/// Does not claim that the returned index delimits a well-formed JSON value.
#[kani::proof]
#[kani::unwind(18)]
fn identity_scanners_never_panic_on_short_input() {
    any_bytes!(bytes, SCANNER_BYTES);
    let start: usize = kani::any();
    kani::assume(start < bytes.len());
    let string_end = scan_string(bytes, start);
    let value_end = scan_value(bytes, start);
    kani::cover!(string_end.is_ok(), "string scan success reachable");
    kani::cover!(value_end.is_ok(), "value scan success reachable");
    assert!(string_end.map_or(true, |end| end < bytes.len()));
    assert!(value_end.map_or(true, |end| end < bytes.len()));
}
