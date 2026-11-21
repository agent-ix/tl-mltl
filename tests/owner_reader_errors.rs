//! Caller-owned semantic errors cross the bounded canonical reader unchanged.

use tl_mltl::wire::common::read_expected;
use tl_mltl::wire::{OwnerLimits, OwnerReadError, OwnerReadErrorCode, OwnerUsage};

#[derive(Debug, Eq, PartialEq)]
enum CallerError {
    ContractMismatch,
    Boundary(OwnerReadError),
}

impl From<OwnerReadError> for CallerError {
    fn from(error: OwnerReadError) -> Self {
        Self::Boundary(error)
    }
}

/// Trace: FR-018-AC-2
#[test]
fn semantic_refusal_propagates_before_expected_value_comparison() {
    let result = read_expected(b"1", &2_u8, OwnerLimits::owner_max(), |_, _| {
        Err(CallerError::ContractMismatch)
    });
    assert_eq!(result, Err(CallerError::ContractMismatch));
}

/// Trace: FR-018-AC-2, NFR-001-AC-3
#[test]
fn boundary_refusal_converts_before_the_validator_runs() {
    let limits = OwnerLimits {
        max_input_bytes: 0,
        ..OwnerLimits::owner_max()
    };
    let result: Result<(u8, OwnerUsage), CallerError> = read_expected(b"1", &1, limits, |_, _| {
        panic!("input must be refused first")
    });
    let CallerError::Boundary(error) = result.expect_err("byte ceiling must refuse") else {
        panic!("boundary error must retain its source");
    };
    assert_eq!(error.code(), OwnerReadErrorCode::ResourceIncomplete);
    assert_eq!(error.field(), "inputBytes");
}

/// Trace: FR-018-AC-2
#[test]
fn canonical_and_expected_refusals_convert_to_the_caller_error() {
    for (bytes, expected, code) in [
        (&b"1 "[..], 1_u8, OwnerReadErrorCode::NonCanonical),
        (&b"1"[..], 2_u8, OwnerReadErrorCode::ExpectedMismatch),
    ] {
        let result = read_expected(bytes, &expected, OwnerLimits::owner_max(), |_, _| {
            Ok::<_, CallerError>(OwnerUsage::default())
        });
        let CallerError::Boundary(error) = result.expect_err("invalid document must refuse") else {
            panic!("boundary error must retain its source");
        };
        assert_eq!(error.code(), code);
    }
}

/// Trace: FR-018-AC-1
#[test]
fn canonical_value_and_usage_survive_the_caller_error_type() {
    let (value, usage) = read_expected(b"1", &1_u8, OwnerLimits::owner_max(), |_, _| {
        Ok::<_, CallerError>(OwnerUsage {
            formula_nodes: 2,
            ..OwnerUsage::default()
        })
    })
    .expect("canonical expected document must pass");
    assert_eq!(value, 1);
    assert_eq!(usage.wire_bytes, 1);
    assert_eq!(usage.formula_nodes, 2);
}
