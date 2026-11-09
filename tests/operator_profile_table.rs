//! Operator x profile table driven through the CLI binary and the wire boundary.
//!
//! The library sweep covers every operator; this table proves the entry points a
//! caller actually reaches agree with it. Origin-complete operators (`O`, `H`, `Y`,
//! `S`, `T`) are admitted by the wire but refused by the CLI, which names the typed
//! past-evaluation API, so each row also evaluates through that API. The derived
//! future operators `W` and `M` reach the CLI as their canonical lowering under the
//! closed and online-prefix profiles, including the refusal rows.

use std::{
    io::Write,
    process::{Command, Output, Stdio},
};

use serde_json::{json, Value};
use tl_mltl::{
    evaluate_past,
    past::history as past_history,
    wire::{command, OwnerLimits},
    CommandDocument, PastEvaluationLimits, PastEvaluationRelationInput, PositionHistoryDocument,
    PositionObservation,
};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

const CLOSED: &str = "mltl.closed-trace/v1";
const PREFIX: &str = "mltl.online-prefix/v1";
const ORIGIN: &str = "mltl.origin-complete-history/v1";

fn run(request: &Value) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_tl-mltl"))
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn request(operation: &str, profile: &str, nodes: &Value, trace: Value) -> Value {
    json!({
        "schemaVersion": "tl-mltl.command/v1",
        "operation": operation,
        "formulaId": "table-row",
        "formula": {
            "schema_version": "tl-syntax.formula/v1",
            "semantic_profile": profile,
            "root": nodes.as_array().unwrap().len() - 1,
            "nodes": nodes,
        },
        "trace": trace,
    })
}

fn trace(closed: bool, instants: &Value) -> Value {
    json!({
        "schemaVersion": "tl-mltl.trace/v1",
        "traceId": "table-trace",
        "closed": closed,
        "instants": instants,
    })
}

fn prop(id: u32) -> Value {
    json!({"kind": "proposition", "proposition": id})
}

fn interval() -> Value {
    json!({"start": 0, "end": 1})
}

/// Canonical lowering of `p W[0,1] q`: `(p U q) | G p`.
fn weak_until() -> Value {
    json!([
        prop(0),
        prop(1),
        {"kind": "until", "interval": interval(), "left": 0, "right": 1},
        {"kind": "globally", "interval": interval(), "operand": 0},
        {"kind": "or", "left": 2, "right": 3},
    ])
}

/// Canonical lowering of `p M[0,1] q`: `(p R q) & F p`.
fn strong_release() -> Value {
    json!([
        prop(0),
        prop(1),
        {"kind": "release", "interval": interval(), "left": 0, "right": 1},
        {"kind": "future", "interval": interval(), "operand": 0},
        {"kind": "and", "left": 2, "right": 3},
    ])
}

/// One past operator row: nodes over propositions 0 (`p`) and 1 (`q`).
fn past_rows() -> Vec<(&'static str, Value)> {
    vec![
        (
            "O",
            json!([prop(0), {"kind": "once", "interval": interval(), "operand": 0}]),
        ),
        (
            "H",
            json!([prop(0), {"kind": "historically", "interval": interval(), "operand": 0}]),
        ),
        (
            "Y",
            json!([prop(0), {"kind": "strong_previous", "operand": 0}]),
        ),
        (
            "S",
            json!([prop(0), prop(1),
                {"kind": "since", "interval": interval(), "left": 0, "right": 1}]),
        ),
        (
            "T",
            json!([prop(0), prop(1),
                {"kind": "triggered", "interval": interval(), "left": 0, "right": 1}]),
        ),
    ]
}

fn verdict(output: &Output) -> String {
    assert!(output.status.success(), "{}", stderr(output));
    let report: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(report["schemaVersion"], "tl-mltl.evaluation/v1");
    report["verdict"].as_str().unwrap().to_owned()
}

