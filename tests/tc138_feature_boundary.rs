//! External-consumer boundary for the non-default infinite provider.

use std::{collections::BTreeSet, fs, path::Path, process::Command};

use serde_json::Value;
use sha2::{Digest, Sha256};

fn cargo(manifest: &Path, args: &[&str]) -> std::process::Output {
    let output = Command::new("cargo")
        .env("CARGO_NET_OFFLINE", "true")
        .arg(args[0])
        .args(&args[1..])
        .arg("--manifest-path")
        .arg(manifest)
        .output()
        .expect("cargo must be available for the external consumer");
    output
}

fn resolved_features(manifest: &Path, extra: &[&str]) -> (Vec<String>, BTreeSet<String>) {
    let mut args = vec!["metadata", "--format-version", "1", "--no-default-features"];
    args.extend_from_slice(extra);
    let output = cargo(manifest, &args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: Value = serde_json::from_slice(&output.stdout).unwrap();
    let package = metadata["packages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|package| {
            package["name"] == "tl-mltl"
                && package["manifest_path"]
                    .as_str()
                    .unwrap()
                    .starts_with(env!("CARGO_MANIFEST_DIR"))
        })
        .unwrap();
    let id = package["id"].as_str().unwrap();
    let nodes = metadata["resolve"]["nodes"].as_array().unwrap();
    let packages = nodes
        .iter()
        .map(|node| node["id"].as_str().unwrap().to_owned())
        .collect();
    let features = nodes.iter().find(|node| node["id"] == id).unwrap()["features"]
        .as_array()
        .unwrap()
        .iter()
        .map(|feature| feature.as_str().unwrap().to_owned())
        .collect();
    (features, packages)
}

const CONSUMER: &str = r#"
use tl_mltl::{analyze_horizon, evaluate_closed, evaluate_prefix, EvaluationLimits};
use tl_syntax::{Formula, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn main() {
    let nodes = [Node::new(NodeKind::Proposition { proposition: PropositionId(7) })];
    let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(0), &nodes).unwrap();
    let prefix = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(0), &nodes).unwrap();
    let trace = [vec![PropositionId(7)]];
    let result = (
        evaluate_closed(closed, "p", &trace, "t", EvaluationLimits::default()).unwrap(),
        evaluate_prefix(prefix, "p", &trace, "t", false, EvaluationLimits::default()).unwrap(),
        analyze_horizon(closed, "p").unwrap(),
    );
    #[cfg(feature = "infinite-trace")]
    assert_eq!(tl_mltl::infinite::FEATURE, "infinite-trace");
    print!("{}", serde_json::to_string(&result).unwrap());
}
"#;

// Trace: TC-138; FR-027-AC-1 through FR-027-AC-3, FR-028-AC-3, FR-034-AC-3
#[test]
fn external_consumer_feature_tree_and_complete_bounded_bytes() {
    let scratch = Path::new(env!("CARGO_MANIFEST_DIR")).join("target");
    fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::tempdir_in(scratch).unwrap();
    let root = temp.path();
    fs::create_dir(root.join("src")).unwrap();
    let manifest = root.join("Cargo.toml");
    let owner = serde_json::to_string(env!("CARGO_MANIFEST_DIR")).unwrap();
    fs::write(&manifest, format!(r#"[package]
name = "tl13-boundary-consumer"
version = "0.0.0"
edition = "2021"

[features]
infinite-trace = ["tl-mltl/infinite-trace"]

[dependencies]
tl-mltl = {{ path = {owner}, default-features = false }}
tl-syntax = {{ version = "=0.3.0", git = "https://github.com/agent-ix/tl-syntax.git", rev = "6aa9b11e29040d64b437da87c9944e3dedd34a86", features = ["alloc", "serde"] }}
serde_json = "=1.0.151"
"#)).unwrap();
    fs::write(root.join("src/main.rs"), CONSUMER).unwrap();
    let (off_features, off_packages) = resolved_features(&manifest, &[]);
    let (on_features, on_packages) =
        resolved_features(&manifest, &["--features", "infinite-trace"]);
    assert!(!off_features
        .iter()
        .any(|feature| feature == "infinite-trace"));
    assert!(on_features
        .iter()
        .any(|feature| feature == "infinite-trace"));
    assert_eq!(
        off_packages, on_packages,
        "provider introduced a feature-only dependency"
    );
    for entry in [
        "src/future/mod.rs",
        "src/past/mod.rs",
        "src/wire/mod.rs",
        "src/mapping/mod.rs",
    ] {
        assert!(
            !fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(entry))
                .unwrap()
                .contains("infinite::"),
            "{entry}"
        );
    }
    let off = cargo(&manifest, &["run", "--quiet", "--no-default-features"]);
    assert!(
        off.status.success(),
        "{}",
        String::from_utf8_lossy(&off.stderr)
    );
    let on = cargo(
        &manifest,
        &[
            "run",
            "--quiet",
            "--no-default-features",
            "--features",
            "infinite-trace",
        ],
    );
    assert!(
        on.status.success(),
        "{}",
        String::from_utf8_lossy(&on.stderr)
    );
    assert_eq!(
        off.stdout, on.stdout,
        "complete bounded report bytes changed"
    );
    let digest = format!("{:x}", Sha256::digest(&off.stdout));
    assert_eq!(
        digest,
        "597c473054fee7824c8aaa233c9d45dedb4b0d7aaee98d7596c5e1ac8bf0f636"
    );
    fs::write(
        root.join("src/main.rs"),
        "fn main() { let _ = tl_mltl::infinite::FEATURE; }",
    )
    .unwrap();
    let absent = cargo(&manifest, &["check", "--quiet", "--no-default-features"]);
    assert!(
        !absent.status.success(),
        "infinite module unexpectedly visible without feature"
    );
    assert!(String::from_utf8_lossy(&absent.stderr).contains("could not find `infinite`"));
}
