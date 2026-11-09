//! TL-358: the campaign report states its populations and non-claims, and makes
//! no release or certification claim, before it reports any ratio.

use std::{fs, process::Command};

use tl_mltl::campaign::{run_campaign, standard_non_claims, Report};

const MEMBERS: &str = r#"{
  "schemaVersion": "tl-mltl.campaign-members/v1",
  "tools": [{"name": "rustc", "command": ["rustc", "--version"]}],
  "members": [
    {"id": "ok", "technique": "t", "kind": "test", "command": ["rustc", "--version"],
     "tools": ["rustc"], "determinism": "exact"},
    {"id": "later", "technique": "t", "kind": "mutation", "state": "planned",
     "ticket": "TL-231", "determinism": "seeded"}
  ]
}"#;

const CLAIM_WORDS: [&str; 4] = ["release", "certif", "qualif", "accepted by"];
const RATIO_WORDS: [&str; 5] = ["ratio", "percent", "rate", "fraction", "coverage"];

fn report() -> Report {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("campaign")).unwrap();
    fs::write(dir.path().join("campaign/members.json"), MEMBERS).unwrap();
    fs::write(dir.path().join("Makefile"), "ci: a\n").unwrap();
    fs::write(dir.path().join("Cargo.lock"), "lock\n").unwrap();
    for args in [
        &["init", "-q"][..],
        &["add", "."],
        &["commit", "-q", "-m", "x"],
    ] {
        let ok = Command::new("git")
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid"])
            .args(args)
            .current_dir(dir.path())
            .status()
            .unwrap()
            .success();
        assert!(ok, "git {args:?}");
    }
    let out = tempfile::tempdir().unwrap();
    run_campaign(dir.path(), out.path(), &[]).unwrap()
}

/// Whether `text` states the word without negating it.
fn affirms(text: &str, word: &str) -> bool {
    let lower = text.to_lowercase();
    lower.contains(word)
        && !(lower.contains(" not ") || lower.contains(" no ") || lower.contains("never"))
}

// Trace: TC-122, NFR-005-AC-3, TC-103, NFR-004-AC-3
#[test]
fn report_carries_the_standard_non_claims() {
    let standard = standard_non_claims();
    assert!(!standard.is_empty());
    assert!(standard.iter().all(|c| !c.trim().is_empty()));
    let report = report();
    assert!(!report.non_claims.is_empty());
    assert_eq!(report.non_claims, standard);
}

// Trace: TC-122, NFR-005-AC-3, TC-103, NFR-004-AC-3
#[test]
fn report_lists_populations_and_limitations_before_any_ratio() {
    let text = serde_json::to_string(&report()).unwrap();
    let at = |key: &str| text.find(&format!("\"{key}\":")).unwrap();
    // Populations (members with their result states), tool versions and the
    // limitations (non-claims) are all stated.
    assert!(at("source") < at("members"));
    assert!(at("tools") < at("members"));
    assert!(at("members") < at("nonClaims"));
    // The report states no ratio at all; were one added it would have to follow
    // both the populations and the limitations.
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut keys = Vec::new();
    collect_keys(&value, &mut keys);
    for key in keys
        .iter()
        .filter(|k| RATIO_WORDS.iter().any(|w| k.to_lowercase().contains(w)))
    {
        assert!(
            at(key) > at("nonClaims"),
            "ratio field `{key}` precedes the limitations"
        );
    }
    assert!(keys
        .iter()
        .all(|k| !RATIO_WORDS.iter().any(|w| k.to_lowercase().contains(w))));
}

fn collect_keys(value: &serde_json::Value, out: &mut Vec<String>) {
    match value {
        serde_json::Value::Object(map) => {
            for (k, v) in map {
                out.push(k.clone());
                collect_keys(v, out);
            }
        }
        serde_json::Value::Array(items) => items.iter().for_each(|v| collect_keys(v, out)),
        _ => {}
    }
}

// Trace: TC-122, NFR-005-AC-3, TC-103, NFR-004-AC-3
#[test]
fn report_makes_no_release_or_certification_claim() {
    let report = report();
    assert!(matches!(
        report.aggregate.as_str(),
        "accept" | "incomplete" | "reject" | "partial"
    ));
    for claim in &report.non_claims {
        for word in CLAIM_WORDS {
            assert!(!affirms(claim, word), "non-claim affirms `{word}`: {claim}");
        }
    }
    // No member detail, tool value, or aggregate states such a claim either.
    let mut texts: Vec<String> = vec![report.aggregate.clone()];
    texts.extend(report.members.iter().filter_map(|m| m.detail.clone()));
    texts.extend(report.tools.values().cloned());
    for text in texts {
        for word in CLAIM_WORDS {
            assert!(!affirms(&text, word), "report affirms `{word}`: {text}");
        }
    }
}

// Trace: TC-122, NFR-005-AC-3
#[test]
fn the_claim_detector_flags_an_affirmative_claim() {
    assert!(affirms("This build is certified for release", "certif"));
    assert!(!affirms(
        "The report does not claim certification",
        "certif"
    ));
}