// Trace: TC-014, FR-005-AC-1, FR-016-AC-4
#[test]
fn derived_future_operators_evaluate_through_the_cli_under_both_profiles() {
    // (operator, profile, nodes, closed, instants, expected verdict)
    let rows = [
        ("W", CLOSED, weak_until(), true, json!([[0], [0]]), "true"),
        ("W", CLOSED, weak_until(), true, json!([[], []]), "false"),
        (
            "M",
            CLOSED,
            strong_release(),
            true,
            json!([[0, 1], [1]]),
            "true",
        ),
        (
            "M",
            CLOSED,
            strong_release(),
            true,
            json!([[1], [1]]),
            "false",
        ),
        ("W", PREFIX, weak_until(), true, json!([[0], [0]]), "true"),
        ("W", PREFIX, weak_until(), false, json!([[0]]), "pending"),
        ("W", PREFIX, weak_until(), false, json!([[]]), "false"),
        (
            "M",
            PREFIX,
            strong_release(),
            true,
            json!([[1], [1]]),
            "false",
        ),
        (
            "M",
            PREFIX,
            strong_release(),
            false,
            json!([[0, 1]]),
            "true",
        ),
        (
            "M",
            PREFIX,
            strong_release(),
            false,
            json!([[1]]),
            "pending",
        ),
    ];
    for (operator, profile, nodes, closed, instants, expected) in rows {
        let output = run(&request(
            "evaluate",
            profile,
            &nodes,
            trace(closed, &instants),
        ));
        assert_eq!(
            verdict(&output),
            expected,
            "{operator} {profile} closed={closed} {instants}"
        );
    }
}

// Trace: TC-014, FR-005-AC-1, FR-016-AC-4
#[test]
fn derived_future_operators_are_refused_where_the_profile_requires_it() {
    for (operator, nodes) in [("W", weak_until()), ("M", strong_release())] {
        // A closed-trace formula over an open trace document.
        let open = run(&request(
            "evaluate",
            CLOSED,
            &nodes,
            trace(false, &json!([[0]])),
        ));
        assert_eq!(open.status.code(), Some(2), "{operator}");
        assert!(
            stderr(&open).contains("closed trace document"),
            "{operator}"
        );

        // Evaluation without a trace.
        let missing = run(&request("evaluate", PREFIX, &nodes, Value::Null));
        assert_eq!(missing.status.code(), Some(2), "{operator}");
        assert!(stderr(&missing).contains("requires trace"), "{operator}");

        // The command wire does not admit the origin-complete profile at all.
        let wrong_profile = run(&request("analyze", ORIGIN, &nodes, Value::Null));
        assert_eq!(wrong_profile.status.code(), Some(2), "{operator}");
        assert!(
            stderr(&wrong_profile).contains("does not admit semantic profile"),
            "{operator}: {}",
            stderr(&wrong_profile)
        );
    }
    // W and M are not wire node kinds: only their lowering is.
    for kind in ["weak_until", "strong_release"] {
        let nodes = json!([prop(0), prop(1),
            {"kind": kind, "interval": interval(), "left": 0, "right": 1}]);
        let output = run(&request("analyze", CLOSED, &nodes, Value::Null));
        assert_eq!(output.status.code(), Some(2), "{kind}");
        assert!(stderr(&output).contains("unknown variant"), "{kind}");
    }
}

// Trace: TC-014, FR-005-AC-1, FR-016-AC-4
#[test]
fn derived_future_operators_round_trip_through_the_command_wire() {
    for nodes in [weak_until(), strong_release()] {
        for profile in [CLOSED, PREFIX] {
            let value = request(
                "evaluate",
                profile,
                &nodes,
                trace(true, &json!([[0, 1], [1]])),
            );
            let document: CommandDocument = serde_json::from_value(value).unwrap();
            let owned = command::derive(&document, OwnerLimits::default()).unwrap();
            let read =
                command::ValidatedCommand::from_json_bytes(owned.bytes(), OwnerLimits::default())
                    .unwrap();
            assert_eq!(read.document(), &document);
            assert_eq!(read.bytes(), owned.bytes());
        }
    }
}

/// History rows `(p, q)` at positions 0, 1, 2.
const HISTORY: [(bool, bool); 3] = [(true, false), (true, true), (false, true)];

