//! The retained 900 s fuzz baseline record names every fuzz target under at
//! least three distinct seeds with the fields FR-021 requires. This checks the
//! record's shape; it does not rerun any fuzzer and proves nothing about the
//! targets.

use std::{collections::BTreeSet, fs, path::Path};

use serde_json::Value;

// Trace: FR-021-AC-1
#[test]
fn baseline_record_covers_every_target_with_three_distinct_seeds() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let record: Value = serde_json::from_slice(
        &fs::read(root.join("fuzz/evidence/2026-11-08/campaign-900s.json")).unwrap(),
    )
    .unwrap();
    let runs = record["runs"].as_array().unwrap();
    let mut targets: BTreeSet<String> = fs::read_dir(root.join("fuzz/fuzz_targets"))
        .unwrap()
        .map(|entry| {
            let name = entry.unwrap().file_name();
            Path::new(&name)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(targets.len(), 6);
    for target in std::mem::take(&mut targets) {
        let mine: Vec<&Value> = runs.iter().filter(|r| r["target"] == target).collect();
        let seeds: BTreeSet<u64> = mine.iter().map(|r| r["seed"].as_u64().unwrap()).collect();
        assert!(seeds.len() >= 3, "{target}: seeds {seeds:?}");
        for run in mine {
            for field in [
                "requested_seconds",
                "observed_seconds",
                "executions",
                "final_cov_edges",
                "final_features",
                "final_corpus_entries",
                "stop_reason",
            ] {
                assert!(!run[field].is_null(), "{target}: missing {field}");
            }
            assert!(run["executions"].as_u64().unwrap() > 0);
        }
    }
}
