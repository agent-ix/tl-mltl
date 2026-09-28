use std::{fs, path::Path};

use sha2::{Digest, Sha256};
use tl_mltl::past::HistoryError;
use tl_mltl::wire::OwnerReadErrorCode;

#[path = "../fuzz/target_logic.rs"]
mod target_logic;

fn checked_seeds(directory: &str) -> Vec<(String, Vec<u8>)> {
    let root = Path::new(directory);
    let manifest = fs::read_to_string(root.join("SHA256SUMS")).expect("seed manifest");
    let mut seeds = Vec::new();
    for row in manifest.lines() {
        let (expected, name) = row.split_once("  ").expect("digest row");
        let bytes = fs::read(root.join(name)).expect("seed bytes");
        assert_eq!(format!("{:x}", Sha256::digest(&bytes)), expected, "{name}");
        seeds.push((name.to_owned(), bytes));
    }
    assert_eq!(
        seeds.len(),
        fs::read_dir(root).expect("seed directory").count() - 1,
        "every committed seed must be pinned"
    );
    seeds
}

// Trace: TL-229; FR-046-AC-1
#[test]
fn wire_cli_seeds_drive_the_shared_fuzz_target_logic() {
    let seeds = checked_seeds("fuzz/corpus/wire_cli_decode");
    assert_eq!(seeds.len(), 7);
    for (name, bytes) in seeds {
        let outcome = target_logic::exercise_wire_cli(&bytes);
        assert_ne!(outcome, target_logic::WireOutcome::Oversize, "{name}");
        if name == "malformed" || name.starts_with("unknown-") {
            assert_eq!(outcome, target_logic::WireOutcome::DecodeRefused, "{name}");
        } else if name == "noncanonical-command" {
            assert_eq!(
                outcome,
                target_logic::WireOutcome::OwnerRefused(OwnerReadErrorCode::NonCanonical),
                "{name}"
            );
        } else {
            // Missing trace is a later CLI operation refusal, not a decode refusal.
            assert_eq!(outcome, target_logic::WireOutcome::Admitted, "{name}");
        }
    }
    assert_eq!(
        target_logic::exercise_wire_cli(&vec![b' '; 64 * 1024 + 1]),
        target_logic::WireOutcome::Oversize
    );
}

// Trace: TL-229; FR-046-AC-1
#[test]
fn trace_history_seeds_drive_bytes_constructors_and_resource_refusal() {
    let seeds = checked_seeds("fuzz/corpus/trace_history_intake");
    assert_eq!(seeds.len(), 9);
    for (name, bytes) in seeds {
        let outcome = target_logic::exercise_trace_history(&bytes);
        assert!(!outcome.oversize, "{name}");
        match name.as_str() {
            "valid-trace" => assert!(outcome.trace_admitted),
            "valid-history" => assert!(outcome.history_admitted),
            "structured-event" | "structured-fixed" => {
                assert!(outcome.structured_admitted);
                assert!(outcome.correction_admitted);
            }
            "structured-out-of-order" => assert!(matches!(
                outcome.structured_error,
                Some(HistoryError::OutOfOrder { .. })
            )),
            "structured-duplicate" => assert!(matches!(
                outcome.structured_error,
                Some(HistoryError::DuplicatePosition { .. })
            )),
            "structured-missing-sample" => assert!(matches!(
                outcome.structured_error,
                Some(HistoryError::IncompleteSample { .. })
            )),
            "structured-over-limit" => {
                assert!(outcome.structured_admitted);
                assert!(outcome.resource_refused);
            }
            "malformed-json" => assert!(!outcome.history_admitted),
            _ => panic!("unreviewed seed {name}"),
        }
    }
    assert!(target_logic::exercise_trace_history(&vec![b' '; 64 * 1024 + 1]).oversize);
}
