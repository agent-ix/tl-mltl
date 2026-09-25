//! Executable inspections of the five published owner boundaries and CI entry point.

use std::{fs, path::Path, process::Command};

use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};
use tl_mltl::{
    analyze_required_history, evaluate_past,
    past::{history, requirement, result},
    wire::{self, command, trace, OwnerLimits, OwnerReadErrorCode},
    ClockBinding, PastEvaluationLimits, PastEvaluationRelationInput, PositionHistoryDocument,
    PositionObservation,
};
use tl_syntax::{
    Formula, FormulaDocument, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile,
};

fn assert_schema(bytes: &[u8], digest: &str, expected_digest: &str, contract: &str) {
    assert_eq!(
        digest, expected_digest,
        "{contract} published digest changed"
    );
    assert_eq!(format!("{:x}", Sha256::digest(bytes)), digest, "{contract}");
    let schema: Value = serde_json::from_slice(bytes).unwrap();
    assert!(schema.is_object(), "{contract} must publish a JSON Schema");
}

fn assert_strict_reader<F>(contract: &str, bytes: &[u8], read: F)
where
    F: Fn(&[u8], OwnerLimits) -> Result<(), OwnerReadErrorCode>,
{
    let limits = OwnerLimits::owner_max();
    assert_eq!(read(bytes, limits), Ok(()), "{contract} canonical read");

    let mut whitespace = vec![b' '];
    whitespace.extend_from_slice(bytes);
    assert_eq!(
        read(&whitespace, limits),
        Err(OwnerReadErrorCode::NonCanonical),
        "{contract} leading whitespace"
    );

    let mut trailing = bytes.to_vec();
    trailing.extend_from_slice(b"{} ");
    assert_eq!(
        read(&trailing, limits),
        Err(OwnerReadErrorCode::InvalidJson),
        "{contract} trailing document"
    );

    let mut unknown: Value = serde_json::from_slice(bytes).unwrap();
    unknown["unexpectedOwnerField"] = json!(true);
    assert!(
        read(&serde_json::to_vec(&unknown).unwrap(), limits).is_err(),
        "{contract} unknown field"
    );

    let mut missing: Value = serde_json::from_slice(bytes).unwrap();
    missing.as_object_mut().unwrap().remove("schemaVersion");
    assert!(
        read(&serde_json::to_vec(&missing).unwrap(), limits).is_err(),
        "{contract} missing schemaVersion"
    );

    let reordered = serde_json::to_vec(&serde_json::from_slice::<Value>(bytes).unwrap()).unwrap();
    assert_ne!(
        reordered, bytes,
        "{contract} fixture needs a distinct field order"
    );
    assert_eq!(
        read(&reordered, limits),
        Err(OwnerReadErrorCode::NonCanonical),
        "{contract} reordered fields"
    );

    let mut stale: Value = serde_json::from_slice(bytes).unwrap();
    stale["schemaVersion"] = json!("tl-mltl.not-this-contract/v1");
    assert!(
        read(&serde_json::to_vec(&stale).unwrap(), limits).is_err(),
        "{contract} stale schema identity"
    );

    let mut duplicate = b"{\"schemaVersion\":\"tl-mltl.not-this-contract/v1\",".to_vec();
    duplicate.extend_from_slice(&bytes[1..]);
    assert!(
        read(&duplicate, limits).is_err(),
        "{contract} duplicate schemaVersion"
    );

    assert_eq!(
        read(
            bytes,
            OwnerLimits {
                max_input_bytes: bytes.len() - 1,
                ..limits
            }
        ),
        Err(OwnerReadErrorCode::ResourceIncomplete),
        "{contract} one-over input ceiling"
    );

    for (field, bounded) in [
        (
            "depth",
            OwnerLimits {
                max_depth: 0,
                ..limits
            },
        ),
        (
            "string",
            OwnerLimits {
                max_string_bytes: 1,
                ..limits
            },
        ),
        (
            "visited fields",
            OwnerLimits {
                max_visited_fields: 1,
                ..limits
            },
        ),
    ] {
        assert_eq!(
            read(bytes, bounded),
            Err(OwnerReadErrorCode::ResourceIncomplete),
            "{contract} {field} ceiling"
        );
    }
}

fn assert_output_ceiling<F>(contract: &str, bytes: &[u8], derive: F)
where
    F: Fn(OwnerLimits) -> Result<(), OwnerReadErrorCode>,
{
    assert_eq!(
        derive(OwnerLimits {
            max_output_bytes: bytes.len() - 1,
            ..OwnerLimits::owner_max()
        }),
        Err(OwnerReadErrorCode::ResourceIncomplete),
        "{contract} one-over output ceiling"
    );
}

