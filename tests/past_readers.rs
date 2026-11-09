//! TL-361: round-trip and rejection tests for the `past::requirement` and
//! `past::result` owner readers (`tl-mltl.history-requirement/v1`,
//! `tl-mltl.past-evaluation/v1`). Every refusal asserts the typed
//! `OwnerReadErrorCode` and the exact `field` locus, never the message.
//! Error-variant and limit-edge coverage of the underlying report types lives
//! in `tests/error_variants.rs` and `tests/limit_register.rs`; this file
//! exercises the byte-level reader contract.

use tl_mltl::past::{requirement, result};
use tl_mltl::wire::{OwnerLimits, OwnerReadErrorCode, OwnerUsage};
use tl_mltl::{
    analyze_required_history, evaluate_past, ClockBinding, HistoryRequirementReport,
    PastEvaluationLimits, PastEvaluationRelationInput, PastEvaluationReport,
    PositionHistoryDocument, PositionObservation,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

fn once_nodes() -> Vec<Node> {
    vec![
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Once {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(0),
        }),
    ]
}

fn formula(nodes: &[Node]) -> Formula<'_> {
    Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(u32::try_from(nodes.len() - 1).unwrap()),
        nodes,
    )
    .unwrap()
}

fn requirement_report() -> HistoryRequirementReport {
    analyze_required_history(formula(&once_nodes()), "f").unwrap()
}

fn result_report() -> PastEvaluationReport {
    let observations = (0..4)
        .map(|position| PositionObservation::new(position, vec![PropositionId(0)], None))
        .collect();
    let history = PositionHistoryDocument::new(
        "h",
        1,
        0,
        3,
        Some(ClockBinding::EventPosition),
        observations,
    )
    .unwrap();
    evaluate_past(
        formula(&once_nodes()),
        "f",
        &history,
        1,
        "m",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap()
}

fn code_of<T>(
    outcome: Result<T, tl_mltl::wire::OwnerReadError>,
) -> (OwnerReadErrorCode, &'static str) {
    let error = outcome.err().expect("reader must refuse");
    (error.code(), error.field())
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec()).unwrap()
}

// ---------------------------------------------------------------------------
// history-requirement/v1
// ---------------------------------------------------------------------------