fn history() -> PositionHistoryDocument {
    let observations = HISTORY
        .iter()
        .enumerate()
        .map(|(position, (p, q))| {
            let mut propositions = Vec::new();
            if *p {
                propositions.push(PropositionId(0));
            }
            if *q {
                propositions.push(PropositionId(1));
            }
            PositionObservation::new(u64::try_from(position).unwrap(), propositions, None)
        })
        .collect();
    PositionHistoryDocument::new(
        "history-a",
        1,
        0,
        2,
        Some(tl_mltl::ClockBinding::EventPosition),
        observations,
    )
    .unwrap()
}

fn typed_verdict(nodes: &[Node], anchor: u64) -> bool {
    let root = NodeId(u32::try_from(nodes.len() - 1).unwrap());
    let formula = Formula::new(SemanticProfile::OriginCompleteHistoryV1, root, nodes).unwrap();
    evaluate_past(
        formula,
        "table-row",
        &history(),
        anchor,
        "map-a",
        1,
        PastEvaluationRelationInput::Original,
        PastEvaluationLimits::default(),
    )
    .unwrap()
    .verdict
}

fn node(kind: NodeKind) -> Node {
    Node::new(kind)
}

fn typed_nodes(operator: &str) -> Vec<Node> {
    let p = || {
        node(NodeKind::Proposition {
            proposition: PropositionId(0),
        })
    };
    let q = || {
        node(NodeKind::Proposition {
            proposition: PropositionId(1),
        })
    };
    let window = Interval::new(0, 1).unwrap();
    let (left, right) = (NodeId(0), NodeId(1));
    match operator {
        "O" => vec![
            p(),
            node(NodeKind::Once {
                interval: window,
                operand: left,
            }),
        ],
        "H" => vec![
            p(),
            node(NodeKind::Historically {
                interval: window,
                operand: left,
            }),
        ],
        "Y" => vec![p(), node(NodeKind::StrongPrevious { operand: left })],
        "S" => vec![
            p(),
            q(),
            node(NodeKind::Since {
                interval: window,
                left,
                right,
            }),
        ],
        "T" => vec![
            p(),
            q(),
            node(NodeKind::Triggered {
                interval: window,
                left,
                right,
            }),
        ],
        other => panic!("unknown past operator {other}"),
    }
}

// Trace: TC-014, FR-005-AC-1
#[test]
fn origin_complete_operators_are_refused_by_cli_and_command_wire_but_typed_evaluable() {
    // (operator, anchor, expected verdict over HISTORY)
    let expected = [
        ("O", 2, true),
        ("O", 1, true),
        ("H", 2, false),
        ("H", 1, true),
        ("Y", 2, true),
        ("Y", 1, true),
        ("S", 2, true),
        ("S", 0, false),
        ("T", 2, true),
        ("T", 0, false),
    ];
    for (operator, nodes) in past_rows() {
        for operation in ["evaluate", "analyze", "map_c2po"] {
            let value = request(operation, ORIGIN, &nodes, trace(true, &json!([[0]])));

            // Command wire: refused at the strict read.
            let refused = command::ValidatedCommand::from_json_bytes(
                value.to_string().as_bytes(),
                OwnerLimits::default(),
            );
            assert!(refused.is_err(), "{operator} {operation}");

            // CLI: exit 2, no structured output, the profile named in the message.
            let output = run(&value);
            assert_eq!(output.status.code(), Some(2), "{operator} {operation}");
            assert!(output.stdout.is_empty(), "{operator} {operation}");
            assert!(
                stderr(&output).contains("does not admit semantic profile"),
                "{operator} {operation}: {}",
                stderr(&output)
            );
        }

        // Typed API: the verdict the CLI cannot produce.
        let typed = typed_nodes(operator);
        for (name, anchor, want) in expected {
            if name == operator {
                assert_eq!(typed_verdict(&typed, anchor), want, "{operator}@{anchor}");
            }
        }
    }
}

// Trace: TC-014, FR-005-AC-1
#[test]
fn origin_complete_history_round_trips_through_its_own_wire() {
    let original = history();
    let owned = past_history::derive(&original, OwnerLimits::default()).unwrap();
    let read = past_history::read(owned.bytes(), &original, OwnerLimits::default()).unwrap();
    assert_eq!(read.bytes(), owned.bytes());
}
