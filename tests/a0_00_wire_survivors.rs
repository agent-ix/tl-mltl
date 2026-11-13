//! TL-371: behavior tests for the shared owner wire machinery (`wire::common`), the trace and the
//! command readers. The first full cargo-mutants run showed that identity preimage extraction, the
//! lexical usage scan, digest helpers, error labels and the per-document ceilings were not asserted
//! at their boundaries. Each test pins exact bytes, counts or labels.

use serde::Serialize;
use sha2::{Digest, Sha256};
use std::sync::mpsc;
use std::time::Duration;
use tl_mltl::wire::common::{identity, is_sha256, produce, raw_sha256, read_expected, OWNER_MAX};
use tl_mltl::wire::{command, trace, OwnerLimits, OwnerReadErrorCode, OwnerUsage};
use tl_mltl::{CommandDocument, TraceDocument, TraceSchemaVersion};
use tl_syntax::PropositionId;

use erased::Erased as _;

/// Runs `work` on a thread and fails if it does not finish: a scan that no longer advances would
/// spin forever, and this turns that hang into an immediate failure.
fn within_seconds<T: Send + 'static>(seconds: u64, work: impl FnOnce() -> T + Send + 'static) -> T {
    let (sender, receiver) = mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(work());
    });
    receiver
        .recv_timeout(Duration::from_secs(seconds))
        .expect("the computation must terminate")
}

// Trace: TC-090
#[test]
fn owner_maxima_and_error_labels_are_exact() {
    within_seconds(30, owner_maxima_and_error_labels_are_exact_body);
}

fn owner_maxima_and_error_labels_are_exact_body() {
    assert_eq!(OWNER_MAX.max_input_bytes, 8_388_608);
    assert_eq!(OWNER_MAX.max_output_bytes, 8_388_608);
    assert_eq!(OWNER_MAX.max_string_bytes, 1_048_576);
    assert_eq!(OwnerLimits::owner_max(), OWNER_MAX);
    assert_eq!(OwnerLimits::default(), OWNER_MAX);
    let labels = [
        (OwnerReadErrorCode::InvalidUtf8, "TL-OWNER-INVALID-UTF8"),
        (OwnerReadErrorCode::InvalidJson, "TL-OWNER-INVALID-JSON"),
        (OwnerReadErrorCode::NonCanonical, "TL-OWNER-NONCANONICAL"),
        (
            OwnerReadErrorCode::ResourceIncomplete,
            "TL-OWNER-RESOURCE-INCOMPLETE",
        ),
        (
            OwnerReadErrorCode::ExpectedMismatch,
            "TL-OWNER-EXPECTED-MISMATCH",
        ),
        (
            OwnerReadErrorCode::InvalidCombination,
            "TL-OWNER-INVALID-COMBINATION",
        ),
        (OwnerReadErrorCode::Encoding, "TL-OWNER-ENCODING"),
    ];
    for (code, label) in labels {
        assert_eq!(code.as_str(), label);
    }
    let error = tl_mltl::wire::OwnerReadError::new(
        OwnerReadErrorCode::Encoding,
        "document",
        OwnerUsage::default(),
    );
    assert_eq!(error.to_string(), "TL-OWNER-ENCODING: document");
    // A ceiling below the owner maximum is kept; one above it is clamped.
    let lowered = OwnerLimits {
        max_depth: 3,
        max_input_bytes: usize::MAX,
        ..OwnerLimits::default()
    }
    .effective();
    assert_eq!((lowered.max_depth, lowered.max_input_bytes), (3, 8_388_608));
}

// Trace: TC-090
#[test]
fn hex_digests_match_known_vectors_and_the_validity_check_is_exact() {
    within_seconds(
        30,
        hex_digests_match_known_vectors_and_the_validity_check_is_exact_body,
    );
}