// Trace: TC-090, FR-018-AC-1
#[test]
fn requirement_round_trips_through_derive_and_read() {
    let report = requirement_report();
    let limits = OwnerLimits::default();
    let owner = requirement::derive(&report, limits).unwrap();
    assert_eq!(owner.value(), &report);
    assert_eq!(owner.usage().history_span, report.required_positions);
    assert_eq!(owner.usage().wire_bytes, owner.bytes().len());

    let read = requirement::read(owner.bytes(), &report, limits).unwrap();
    assert_eq!(read.report(), &report);
    assert_eq!(read.bytes(), owner.bytes());
    assert_eq!(read.usage().history_span, 1);
    assert_eq!(read.usage().wire_bytes, owner.bytes().len());
    assert!(read.usage().depth >= 1);
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn requirement_reader_refuses_noncanonical_and_malformed_bytes() {
    let report = requirement_report();
    let limits = OwnerLimits::default();
    let canonical = requirement::derive(&report, limits)
        .unwrap()
        .bytes()
        .to_vec();
    let body = text(&canonical);
    let refused = |bytes: &[u8]| code_of(requirement::read(bytes, &report, limits));

    // Insignificant whitespace and a trailing newline parse but are not canonical.
    assert_eq!(
        refused(format!("{body}\n").as_bytes()),
        (OwnerReadErrorCode::NonCanonical, "document")
    );
    assert_eq!(
        refused(body.replacen(',', ", ", 1).as_bytes()),
        (OwnerReadErrorCode::NonCanonical, "document")
    );
    // Reordered members: re-emit through a key-sorted map.
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    let reordered = serde_json::to_string(&value).unwrap();
    assert_ne!(reordered, body, "fixture must actually reorder members");
    assert_eq!(
        refused(reordered.as_bytes()),
        (OwnerReadErrorCode::NonCanonical, "document")
    );

    // Trailing non-whitespace data is not one document.
    assert_eq!(
        refused(format!("{body}x").as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "trailingData")
    );
    // Unknown, duplicate and missing members.
    assert_eq!(
        refused(body.replacen('{', "{\"extra\":1,", 1).as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    assert_eq!(
        refused(body.replacen('{', "{\"unit\":\"positions\",", 1).as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    assert_eq!(
        refused(body.replace(",\"unit\":\"positions\"", "").as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    assert_eq!(refused(b""), (OwnerReadErrorCode::InvalidJson, "document"));
    assert_eq!(
        refused(&[0xff, 0xfe]),
        (OwnerReadErrorCode::InvalidUtf8, "document")
    );
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn requirement_reader_refuses_wrong_schema_and_stale_identity() {
    let report = requirement_report();
    let limits = OwnerLimits::default();
    let canonical = requirement::derive(&report, limits)
        .unwrap()
        .bytes()
        .to_vec();
    let body = text(&canonical);
    let refused = |bytes: &[u8]| code_of(requirement::read(bytes, &report, limits));

    let wrong_schema = body.replace("history-requirement/v1", "history-requirement/v2");
    assert_ne!(wrong_schema, body);
    assert_eq!(
        refused(wrong_schema.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    let other_contract = body.replace("history-requirement/v1", "past-evaluation/v1");
    assert_ne!(other_contract, body);
    assert_eq!(
        refused(other_contract.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );

    // A well-formed but different formula digest is a stale identity: the bytes
    // are valid and canonical, so the independently supplied report disagrees.
    let stale = body.replace(&report.formula_sha256, &"0".repeat(64));
    assert_ne!(stale, body);
    assert_eq!(
        refused(stale.as_bytes()),
        (OwnerReadErrorCode::ExpectedMismatch, "expected")
    );
    // A malformed digest never reaches the comparison.
    let malformed = body.replace(&report.formula_sha256, "NOT-A-DIGEST");
    assert_eq!(
        refused(malformed.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    // A closed unit that the reader does not recognise is refused on the wire.
    let unit = body.replace("\"positions\"", "\"seconds\"");
    assert_eq!(
        refused(unit.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );

    // Valid canonical bytes against a different expected report.
    let mut other = report.clone();
    other.required_positions += 1;
    assert_eq!(
        code_of(requirement::read(&canonical, &other, limits)),
        (OwnerReadErrorCode::ExpectedMismatch, "expected")
    );
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn requirement_producer_refuses_an_invalid_report() {
    let mut report = requirement_report();
    report.unit = "seconds".to_owned();
    assert_eq!(
        code_of(requirement::derive(&report, OwnerLimits::default())),
        (OwnerReadErrorCode::InvalidCombination, "requirement")
    );
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn requirement_ceilings_are_enforced_on_read_and_derive() {
    let report = requirement_report();
    let canonical = requirement::derive(&report, OwnerLimits::default())
        .unwrap()
        .bytes()
        .to_vec();

    // The report requires one position of history; a ceiling of zero refuses it,
    // and the refusal reports the work observed.
    let span = OwnerLimits {
        max_history_span: 0,
        ..OwnerLimits::default()
    };
    let error = requirement::derive(&report, span).err().unwrap();
    assert_eq!(
        (error.code(), error.field()),
        (OwnerReadErrorCode::ResourceIncomplete, "historySpan")
    );
    assert_eq!(error.usage().history_span, report.required_positions);
    assert_eq!(
        code_of(requirement::read(&canonical, &report, span)),
        (OwnerReadErrorCode::ResourceIncomplete, "historySpan")
    );

    // The ceiling is inclusive: exactly the required span is accepted.
    let exact = OwnerLimits {
        max_history_span: report.required_positions,
        ..OwnerLimits::default()
    };
    assert!(requirement::read(&canonical, &report, exact).is_ok());

    // Lexical ceilings fire before the document is decoded.
    let input = OwnerLimits {
        max_input_bytes: canonical.len() - 1,
        ..OwnerLimits::default()
    };
    assert!(requirement::read(
        &canonical,
        &report,
        OwnerLimits {
            max_input_bytes: canonical.len(),
            ..OwnerLimits::default()
        }
    )
    .is_ok());
    assert_eq!(
        code_of(requirement::read(&canonical, &report, input)),
        (OwnerReadErrorCode::ResourceIncomplete, "inputBytes")
    );
    let output = OwnerLimits {
        max_output_bytes: canonical.len() - 1,
        ..OwnerLimits::default()
    };
    assert!(requirement::derive(
        &report,
        OwnerLimits {
            max_output_bytes: canonical.len(),
            ..OwnerLimits::default()
        }
    )
    .is_ok());
    assert_eq!(
        code_of(requirement::derive(&report, output)),
        (OwnerReadErrorCode::ResourceIncomplete, "outputBytes")
    );
    let depth = OwnerLimits {
        max_depth: 0,
        ..OwnerLimits::default()
    };
    assert_eq!(
        code_of(requirement::read(&canonical, &report, depth)),
        (OwnerReadErrorCode::ResourceIncomplete, "depth")
    );
    let string = OwnerLimits {
        max_string_bytes: 1,
        ..OwnerLimits::default()
    };
    assert_eq!(
        code_of(requirement::read(&canonical, &report, string)),
        (OwnerReadErrorCode::ResourceIncomplete, "stringBytes")
    );
    let visited = OwnerLimits {
        max_visited_fields: 1,
        ..OwnerLimits::default()
    };
    assert_eq!(
        code_of(requirement::read(&canonical, &report, visited)),
        (OwnerReadErrorCode::ResourceIncomplete, "visitedFields")
    );
}

// ---------------------------------------------------------------------------
// past-evaluation/v1
// ---------------------------------------------------------------------------

// Trace: TC-090, FR-018-AC-1
#[test]
fn result_round_trips_through_derive_and_read() {
    let report = result_report();
    let limits = OwnerLimits::default();
    let owner = result::derive(&report, limits).unwrap();
    assert_eq!(owner.value(), &report);

    let read = result::read(owner.bytes(), &report, limits).unwrap();
    assert_eq!(read.report(), &report);
    assert_eq!(read.bytes(), owner.bytes());
    let usage = read.usage();
    assert_eq!(usage.positions, report.stats.input_positions);
    assert_eq!(usage.history_span, report.required_history);
    assert_eq!(usage.evaluation_steps, report.stats.steps);
    assert_eq!(usage.recursion_depth, report.stats.max_recursion_depth);
    assert_eq!(usage.wire_bytes, owner.bytes().len());
    assert_eq!(owner.usage(), usage);
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn result_reader_refuses_noncanonical_and_malformed_bytes() {
    let report = result_report();
    let limits = OwnerLimits::default();
    let canonical = result::derive(&report, limits).unwrap().bytes().to_vec();
    let body = text(&canonical);
    let refused = |bytes: &[u8]| code_of(result::read(bytes, &report, limits));

    assert_eq!(
        refused(format!("{body}\n").as_bytes()),
        (OwnerReadErrorCode::NonCanonical, "document")
    );
    let value: serde_json::Value = serde_json::from_str(&body).unwrap();
    let reordered = serde_json::to_string(&value).unwrap();
    assert_ne!(reordered, body, "fixture must actually reorder members");
    assert_eq!(
        refused(reordered.as_bytes()),
        (OwnerReadErrorCode::NonCanonical, "document")
    );
    assert_eq!(
        refused(format!("{body}{{}}").as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "trailingData")
    );
    assert_eq!(
        refused(body.replacen('{', "{\"extra\":1,", 1).as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    assert_eq!(
        refused(body.replacen('{', "{\"verdict\":true,", 1).as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    assert_eq!(
        refused(body.replace(",\"verdict\":true", "").as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    assert_eq!(
        refused(&[0xc3, 0x28]),
        (OwnerReadErrorCode::InvalidUtf8, "document")
    );
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn result_reader_refuses_wrong_schema_and_stale_digest() {
    let report = result_report();
    let limits = OwnerLimits::default();
    let canonical = result::derive(&report, limits).unwrap().bytes().to_vec();
    let body = text(&canonical);
    let refused = |bytes: &[u8]| code_of(result::read(bytes, &report, limits));

    let wrong_schema = body.replace("past-evaluation/v1", "past-evaluation/v2");
    assert_ne!(wrong_schema, body);
    assert_eq!(
        refused(wrong_schema.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );

    // Flipping the verdict without recomputing the digest is a stale digest; the
    // report type refuses to deserialize it.
    assert!(report.verdict);
    let stale = body.replace("\"verdict\":true", "\"verdict\":false");
    assert_ne!(stale, body);
    assert_eq!(
        refused(stale.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    // A syntactically malformed digest is refused the same way.
    let malformed = body.replace(&report.result_sha256, "zz");
    assert_eq!(
        refused(malformed.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );
    // A well-formed digest that does not match the content is stale too.
    let other_digest = body.replace(&report.result_sha256, &"0".repeat(64));
    assert_eq!(
        refused(other_digest.as_bytes()),
        (OwnerReadErrorCode::InvalidJson, "document")
    );

    // The producer refuses the same stale state with InvalidCombination.
    let mut tampered = report.clone();
    tampered.verdict = !tampered.verdict;
    assert_eq!(
        code_of(result::derive(&tampered, limits)),
        (OwnerReadErrorCode::InvalidCombination, "pastResult")
    );

    // Valid canonical bytes against a different expected report.
    let mut other = report.clone();
    other.anchor = 0;
    assert_eq!(
        code_of(result::read(&canonical, &other, limits)),
        (OwnerReadErrorCode::ExpectedMismatch, "expected")
    );
}

// Trace: TC-090, FR-018-AC-2
#[test]
fn result_ceilings_are_enforced_per_dimension_and_inclusively() {
    let report = result_report();
    let canonical = result::derive(&report, OwnerLimits::default())
        .unwrap()
        .bytes()
        .to_vec();
    let at_report = OwnerLimits {
        max_positions: report.stats.input_positions,
        max_history_span: report.required_history,
        max_evaluation_steps: report.stats.steps,
        max_recursion_depth: report.stats.max_recursion_depth,
        ..OwnerLimits::default()
    };
    assert!(result::read(&canonical, &report, at_report).is_ok());
    assert!(result::derive(&report, at_report).is_ok());

    let cases = [
        OwnerLimits {
            max_positions: report.stats.input_positions - 1,
            ..at_report
        },
        OwnerLimits {
            max_history_span: report.required_history - 1,
            ..at_report
        },
        OwnerLimits {
            max_evaluation_steps: report.stats.steps - 1,
            ..at_report
        },
        OwnerLimits {
            max_recursion_depth: report.stats.max_recursion_depth - 1,
            ..at_report
        },
    ];
    for limits in cases {
        assert_eq!(
            code_of(result::read(&canonical, &report, limits)),
            (OwnerReadErrorCode::ResourceIncomplete, "pastResultLimits")
        );
        let error = result::derive(&report, limits).err().unwrap();
        assert_eq!(
            (error.code(), error.field()),
            (OwnerReadErrorCode::ResourceIncomplete, "pastResultLimits")
        );
        assert_eq!(
            error.usage(),
            OwnerUsage {
                positions: report.stats.input_positions,
                history_span: report.required_history,
                evaluation_steps: report.stats.steps,
                recursion_depth: report.stats.max_recursion_depth,
                ..OwnerUsage::default()
            }
        );
    }

    let input = OwnerLimits {
        max_input_bytes: canonical.len() - 1,
        ..OwnerLimits::default()
    };
    assert!(result::read(
        &canonical,
        &report,
        OwnerLimits {
            max_input_bytes: canonical.len(),
            ..OwnerLimits::default()
        }
    )
    .is_ok());
    assert_eq!(
        code_of(result::read(&canonical, &report, input)),
        (OwnerReadErrorCode::ResourceIncomplete, "inputBytes")
    );
    let output = OwnerLimits {
        max_output_bytes: canonical.len() - 1,
        ..OwnerLimits::default()
    };
    assert!(result::derive(
        &report,
        OwnerLimits {
            max_output_bytes: canonical.len(),
            ..OwnerLimits::default()
        }
    )
    .is_ok());
    assert_eq!(
        code_of(result::derive(&report, output)),
        (OwnerReadErrorCode::ResourceIncomplete, "outputBytes")
    );
}