// Trace: TC-090, FR-018-AC-1, FR-018-AC-2
#[test]
fn retained_owner_contracts_have_pinned_schemas_and_strict_readers() {
    let limits = OwnerLimits::owner_max();
    for (contract, bytes, digest, expected_digest) in [
        (
            trace::CONTRACT,
            trace::SCHEMA_BYTES,
            trace::SCHEMA_SHA256,
            "9c1020bb56cbd38ebc10405a39ccffc156908bb00f032de4c518626343989946",
        ),
        (
            command::CONTRACT,
            command::SCHEMA_BYTES,
            command::SCHEMA_SHA256,
            "71d10039c482a00885802611009196a1a0472d5af522fb5ff8b82ada8d68780c",
        ),
        (
            "tl-mltl.position-history/v1",
            history::SCHEMA_BYTES,
            history::SCHEMA_SHA256,
            "55cc048b6196436af5f85cf0f29539297ad68f246f81a5ea76fba3512ca61511",
        ),
        (
            "tl-mltl.history-requirement/v1",
            requirement::SCHEMA_BYTES,
            requirement::SCHEMA_SHA256,
            "4da9ad685f502369e369ecac38c0d23bb1e094c4c8982a70774ae9ae0c356bf9",
        ),
        (
            "tl-mltl.past-evaluation/v1",
            result::SCHEMA_BYTES,
            result::SCHEMA_SHA256,
            "a269301cf8150d57bb9f79e5784fbe29461ef37eb5eaeb3e571543eff2e2fa73",
        ),
    ] {
        assert_schema(bytes, digest, expected_digest, contract);
    }

    let trace_document = wire::TraceDocument {
        schema_version: wire::TraceSchemaVersion::V1,
        trace_id: "trace-1".to_owned(),
        closed: true,
        instants: vec![vec![PropositionId(0)]],
    };
    let trace_bytes = trace::derive(&trace_document, limits).unwrap();
    assert_output_ceiling(trace::CONTRACT, trace_bytes.bytes(), |limits| {
        trace::derive(&trace_document, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    assert_strict_reader(trace::CONTRACT, trace_bytes.bytes(), |bytes, limits| {
        trace::read(bytes, &trace_document, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    assert_eq!(
        trace::read(
            trace_bytes.bytes(),
            &trace_document,
            OwnerLimits {
                max_positions: 0,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );
    assert_eq!(
        trace::read(
            trace_bytes.bytes(),
            &trace_document,
            OwnerLimits {
                max_propositions: 0,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );

    let future_node = Node::new(NodeKind::Proposition {
        proposition: PropositionId(0),
    });
    let command_document = wire::CommandDocument {
        schema_version: wire::CommandSchemaVersion::V1,
        operation: wire::Operation::Evaluate,
        formula_id: "formula-1".to_owned(),
        formula: FormulaDocument::new(SemanticProfile::ClosedTraceV1, NodeId(0), vec![future_node])
            .unwrap(),
        trace: Some(trace_document),
    };
    let command_bytes = command::derive(&command_document, limits).unwrap();
    assert_output_ceiling(command::CONTRACT, command_bytes.bytes(), |limits| {
        command::derive(&command_document, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    assert_strict_reader(command::CONTRACT, command_bytes.bytes(), |bytes, limits| {
        command::read(bytes, &command_document, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    assert_eq!(
        command::read(
            command_bytes.bytes(),
            &command_document,
            OwnerLimits {
                max_formula_nodes: 0,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );
    assert_eq!(
        command::read(
            command_bytes.bytes(),
            &command_document,
            OwnerLimits {
                max_formula_depth: 0,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );

    let history_document = PositionHistoryDocument::new(
        "history-1",
        1,
        0,
        1,
        Some(ClockBinding::EventPosition),
        vec![
            PositionObservation::new(0, vec![PropositionId(0)], None),
            PositionObservation::new(1, vec![], None),
        ],
    )
    .unwrap();
    let history_bytes = history::derive(&history_document, limits).unwrap();
    let other_history = PositionHistoryDocument::new(
        "history-other",
        1,
        0,
        1,
        Some(ClockBinding::EventPosition),
        vec![
            PositionObservation::new(0, vec![PropositionId(0)], None),
            PositionObservation::new(1, vec![], None),
        ],
    )
    .unwrap();
    assert_eq!(
        history::read(history_bytes.bytes(), &other_history, limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );
    assert_output_ceiling("position-history", history_bytes.bytes(), |limits| {
        history::derive(&history_document, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    assert_strict_reader(
        "position-history",
        history_bytes.bytes(),
        |bytes, limits| {
            history::read(bytes, &history_document, limits)
                .map(|_| ())
                .map_err(|error| error.code())
        },
    );
    for bounded in [
        OwnerLimits {
            max_positions: 1,
            ..limits
        },
        OwnerLimits {
            max_propositions: 0,
            ..limits
        },
        OwnerLimits {
            max_history_span: 0,
            ..limits
        },
    ] {
        assert_eq!(
            history::read(history_bytes.bytes(), &history_document, bounded)
                .unwrap_err()
                .code(),
            OwnerReadErrorCode::ResourceIncomplete
        );
    }

    let past_node = [
        Node::new(NodeKind::Proposition {
            proposition: PropositionId(0),
        }),
        Node::new(NodeKind::Once {
            interval: Interval::new(0, 1).unwrap(),
            operand: NodeId(0),
        }),
    ];
    let past_formula = Formula::new(
        SemanticProfile::OriginCompleteHistoryV1,
        NodeId(1),
        &past_node,
    )
    .unwrap();
    let requirement_report = analyze_required_history(past_formula, "formula-1").unwrap();
    let requirement_bytes = requirement::derive(&requirement_report, limits).unwrap();
    let other_requirement = analyze_required_history(past_formula, "formula-other").unwrap();
    assert_eq!(
        requirement::read(requirement_bytes.bytes(), &other_requirement, limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );
    assert_output_ceiling("history-requirement", requirement_bytes.bytes(), |limits| {
        requirement::derive(&requirement_report, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    assert_strict_reader(
        "history-requirement",
        requirement_bytes.bytes(),
        |bytes, limits| {
            requirement::read(bytes, &requirement_report, limits)
                .map(|_| ())
                .map_err(|error| error.code())
        },
    );
    assert!(requirement_report.required_positions > 0);
    assert_eq!(
        requirement::read(
            requirement_bytes.bytes(),
            &requirement_report,
            OwnerLimits {
                max_history_span: requirement_report.required_positions - 1,
                ..limits
            }
        )
        .unwrap_err()
        .code(),
        OwnerReadErrorCode::ResourceIncomplete
    );

    let past_report = evaluate_past(
        past_formula,
        "formula-1",
        &history_document,
        1,
        "map-1",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    let past_bytes = result::derive(&past_report, limits).unwrap();
    let other_past_report = evaluate_past(
        past_formula,
        "formula-1",
        &history_document,
        1,
        "map-other",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(
        result::read(past_bytes.bytes(), &other_past_report, limits)
            .unwrap_err()
            .code(),
        OwnerReadErrorCode::ExpectedMismatch
    );
    assert_output_ceiling("past-evaluation", past_bytes.bytes(), |limits| {
        result::derive(&past_report, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    assert_strict_reader("past-evaluation", past_bytes.bytes(), |bytes, limits| {
        result::read(bytes, &past_report, limits)
            .map(|_| ())
            .map_err(|error| error.code())
    });
    for bounded in [
        OwnerLimits {
            max_positions: past_report.stats.input_positions - 1,
            ..limits
        },
        OwnerLimits {
            max_history_span: past_report.required_history - 1,
            ..limits
        },
        OwnerLimits {
            max_evaluation_steps: past_report.stats.steps - 1,
            ..limits
        },
        OwnerLimits {
            max_recursion_depth: past_report.stats.max_recursion_depth - 1,
            ..limits
        },
    ] {
        assert_eq!(
            result::read(past_bytes.bytes(), &past_report, bounded)
                .unwrap_err()
                .code(),
            OwnerReadErrorCode::ResourceIncomplete
        );
    }
}

// Trace: TC-090, FR-018-AC-3
#[test]
fn tl179_removal_preserved_existing_evaluator_cli_and_legacy_mapping_sources() {
    // TL-179 removed the separate QObs request/result/mapping modules at this
    // exact merge. A source comparison is the historical part of the claim;
    // the current behavior is exercised by reference, past-history, CLI, and
    // C2PO mapping tests rather than inferred from this diff alone.
    let output = Command::new("git")
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .args([
            "diff",
            "--exit-code",
            "8cfd85b25abd87ae5578ec1f2498ce5ec5413131",
            "6b4ba3cfd6e25f901490714ea9d8e8940e083e8d",
            "--",
            "src/future",
            "src/past",
            "src/main.rs",
            "src/mapping/legacy.rs",
            "src/wire/trace.rs",
            "src/wire/command.rs",
            "src/wire/common.rs",
            "src/wire/legacy.rs",
            "schemas/trace-v1.schema.json",
            "schemas/command-v1.schema.json",
            "schemas/position-history-v1.schema.json",
            "schemas/history-requirement-v1.schema.json",
            "schemas/past-evaluation-v1.schema.json",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "TL-179 changed a retained behavior path:\n{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

// Trace: TC-137, NFR-006-AC-8
#[test]
fn documented_and_hosted_full_gate_paths_invoke_the_guarded_entry_point() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = fs::read_to_string(root.join("README.md")).unwrap();
    let guidance = fs::read_to_string(root.join("CLAUDE.md")).unwrap();
    assert!(
        readme.lines().any(|line| line.trim() == "make guarded-ci"),
        "README must show an executable guarded full-gate command"
    );
    assert!(
        guidance
            .lines()
            .any(|line| line.trim().starts_with("make guarded-ci ")),
        "CLAUDE.md must list the guarded full-gate command"
    );

    let workflow_dir = root.join(".github/workflows");
    let mut guarded_runs = 0;
    for entry in fs::read_dir(workflow_dir).unwrap() {
        let path = entry.unwrap().path();
        if !matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("yml" | "yaml")
        ) {
            continue;
        }
        let workflow = fs::read_to_string(&path).unwrap();
        for line in workflow.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.is_empty() {
                continue;
            }
            assert!(
                !line.starts_with("run: make ci")
                    && line != "make ci"
                    && !line.starts_with("make ci "),
                "bare CI entry point in {}",
                path.display()
            );
            if line == "run: make guarded-ci" {
                guarded_runs += 1;
            }
        }
    }
    assert!(guarded_runs > 0, "no hosted full-gate invocation found");
}