fn hex_digests_match_known_vectors_and_the_validity_check_is_exact_body() {
    assert_eq!(
        raw_sha256(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        raw_sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    for digit in ['0', '9', 'a', 'f'] {
        assert!(is_sha256(&digit.to_string().repeat(64)), "{digit}");
    }
    assert!(is_sha256(&raw_sha256(b"abc")));
    assert!(!is_sha256(&"a".repeat(63)));
    assert!(!is_sha256(&"a".repeat(65)));
    assert!(!is_sha256(&"g".repeat(64)));
    assert!(!is_sha256(&"A".repeat(64)));
    assert!(!is_sha256(&format!("{}g", "a".repeat(63))));
    assert!(!is_sha256(""));
}

#[derive(Serialize)]
struct Members<'a> {
    a: &'a str,
    b: serde_json::Value,
    ab: u32,
    z: &'a str,
}

fn expected_identity(contract: &str, preimage: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(contract.as_bytes());
    digest.update([0]);
    digest.update(preimage.as_bytes());
    format!("{:x}", digest.finalize())
}

// Trace: TC-090
#[test]
fn identity_omits_exactly_the_named_member_wherever_it_sits() {
    within_seconds(
        30,
        identity_omits_exactly_the_named_member_wherever_it_sits_body,
    );
}

fn identity_omits_exactly_the_named_member_wherever_it_sits_body() {
    let tricky = "x,\"y\\\"}],{[\\\\";
    let value = Members {
        a: tricky,
        b: serde_json::json!({"k": [1, {"m": "n,o}"}], "p": "q"}),
        ab: 7,
        z: "end",
    };
    let full = serde_json::to_string(&value).unwrap();
    let a_member = format!("\"a\":{},", serde_json::to_string(tricky).unwrap());
    let b_member = format!(
        "\"b\":{},",
        serde_json::to_string(&serde_json::json!({"k": [1, {"m": "n,o}"}], "p": "q"})).unwrap()
    );
    // First member: its trailing comma goes with it.
    assert_eq!(
        identity("c", &value, "a").unwrap(),
        expected_identity("c", &full.replacen(&a_member, "", 1))
    );
    // Middle member with nested containers and strings holding delimiters.
    assert_eq!(
        identity("c", &value, "b").unwrap(),
        expected_identity("c", &full.replacen(&b_member, "", 1))
    );
    // A member whose name is a prefix of another is matched exactly.
    assert_eq!(
        identity("c", &value, "ab").unwrap(),
        expected_identity("c", &full.replacen("\"ab\":7,", "", 1))
    );
    // Last member: its preceding comma goes with it.
    assert_eq!(
        identity("c", &value, "z").unwrap(),
        expected_identity("c", &full.replacen(",\"z\":\"end\"", "", 1))
    );
    // The contract name is part of the preimage.
    assert_ne!(
        identity("c", &value, "z").unwrap(),
        identity("d", &value, "z").unwrap()
    );
}

#[derive(Clone, Serialize)]
struct Only {
    a: u32,
}

#[derive(Serialize)]
struct Empty {}

// Trace: TC-090
#[test]
fn identity_refuses_what_it_cannot_split() {
    within_seconds(30, identity_refuses_what_it_cannot_split_body);
}

fn identity_refuses_what_it_cannot_split_body() {
    assert_eq!(
        identity("c", &Only { a: 1 }, "a").unwrap(),
        expected_identity("c", "{}")
    );
    let missing = identity("c", &Only { a: 1 }, "b").unwrap_err();
    assert_eq!(
        (missing.code(), missing.field()),
        (OwnerReadErrorCode::Encoding, "identityField")
    );
    let empty = identity("c", &Empty {}, "a").unwrap_err();
    assert_eq!(
        (empty.code(), empty.field()),
        (OwnerReadErrorCode::Encoding, "identityField")
    );
    for not_an_object in [identity("c", &vec![1, 2], "a"), identity("c", &"text", "a")] {
        let error = not_an_object.unwrap_err();
        assert_eq!(
            (error.code(), error.field()),
            (OwnerReadErrorCode::Encoding, "identityPreimage")
        );
    }
}

// Trace: TC-090
#[test]
fn the_lexical_scan_measures_depth_strings_and_fields() {
    within_seconds(30, the_lexical_scan_measures_depth_strings_and_fields_body);
}

fn the_lexical_scan_measures_depth_strings_and_fields_body() {
    let usage = |value: &dyn erased::Erased| value.usage();
    // Arrays count one visited field per element; objects one per member.
    assert_eq!(usage(&vec![1, 2, 3]).visited_fields, 3);
    assert_eq!(usage(&vec![vec![1], vec![2]]).depth, 2);
    assert_eq!(usage(&vec![vec![1], vec![2]]).visited_fields, 4);
    assert_eq!(usage(&vec!["ab", "cde"]).string_bytes, 3);
    // An escaped quote counts as two encoded bytes and does not end the string.
    assert_eq!(usage(&vec!["a\"b"]).string_bytes, 4);
    assert_eq!(usage(&vec!["a\\"]).string_bytes, 3);
    assert_eq!(usage(&vec!["x"]).wire_bytes, 5);
    // A string followed by more content is measured from its own start.
    assert_eq!(usage(&vec!["abcd", "e"]).string_bytes, 4);
    assert_eq!(usage(&Only { a: 5 }).visited_fields, 1);
}

mod erased {
    use super::*;

    pub trait Erased {
        fn usage(&self) -> OwnerUsage;
    }

    impl<T: Serialize + Clone> Erased for T {
        fn usage(&self) -> OwnerUsage {
            produce(self.clone(), OwnerUsage::default(), OwnerLimits::default())
                .unwrap()
                .usage()
        }
    }
}

// Trace: TC-090
#[test]
fn whitespace_and_unterminated_strings_neither_hang_nor_panic() {
    within_seconds(
        30,
        whitespace_and_unterminated_strings_neither_hang_nor_panic_body,
    );
}

fn whitespace_and_unterminated_strings_neither_hang_nor_panic_body() {
    let outcome = within_seconds(20, || {
        let limits = OwnerLimits::default();
        let ok = read_expected::<Vec<u32>, _>(b" [ 1 , 2 ] ", &vec![1, 2], limits, |_, _| {
            Ok(OwnerUsage::default())
        });
        let cut = read_expected::<Vec<String>, _>(b"[\"abc", &vec![], limits, |_, _| {
            Ok(OwnerUsage::default())
        });
        (ok.map(|(_, usage)| usage.visited_fields), cut.map(|_| ()))
    });
    // The spaces make the bytes non-canonical, but the lexical scan accepts them.
    let error = outcome.0.unwrap_err();
    assert_eq!(error.code(), OwnerReadErrorCode::NonCanonical);
    assert_eq!(
        outcome.1.unwrap_err().code(),
        OwnerReadErrorCode::InvalidJson
    );
}

// Trace: TC-090
#[test]
fn produce_reports_the_bytes_it_attempted_to_write() {
    within_seconds(30, produce_reports_the_bytes_it_attempted_to_write_body);
}

fn produce_reports_the_bytes_it_attempted_to_write_body() {
    let limits = OwnerLimits {
        max_output_bytes: 3,
        ..OwnerLimits::default()
    };
    let error = produce(vec![1, 2, 3], OwnerUsage::default(), limits).unwrap_err();
    assert_eq!(
        (error.code(), error.field()),
        (OwnerReadErrorCode::ResourceIncomplete, "outputBytes")
    );
    assert!(error.usage().wire_bytes > 3, "{:?}", error.usage());
    let input_error = read_expected::<Vec<u32>, _>(
        b"[1,2,3]",
        &vec![1, 2, 3],
        OwnerLimits {
            max_input_bytes: 4,
            ..OwnerLimits::default()
        },
        |_, _| Ok(OwnerUsage::default()),
    )
    .unwrap_err();
    assert_eq!(input_error.field(), "inputBytes");
    assert_eq!(input_error.usage().wire_bytes, 7);
}

fn trace_document(id: &str, instants: Vec<Vec<PropositionId>>) -> TraceDocument {
    TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: id.to_owned(),
        closed: true,
        instants,
    }
}

