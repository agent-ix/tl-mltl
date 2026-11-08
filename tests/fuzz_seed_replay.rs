//! The wire and history fuzz seeds must reach the accepting and refusing sides of
//! the owner readers, so those targets cannot be vacuous.

use std::{fs, path::Path};

use tl_mltl::past::history;
use tl_mltl::wire::{OwnerLimits, ValidatedCommand, ValidatedTrace};
use tl_mltl::PositionHistoryDocument;

fn seeds(target: &str) -> Vec<(String, Vec<u8>)> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("fuzz/corpus")
        .join(target);
    let mut out: Vec<_> = fs::read_dir(root)
        .unwrap()
        .map(|entry| {
            let path = entry.unwrap().path();
            (
                path.file_name().unwrap().to_string_lossy().into_owned(),
                fs::read(path).unwrap(),
            )
        })
        .collect();
    out.sort();
    out
}

// Trace: TC-090, FR-018-AC-1, FR-018-AC-2
#[test]
fn wire_cli_decode_seeds_reach_owner_readers_and_refusals() {
    let limits = OwnerLimits::owner_max();
    let all = seeds("wire_cli_decode");
    let canonical = all
        .iter()
        .find(|(name, _)| name == "canonical-command")
        .unwrap();
    let trace = all
        .iter()
        .find(|(name, _)| name == "canonical-trace")
        .unwrap();
    let command = ValidatedCommand::from_json_bytes(&canonical.1, limits).unwrap();
    assert_eq!(command.bytes(), canonical.1);
    ValidatedTrace::from_json_bytes(&trace.1, limits).unwrap();
    let refused = all
        .iter()
        .filter(|(_, bytes)| ValidatedCommand::from_json_bytes(bytes, limits).is_err())
        .count();
    assert!(
        refused >= 5,
        "non-canonical or invalid command seeds refused: {refused}"
    );
}

// Trace: TC-090, FR-018-AC-1, FR-018-AC-2
#[test]
fn history_intake_seed_reaches_the_history_owner_boundary() {
    let limits = OwnerLimits::owner_max();
    let all = seeds("history_intake");
    let seed = all
        .iter()
        .find(|(name, _)| name == "valid-event-history")
        .unwrap();
    let document: PositionHistoryDocument = serde_json::from_slice(&seed.1).unwrap();
    let owner = history::derive(&document, limits).unwrap();
    assert_eq!(owner.bytes(), seed.1);
    history::read(owner.bytes(), &document, limits).unwrap();
}
