#[path = "../support/finite_oracle.rs"]
mod finite_oracle;

use finite_oracle::check;
use sha2::{Digest, Sha256};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

const SEEDS: &[(&str, &[u8])] = &[
    (
        "boolean-nesting",
        include_bytes!("../corpus/finite_oracle_differential/boolean-nesting"),
    ),
    (
        "closed-release-nested",
        include_bytes!("../corpus/finite_oracle_differential/closed-release-nested"),
    ),
    (
        "closed-until-origin",
        include_bytes!("../corpus/finite_oracle_differential/closed-until-origin"),
    ),
    (
        "past-since-origin",
        include_bytes!("../corpus/finite_oracle_differential/past-since-origin"),
    ),
    (
        "past-triggered-nested",
        include_bytes!("../corpus/finite_oracle_differential/past-triggered-nested"),
    ),
];

fn validate_seed_manifest() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("corpus/finite_oracle_differential");
    let manifest = fs::read_to_string(root.join("SHA256SUMS")).expect("seed manifest");
    let mut declared = HashMap::new();
    for row in manifest.lines() {
        let (digest, name) = row.split_once("  ").expect("digest row");
        assert!(
            declared.insert(name, digest).is_none(),
            "duplicate seed {name}"
        );
    }
    assert_eq!(declared.len(), SEEDS.len());
    assert_eq!(fs::read_dir(&root).unwrap().count(), SEEDS.len() + 1);
    for (name, bytes) in SEEDS {
        assert_eq!(fs::read(root.join(name)).unwrap(), *bytes, "{name}");
        assert_eq!(
            declared.remove(name).expect("seed declared"),
            format!("{:x}", Sha256::digest(bytes)),
            "{name}"
        );
    }
    assert!(declared.is_empty(), "unmatched seed declaration");
}

// Trace: TL-230, FR-043-AC-1, FR-046-AC-1.
#[test]
fn seeded_and_bounded_generated_cases_agree_with_independent_oracle() {
    validate_seed_manifest();
    let mut root_variants = HashSet::new();
    for (index, (_, seed)) in SEEDS.iter().enumerate() {
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