// Trace: TC-090
#[test]
fn a_trace_reports_counts_and_enforces_each_ceiling_inclusively() {
    within_seconds(
        30,
        a_trace_reports_counts_and_enforces_each_ceiling_inclusively_body,
    );
}

fn a_trace_reports_counts_and_enforces_each_ceiling_inclusively_body() {
    let document = trace_document("t", vec![vec![PropositionId(1), PropositionId(2)], vec![]]);
    let owned = trace::derive(&document, OwnerLimits::default()).unwrap();
    assert_eq!(owned.usage().positions, 2);
    assert_eq!(owned.usage().propositions, 2);
    let validated =
        trace::ValidatedTrace::from_json_bytes(owned.bytes(), OwnerLimits::default()).unwrap();
    assert_eq!(validated.canonical_json_bytes(), owned.bytes());
    assert!(!validated.canonical_json_bytes().is_empty());
    assert_eq!(validated.usage().positions, 2);
    assert_eq!(validated.usage().propositions, 2);

    let field = |limits: OwnerLimits, document: &TraceDocument| {
        trace::derive(document, limits)
            .err()
            .map(|e| (e.code(), e.field()))
    };
    let default = OwnerLimits::default();
    let resource = OwnerReadErrorCode::ResourceIncomplete;
    assert_eq!(
        field(
            OwnerLimits {
                max_positions: 2,
                ..default
            },
            &document
        ),
        None
    );
    assert_eq!(
        field(
            OwnerLimits {
                max_positions: 1,
                ..default
            },
            &document
        ),
        Some((resource, "positions"))
    );
    assert_eq!(
        field(
            OwnerLimits {
                max_propositions: 2,
                ..default
            },
            &document
        ),
        None
    );
    assert_eq!(
        field(
            OwnerLimits {
                max_propositions: 1,
                ..default
            },
            &document
        ),
        Some((resource, "propositions"))
    );
    // The identity bound is the smaller of 256 and the string ceiling, inclusive.
    let combination = OwnerReadErrorCode::InvalidCombination;
    let named = |len: usize| trace_document(&"i".repeat(len), vec![]);
    assert_eq!(field(default, &named(256)), None);
    assert_eq!(field(default, &named(257)), Some((combination, "traceId")));
    assert_eq!(field(default, &named(0)), Some((combination, "traceId")));
    let short = OwnerLimits {
        max_string_bytes: 20,
        ..default
    };
    assert_eq!(field(short, &named(20)), None);
    assert_eq!(field(short, &named(21)), Some((combination, "traceId")));
}

