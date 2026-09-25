#[path = "../support/finite_oracle.rs"]
mod finite_oracle;

use finite_oracle::check;
use std::collections::HashSet;

const SEEDS: &[&[u8]] = &[
    include_bytes!("../corpus/finite_oracle_differential/boolean-nesting"),
    include_bytes!("../corpus/finite_oracle_differential/closed-release-nested"),
    include_bytes!("../corpus/finite_oracle_differential/closed-until-origin"),
    include_bytes!("../corpus/finite_oracle_differential/past-since-origin"),
    include_bytes!("../corpus/finite_oracle_differential/past-triggered-nested"),
];

// Trace: TL-230, FR-043-AC-1, FR-046-AC-1.
#[test]
fn seeded_and_bounded_generated_cases_agree_with_independent_oracle() {
    let mut root_variants = HashSet::new();
    for (index, seed) in SEEDS.iter().enumerate() {
        assert_eq!(check(seed, false), Ok(()), "seed {index}");
        root_variants.insert(std::mem::discriminant(&finite_oracle::root_kind(seed)));
    }
    let mut state = 0x5eed_2301_u64;
    for index in 0..1_000 {
        let mut input = [0_u8; 32];
        for byte in &mut input {
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;
            *byte = state.to_le_bytes()[0];
        }
        assert_eq!(check(&input, false), Ok(()), "generated case {index}");
        root_variants.insert(std::mem::discriminant(&finite_oracle::root_kind(&input)));
    }
    assert_eq!(
        root_variants.len(),
        17,
        "bounded smoke missed an operator family"
    );
}

// Trace: TL-230, FR-043-AC-2, FR-046-AC-2.
#[test]
fn seeded_until_evaluator_fault_is_detected_at_origin() {
    let seed = include_bytes!("../corpus/finite_oracle_differential/closed-until-origin");
    assert_eq!(check(seed, false), Ok(()));
    let mismatch = check(seed, true).expect_err("seeded evaluator fault must disagree");
    assert_eq!(mismatch.position, 0);
    assert_ne!(mismatch.expected, mismatch.observed);
    assert!(matches!(mismatch.root, tl_syntax::NodeKind::Until { .. }));
}
