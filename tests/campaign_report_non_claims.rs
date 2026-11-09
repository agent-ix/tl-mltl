//! TL-358: the campaign report states its populations and non-claims, and makes
//! no release or certification claim, before it reports any ratio.

use std::{fs, process::Command};

use tl_mltl::campaign::{run_campaign, standard_non_claims, Report, Status};

const MEMBERS: &str = r#"{
  "schemaVersion": "tl-mltl.campaign-members/v1",
  "tools": [{"name": "rustc", "command": ["rustc", "--version"]}],
  "members": [
    {"id": "ok", "technique": "t", "kind": "test", "command": ["rustc", "--version"],
     "tools": ["rustc"], "determinism": "exact"},
    {"id": "seeded", "technique": "t", "kind": "test", "command": ["rustc", "--version"],
     "tools": ["rustc"], "determinism": "seeded", "seed": "7"},
    {"id": "later", "technique": "t", "kind": "mutation", "state": "planned",
     "ticket": "TL-231", "determinism": "seeded"}
  ]
}"#;

/// A member whose evidence claims certification instead of agreement.
const CLAIMING_MEMBER: &str = r#"{
  "schemaVersion": "tl-mltl.campaign-members/v1",
  "tools": [],
  "members": [
    {"id": "claims", "technique": "t", "kind": "sweep",
     "command": ["sh", "-c", "printf '{\"result\":\"certified\"}' > \"$EV/a.json\""],
     "determinism": "exact", "evidence": {"env": "EV", "volatileKeys": []}}
  ]
}"#;

const CLAIM_WORDS: [&str; 5] = ["release", "certif", "qualif", "accepted by", "monitor"];
const RATIO_TOKENS: [&str; 5] = ["ratio", "percent", "rate", "fraction", "coverage"];

fn report_for(members: &str) -> Report {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("campaign")).unwrap();
    fs::write(dir.path().join("campaign/members.json"), members).unwrap();
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

fn report() -> Report {
    report_for(MEMBERS)
}

/// Whether `text` states `word` without a negation in the four words before it.
fn affirms(text: &str, word: &str) -> bool {
    let lower = text.to_lowercase();
    let Some(at) = lower.find(word) else {
        return false;
    };
    let before: Vec<&str> = lower[..at].split_whitespace().rev().take(4).collect();
    !before
        .iter()
        .any(|w| matches!(*w, "not" | "no" | "never" | "without" | "neither") || w.ends_with("n't"))
}

/// Lower-case alphanumeric tokens of a camelCase or snake_case key.
fn key_tokens(key: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    for c in key.chars() {
        if !c.is_alphanumeric() || c.is_uppercase() {
            out.push(String::new());
        }
        if c.is_alphanumeric() {
            out.last_mut().unwrap().extend(c.to_lowercase());
        }
    }
    out.retain(|t| !t.is_empty());
    out
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

// Trace: TC-122, NFR-005-AC-3
#[test]
fn report_carries_the_standard_non_claims() {
    let standard = standard_non_claims();
    assert!(!standard.is_empty());
    assert!(standard.iter().all(|c| !c.trim().is_empty()));
    let report = report();
    assert!(!report.non_claims.is_empty());
    assert_eq!(report.non_claims, standard);
}

// Trace: TC-122, NFR-005-AC-3
#[test]
fn report_states_populations_seeds_tools_and_limitations_before_any_ratio() {
    let report = report();
    // Populations with their result states, seeds and tool versions.
    assert_eq!(report.members.len(), 3);
    let seeded = report.members.iter().find(|m| m.id == "seeded").unwrap();
    assert_eq!(seeded.seed.as_deref(), Some("7"));
    assert!(report.tools.contains_key("rustc"));
    assert!(report.tools["rustc"].contains("rustc"));

    let text = serde_json::to_string(&report).unwrap();
    let at = |key: &str| text.find(&format!("\"{key}\":")).unwrap();
    assert!(at("source") < at("members"));
    assert!(at("tools") < at("members"));
    assert!(at("members") < at("nonClaims"));

    // The report carries no ratio field.
    let value: serde_json::Value = serde_json::from_str(&text).unwrap();
    let mut keys = Vec::new();
    collect_keys(&value, &mut keys);
    for key in &keys {
        assert!(
            !key_tokens(key)
                .iter()
                .any(|t| RATIO_TOKENS.contains(&t.as_str())),
            "ratio field `{key}`"
        );
    }
}

// Trace: TC-122, NFR-005-AC-3
#[test]
fn report_makes_no_release_or_certification_claim() {
    let report = report();
    assert!(matches!(
        report.aggregate.as_str(),
        "accept" | "incomplete" | "reject" | "partial"
    ));
    let mut texts: Vec<String> = vec![report.aggregate.clone()];
    texts.extend(report.non_claims.iter().cloned());
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
fn a_member_that_claims_certification_is_refused_not_passed() {
    let report = report_for(CLAIMING_MEMBER);
    assert_eq!(report.members[0].status, Status::Fail);
    assert_eq!(report.aggregate, "reject");
}

// Trace: TC-122, NFR-005-AC-3
#[test]
fn the_claim_detector_separates_claims_from_negations() {
    assert!(affirms("This build is certified for release", "certif"));
    assert!(affirms("Verified by monitor acceptance", "monitor"));
    assert!(!affirms(
        "The report does not claim certification",
        "certif"
    ));
    assert!(!affirms("It never implies release", "release"));
    // A distant negation does not excuse a claim.
    assert!(affirms("Not all checks ran; this is certified", "certif"));
    assert_eq!(key_tokens("generatedAt"), ["generated", "at"]);
    assert!(!key_tokens("generatedAt").iter().any(|t| t == "rate"));
}
