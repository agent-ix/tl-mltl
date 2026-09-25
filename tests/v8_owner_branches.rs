//! Focused public owner-boundary cases from the FR-050 critical branch census.

use serde_json::json;
use tl_mltl::wire::common::{
    is_sha256, produce, read_expected, OwnerLimits, OwnerReadErrorCode, OwnerUsage,
};

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