fn command_document(formula_id: &str, trace: Option<&TraceDocument>) -> CommandDocument {
    serde_json::from_value(serde_json::json!({
        "schemaVersion": "tl-mltl.command/v1",
        "operation": "evaluate",
        "formulaId": formula_id,
        "formula": {
            "schema_version": "tl-syntax.formula/v1",
            "semantic_profile": "mltl.online-prefix/v1",
            "root": 1,
            "nodes": [
                {"kind": "proposition", "proposition": 1},
                {"kind": "not", "operand": 0}
            ]
        },
        "trace": trace.map(|t| serde_json::to_value(t).unwrap()),
    }))
    .unwrap()
}

// Trace: TC-090
#[test]
fn a_command_reports_its_parts_and_is_read_only_against_the_expected_document() {
    within_seconds(
        30,
        a_command_reports_its_parts_and_is_read_only_against_the_expected_document_body,
    );
}

fn a_command_reports_its_parts_and_is_read_only_against_the_expected_document_body() {
    let trace = trace_document(
        "t",
        vec![
            vec![PropositionId(1)],
            vec![PropositionId(1), PropositionId(2)],
        ],
    );
    let document = command_document("f", Some(&trace));
    let owned = command::derive(&document, OwnerLimits::default()).unwrap();
    let usage = owned.usage();
    assert_eq!(usage.formula_nodes, 2);
    assert_eq!(usage.positions, 2);
    assert_eq!(usage.propositions, 3);
    let read = command::read(owned.bytes(), &document, OwnerLimits::default()).unwrap();
    assert_eq!(read.document(), &document);
    assert_eq!(read.usage().formula_nodes, 2);
    assert_eq!(read.usage().positions, 2);
    assert_eq!(read.usage().propositions, 3);
    let other = command_document("g", Some(&trace));
    let mismatch = command::read(owned.bytes(), &other, OwnerLimits::default()).unwrap_err();
    assert_eq!(
        (mismatch.code(), mismatch.field()),
        (OwnerReadErrorCode::ExpectedMismatch, "command")
    );
    let without = command_document("f", None);
    let bare = command::derive(&without, OwnerLimits::default()).unwrap();
    assert_eq!((bare.usage().positions, bare.usage().propositions), (0, 0));
}

