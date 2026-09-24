//! Focused public owner-boundary cases from the FR-050 critical branch census.

use serde_json::json;
use sha2::{Digest as _, Sha256};
use tl_mltl::wire::common::{
    identity, is_sha256, produce, read_expected, OwnerLimits, OwnerReadErrorCode, OwnerUsage,
};

fn expected_identity(contract: &str, preimage: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(contract.as_bytes());
    digest.update([0]);
    digest.update(preimage.as_bytes());
    format!("{:x}", digest.finalize())
}

// Trace: FR-050-AC-1
#[test]
fn owner_identity_omits_first_middle_and_last_members_without_losing_nested_json() {
    let value = json!({"a": [{"q": "x\"y"}], "m": {"n": [1, 2]}, "z": "tail"});
    let contract = "tl-owner-v8-case";
    for (omitted, preimage) in [
        ("a", r#"{"m":{"n":[1,2]},"z":"tail"}"#),
        ("m", r#"{"a":[{"q":"x\"y"}],"z":"tail"}"#),
        ("z", r#"{"a":[{"q":"x\"y"}],"m":{"n":[1,2]}}"#),
    ] {
        assert_eq!(
            identity(contract, &value, omitted).unwrap(),
            expected_identity(contract, preimage)
        );
    }
    let missing = identity(contract, &value, "absent").unwrap_err();
    assert_eq!(missing.code(), OwnerReadErrorCode::Encoding);
    assert_eq!(missing.field(), "identityField");
    assert_eq!(
        identity(contract, &json!([1, 2]), "a").unwrap_err().code(),
        OwnerReadErrorCode::Encoding
    );
    assert_eq!(
        identity(contract, &json!({"a": 1}), "a").unwrap(),
        expected_identity(contract, "{}")
    );
    assert_eq!(
        identity(contract, &json!({}), "absent")
            .unwrap_err()
            .field(),
        "identityField"
    );
}

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
struct FailingSerialization;

impl serde::Serialize for FailingSerialization {
    fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
        Err(<S::Error as serde::ser::Error>::custom(
            "deliberate serialization failure",
        ))
    }
}

// Trace: FR-050-AC-1
#[test]
fn owner_producer_distinguishes_serialization_failure_from_output_budget() {
    let error = produce(
        FailingSerialization,
        OwnerUsage::default(),
        OwnerLimits::default(),
    )
    .err()
    .unwrap();
    assert_eq!(error.code(), OwnerReadErrorCode::Encoding);
    assert_eq!(error.field(), "document");
}
