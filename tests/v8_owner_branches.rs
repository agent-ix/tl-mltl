//! Focused public owner-boundary cases from the FR-050 critical branch census.

use serde_json::json;
use tl_mltl::wire::common::{
    is_sha256, produce, read_expected, OwnerLimits, OwnerReadErrorCode, OwnerUsage,
};
use tl_mltl::wire::{command, trace, CommandDocument, TraceDocument};

// Trace: FR-050-AC-1
#[test]
fn owner_wire_reports_real_output_limit_expected_mismatch_and_digest_syntax() {
    let value = json!({"a": [1, 2, 3]});
    let short = OwnerLimits {
        max_output_bytes: 4,
        ..OwnerLimits::default()
    };
    let limited = produce(value.clone(), OwnerUsage::default(), short).unwrap_err();
    assert_eq!(limited.code(), OwnerReadErrorCode::ResourceIncomplete);
    assert_eq!(limited.field(), "outputBytes");
    assert!(limited.usage().wire_bytes > short.max_output_bytes);

    let produced = produce(value, OwnerUsage::default(), OwnerLimits::default()).unwrap();
    let wrong = json!({"a": [1, 2, 4]});
    let mismatch = read_expected(produced.bytes(), &wrong, OwnerLimits::default(), |_, _| {
        Ok(OwnerUsage::default())
    })
    .unwrap_err();
    assert_eq!(mismatch.code(), OwnerReadErrorCode::ExpectedMismatch);

    assert!(is_sha256(&"0".repeat(64)));
    assert!(is_sha256(&"a".repeat(64)));
    assert!(!is_sha256(&"A".repeat(64)));
    assert!(!is_sha256(&"g".repeat(64)));
    assert!(!is_sha256(&"0".repeat(63)));

    let one_digit = produce(
        json!({"a": [1]}),
        OwnerUsage::default(),
        OwnerLimits::default(),
    )
    .unwrap();
    let two_digits = produce(
        json!({"a": [12]}),
        OwnerUsage::default(),
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(
        one_digit.usage().visited_fields,
        two_digits.usage().visited_fields
    );

    let malformed = br#"{"a":"unterminated"#;
    assert_eq!(
        read_expected(malformed, &json!({}), OwnerLimits::default(), |_, _| {
            Ok(OwnerUsage::default())
        })
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::InvalidJson
    );
}

#[derive(Clone)]
enum BoundedSerialization {
    ExceedsBudget,
    Fails,
}

impl serde::Serialize for BoundedSerialization {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Self::ExceedsBudget => serializer.serialize_str("outside-budget"),
            Self::Fails => Err(<S::Error as serde::ser::Error>::custom(
                "deliberate serialization failure",
            )),
        }
    }
}

// Trace: FR-050-AC-1
#[test]
fn owner_producer_distinguishes_serialization_failure_from_output_budget() {
    let accepted = produce(
        BoundedSerialization::ExceedsBudget,
        OwnerUsage::default(),
        OwnerLimits::default(),
    )
    .unwrap();
    assert_eq!(accepted.bytes(), br#""outside-budget""#);

    let short = OwnerLimits {
        max_output_bytes: 4,
        ..OwnerLimits::default()
    };
    let limited = produce(
        BoundedSerialization::ExceedsBudget,
        OwnerUsage::default(),
        short,
    )
    .err()
    .unwrap();
    assert_eq!(limited.code(), OwnerReadErrorCode::ResourceIncomplete);
    assert_eq!(limited.field(), "outputBytes");

    let error = produce(
        BoundedSerialization::Fails,
        OwnerUsage::default(),
        OwnerLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code(), OwnerReadErrorCode::Encoding);
    assert_eq!(error.field(), "document");
}

// Trace: FR-050-AC-1, FR-045-AC-1
#[test]
fn trace_owner_admits_exact_bytes_and_refuses_limits_and_identity_confusion() {
    let document: TraceDocument = serde_json::from_value(json!({
        "schemaVersion":"tl-mltl.trace/v1", "traceId":"trace-a", "closed":true,
        "instants":[[0,1], [1]]
    }))
    .unwrap();
    let limits = OwnerLimits::default();
    let owner = trace::derive(&document, limits).unwrap();
    let admitted = trace::ValidatedTrace::from_json_bytes(owner.bytes(), limits).unwrap();
    assert_eq!(admitted.document(), &document);
    assert_eq!(admitted.canonical_json_bytes(), owner.bytes());
    assert_eq!(
        trace::read(owner.bytes(), &document, limits)
            .unwrap()
            .usage()
            .positions,
        2
    );
    let mut other = document.clone();
    other.trace_id = "trace-b".into();
    assert_eq!(
        trace::read(owner.bytes(), &other, limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );

    let limited = OwnerLimits {
        max_positions: 1,
        ..limits
    };
    assert_eq!(
        trace::derive(&document, limited).unwrap_err().field(),
        "positions"
    );
    let limited = OwnerLimits {
        max_propositions: 2,
        ..limits
    };
    assert_eq!(
        trace::derive(&document, limited).unwrap_err().field(),
        "propositions"
    );
    let mut unsorted = document.clone();
    unsorted.instants[0].reverse();
    assert_eq!(
        trace::derive(&unsorted, limits).unwrap_err().field(),
        "instants"
    );
    let mut empty = document;
    empty.trace_id.clear();
    assert_eq!(
        trace::ValidatedTrace::admit(&empty, limits)
            .unwrap_err()
            .field(),
        "traceId"
    );
}

// Trace: FR-050-AC-1, FR-045-AC-1
#[test]
fn command_owner_admits_embedded_formula_and_trace_and_refuses_mismatch() {
    let request: CommandDocument = serde_json::from_value(json!({
        "schemaVersion":"tl-mltl.command/v1", "operation":"evaluate",
        "formulaId":"formula-a",
        "formula":{
            "schema_version":"tl-syntax.formula/v1",
            "semantic_profile":"mltl.closed-trace/v1", "root":0,
            "nodes":[{"kind":"true"}]
        },
        "trace":{
            "schemaVersion":"tl-mltl.trace/v1", "traceId":"trace-a",
            "closed":true, "instants":[[0]]
        }
    }))
    .unwrap();
    let limits = OwnerLimits::default();
    let owner = command::derive(&request, limits).unwrap();
    let admitted = command::ValidatedCommand::from_json_bytes(owner.bytes(), limits).unwrap();
    assert_eq!(admitted.document(), &request);
    assert_eq!(admitted.bytes(), owner.bytes());
    assert_eq!(admitted.usage().positions, 1);
    assert_eq!(
        command::read(owner.bytes(), &request, limits)
            .unwrap()
            .usage()
            .formula_nodes,
        1
    );
    let mut other = request.clone();
    other.formula_id = "formula-b".into();
    assert_eq!(
        command::read(owner.bytes(), &other, limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );
    let mut empty = request.clone();
    empty.formula_id.clear();
    assert_eq!(
        command::derive(&empty, limits).unwrap_err().field(),
        "formulaId"
    );
    let mut long = request.clone();
    long.formula_id = "f".repeat(257);
    assert_eq!(
        command::derive(&long, limits).unwrap_err().field(),
        "formulaId"
    );
    let mut no_trace = request.clone();
    no_trace.trace = None;
    assert_eq!(
        command::derive(&no_trace, limits)
            .unwrap()
            .usage()
            .positions,
        0
    );
    let limited = OwnerLimits {
        max_formula_nodes: 0,
        ..limits
    };
    assert_eq!(
        command::derive(&request, limited).unwrap_err().field(),
        "formula"
    );
}
