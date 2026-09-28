//! Ordinary Rust replay of the fixed Kani seeded-false counterexample.
//! Trace: FR-047-AC-1, TC-181.

#[test]
fn seeded_false_cardinality_counterexample_replays() {
    let start = u32::from_le_bytes([0, 0, 0, 128]);
    let end = u32::from_le_bytes([0, 0, 0, 192]);
    let interval = tl_syntax::Interval::new(start, end).unwrap();
    assert_eq!(interval.cardinality(), Some(1_073_741_825));
    assert_ne!(interval.cardinality(), Some(1));
}
