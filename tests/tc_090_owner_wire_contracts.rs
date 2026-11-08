//! TC-090: the retained TL-owned wire contracts publish immutable schema bytes and digests,
//! read strictly, and carry no dependency on the Quire ecosystem.

use sha2::{Digest, Sha256};
use tl_mltl::past::{history, requirement, result};
use tl_mltl::wire::{command, trace, OwnerLimits, OwnerReadErrorCode};
use tl_mltl::{TraceDocument, TraceSchemaVersion};
use tl_syntax::PropositionId;

fn sha256(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn sample_trace() -> TraceDocument {
    TraceDocument {
        schema_version: TraceSchemaVersion::V1,
        trace_id: "tc-090".to_owned(),
        closed: true,
        instants: vec![vec![PropositionId(1)], vec![]],
    }
}

// Trace: TC-090, FR-018-AC-1
#[test]
fn every_retained_contract_publishes_schema_bytes_matching_its_digest() {
    for (name, bytes, digest) in [
        ("trace", trace::SCHEMA_BYTES, trace::SCHEMA_SHA256),
        ("command", command::SCHEMA_BYTES, command::SCHEMA_SHA256),
        (
            "position-history",
            history::SCHEMA_BYTES,
            history::SCHEMA_SHA256,
        ),
        (
            "history-requirement",
            requirement::SCHEMA_BYTES,
            requirement::SCHEMA_SHA256,
        ),
        (
            "past-evaluation",
            result::SCHEMA_BYTES,
            result::SCHEMA_SHA256,
        ),
    ] {
        assert_eq!(sha256(bytes), digest, "{name} schema digest");
        serde_json::from_slice::<serde_json::Value>(bytes)
            .unwrap_or_else(|error| panic!("{name} schema is not JSON: {error}"));
    }
}

// Trace: TC-090, FR-018-AC-1, FR-018-AC-2
#[test]
fn trace_reader_accepts_only_canonical_bytes_and_refuses_with_typed_errors() {
    let limits = OwnerLimits::default();
    let document = sample_trace();
    let owner = trace::derive(&document, limits).unwrap();
    let canonical = owner.bytes().to_vec();
    let accepted = trace::read(&canonical, &document, limits).unwrap();
    assert_eq!(accepted.bytes(), canonical.as_slice());

    let refused = |bytes: &[u8]| {
        trace::ValidatedTrace::from_json_bytes(bytes, limits)
            .expect_err("reader must refuse")
            .code()
    };

    let mut trailing = canonical.clone();
    trailing.push(b'\n');
    assert!(matches!(
        refused(&trailing),
        OwnerReadErrorCode::NonCanonical | OwnerReadErrorCode::InvalidJson
    ));

    let text = String::from_utf8(canonical.clone()).unwrap();
    let unknown = text.replacen('{', "{\"extra\":1,", 1);
    assert!(matches!(
        refused(unknown.as_bytes()),
        OwnerReadErrorCode::InvalidJson | OwnerReadErrorCode::NonCanonical
    ));

    let duplicate = text.replacen('{', "{\"closed\":true,", 1);
    assert!(matches!(
        refused(duplicate.as_bytes()),
        OwnerReadErrorCode::InvalidJson | OwnerReadErrorCode::NonCanonical
    ));

    let wrong_contract = text.replace("tl-mltl.trace/v1", "tl-mltl.trace/v2");
    assert!(matches!(
        refused(wrong_contract.as_bytes()),
        OwnerReadErrorCode::InvalidJson
            | OwnerReadErrorCode::ContractMismatch
            | OwnerReadErrorCode::NonCanonical
    ));

    assert_eq!(refused(&[0xff, 0xfe]), OwnerReadErrorCode::InvalidUtf8);

    let mut other = sample_trace();
    other.trace_id = "different".to_owned();
    assert_eq!(
        trace::read(&canonical, &other, limits)
            .err()
            .unwrap()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn owner_ceilings_are_enforced_before_acceptance() {
    let document = sample_trace();
    let limits = OwnerLimits {
        max_positions: 1,
        ..OwnerLimits::default()
    };
    assert_eq!(
        trace::derive(&document, limits).err().unwrap().code(),
        OwnerReadErrorCode::ResourceIncomplete
    );
    let canonical = trace::derive(&document, OwnerLimits::default())
        .unwrap()
        .bytes()
        .to_vec();
    let small = OwnerLimits {
        max_input_bytes: canonical.len() - 1,
        ..OwnerLimits::default()
    };
    assert_eq!(
        trace::ValidatedTrace::from_json_bytes(&canonical, small)
            .err()
            .unwrap()
            .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );
}

// Trace: TC-090, FR-018-AC-3
#[test]
fn crate_has_no_quire_ecosystem_dependency() {
    let manifest = include_str!("../Cargo.toml");
    for forbidden in [
        "quire-observation",
        "agent-ix-baseline-producer",
        "quire-mltl",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "Cargo.toml names {forbidden}"
        );
    }
}
