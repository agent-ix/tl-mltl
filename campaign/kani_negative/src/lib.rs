//! Verifier-only false claim against the pinned production interval API.
//! The claim is isolated from all production crates and intentionally fails.
//! Trace: FR-047-AC-1, TC-181.

#[cfg(kani)]
#[kani::proof]
fn seeded_false_cardinality_claim() {
    let start: u32 = kani::any();
    let end: u32 = kani::any();
    if let Ok(interval) = tl_syntax::Interval::new(start, end) {
        assert!(interval.cardinality() == Some(1));
    }
}
