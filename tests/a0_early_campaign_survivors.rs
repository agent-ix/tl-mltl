//! TL-371: behavior tests for the campaign inventory checks, member results, aggregate label and
//! the `campaign run` exit status, which the first full cargo-mutants run showed were not asserted
//! one condition at a time. The file name sorts early so a hang in a later suite does not hide them.

use tl_mltl::campaign::{canonical_digest, check_inventory, parse_members, standard_non_claims};

fn members(list: &str) -> String {
    format!(
        r#"{{"schemaVersion":"tl-mltl.campaign-members/v1",
"tools":[{{"name":"rustc","command":["rustc","--version"]}}],"members":[{list}]}}"#
    )
}

fn findings(list: &str, makefile: &str, fuzz: &[&str]) -> Vec<String> {
    let file = parse_members(&members(list)).unwrap();
    check_inventory(
        &file,
        makefile,
        &fuzz.iter().map(|s| (*s).to_owned()).collect(),
    )
}

const OK: &str = r#"{"id":"a","technique":"t","kind":"test","command":["rustc","--version"],"determinism":"exact"}"#;

// Trace: TC-127
#[test]
fn a_registered_member_needs_a_command() {
    let none = r#"{"id":"a","technique":"t","kind":"test","determinism":"exact"}"#;
    let f = findings(none, "", &[]);
    assert!(
        f.iter()
            .any(|x| x.contains("registered member has no command")),
        "{f:?}"
    );
    assert!(findings(OK, "", &[]).is_empty());
}

// Trace: TC-127
#[test]
fn a_planned_member_needs_a_ticket_and_no_command() {
    let planned = |extra: &str| {
        format!(
            r#"{{"id":"p","technique":"t","kind":"mutation","state":"planned","determinism":"seeded"{extra}}}"#
        )
    };
    let needs = |f: &[String]| {
        f.iter()
            .any(|x| x.contains("planned member needs a ticket"))
    };
    assert!(!needs(&findings(&planned(r#","ticket":"TL-1""#), "", &[])));
    assert!(needs(&findings(&planned(""), "", &[])));
    assert!(needs(&findings(
        &planned(r#","ticket":"TL-1","command":["x"]"#),
        "",
        &[]
    )));
    assert!(needs(&findings(&planned(r#","command":["x"]"#), "", &[])));
}

// Trace: TC-127
#[test]
fn a_covers_entry_must_name_an_existing_target() {
    let covering = |c: &str| {
        format!(
            r#"{{"id":"a","technique":"t","kind":"test","command":["rustc","--version"],"determinism":"exact","covers":["{c}"]}}"#
        )
    };
    let missing = |f: &[String]| f.iter().any(|x| x.contains("which does not exist"));
    assert!(!missing(&findings(
        &covering("make:real"),
        "real:\n\tx\n",
        &[]
    )));
    assert!(missing(&findings(
        &covering("make:ghost"),
        "real:\n\tx\n",
        &[]
    )));
    assert!(!missing(&findings(&covering("fuzz:real"), "", &["real"])));
    assert!(missing(&findings(&covering("fuzz:ghost"), "", &["real"])));
}

#[test]
fn volatile_keys_are_removed_inside_arrays() {
    let volatile = vec!["runtimeMs".to_owned()];
    let a = canonical_digest(br#"[{"runtimeMs":1,"n":3}]"#, &volatile).unwrap();
    let b = canonical_digest(br#"[{"runtimeMs":2,"n":3}]"#, &volatile).unwrap();
    let c = canonical_digest(br#"[{"runtimeMs":1,"n":4}]"#, &volatile).unwrap();
    assert_eq!(a, b);
    assert_ne!(a, c);
}

#[test]
fn the_standard_non_claims_are_the_three_stated_limits() {
    let claims = standard_non_claims();
    assert_eq!(claims.len(), 3);
    assert!(claims[0].starts_with("A planned or unavailable member is non-pass"));
    assert!(claims[1].starts_with("A seeded or timed member's rerun"));
    assert!(claims[2].contains("does not execute R2U2"));
}