// Trace: TC-090
#[test]
fn a_command_formula_identity_is_bounded_inclusively() {
    within_seconds(30, a_command_formula_identity_is_bounded_inclusively_body);
}

fn a_command_formula_identity_is_bounded_inclusively_body() {
    let default = OwnerLimits::default();
    let outcome = |limits: OwnerLimits, id: &str| {
        command::derive(&command_document(id, None), limits)
            .err()
            .map(|e| (e.code(), e.field()))
    };
    let combination = OwnerReadErrorCode::InvalidCombination;
    assert_eq!(outcome(default, &"i".repeat(256)), None);
    assert_eq!(
        outcome(default, &"i".repeat(257)),
        Some((combination, "formulaId"))
    );
    assert_eq!(outcome(default, ""), Some((combination, "formulaId")));
    let short = OwnerLimits {
        max_string_bytes: 30,
        ..default
    };
    assert_eq!(outcome(short, &"i".repeat(30)), None);
    assert_eq!(
        outcome(short, &"i".repeat(31)),
        Some((combination, "formulaId"))
    );
}

// Trace: TC-090
#[test]
fn a_member_value_with_an_escaped_quote_followed_by_a_comma_is_skipped_whole() {
    within_seconds(30, || {
        let value = Members {
            a: "q\"r,s",
            b: serde_json::json!("a\\"),
            ab: 1,
            z: "end",
        };
        let full = serde_json::to_string(&value).unwrap();
        let a_member = format!("\"a\":{},", serde_json::to_string("q\"r,s").unwrap());
        assert_eq!(
            identity("c", &value, "a").unwrap(),
            expected_identity("c", &full.replacen(&a_member, "", 1))
        );
        // A trailing backslash escapes itself and so does not swallow the closing quote.
        let b_member = format!("\"b\":{},", serde_json::to_string("a\\").unwrap());
        assert_eq!(
            identity("c", &value, "b").unwrap(),
            expected_identity("c", &full.replacen(&b_member, "", 1))
        );
    });
}

// Trace: TC-090
#[test]
fn empty_containers_count_no_fields_and_the_depth_limit_is_one_past_the_ceiling() {
    within_seconds(30, || {
        assert_eq!(Vec::<u32>::new().usage().visited_fields, 0);
        assert_eq!(vec![Vec::<u32>::new()].usage().visited_fields, 1);
        let deep = |levels: usize| {
            let bytes = format!("{}{}", "[".repeat(levels), "]".repeat(levels));
            read_expected::<serde_json::Value, _>(
                bytes.as_bytes(),
                &serde_json::Value::Null,
                OwnerLimits::default(),
                |_, _| Ok(OwnerUsage::default()),
            )
        };
        let at_limit = deep(64).unwrap_err();
        assert_eq!(at_limit.code(), OwnerReadErrorCode::ExpectedMismatch);
        assert_ne!(at_limit.field(), "depth");
        let past = deep(65).unwrap_err();
        assert_eq!(
            (past.code(), past.field()),
            (OwnerReadErrorCode::ResourceIncomplete, "depth")
        );
        assert_eq!(past.usage().depth, 65);
    });
}
