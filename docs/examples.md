# Examples

Every Rust example here compiles and passes against the crate. They use the default features and these imports:

```rust
use tl_mltl::*;
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};
```

## Evaluate a closed trace and an open prefix

The README example evaluates `F[1,2] p0`. This example uses `G[0,2] p0` under the online-prefix profile and shows how the verdict changes as instants arrive. It mirrors `tests/reference.rs`.

```rust
let nodes = [
    Node::new(NodeKind::Proposition { proposition: PropositionId(0) }),
    Node::new(NodeKind::Globally {
        interval: Interval::new(0, 2).unwrap(),
        operand: NodeId(0),
    }),
];
let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(1), &nodes).unwrap();
let limits = EvaluationLimits::default();
let p = PropositionId(0);

// Two instants where p holds: the window [0,2] is not fully observed.
let report = evaluate_prefix(formula, "g", &[vec![p], vec![p]], "t", false, limits).unwrap();
assert_eq!(report.verdict, TruthValue::Pending);

// One violation decides the formula, whatever comes next.
let report = evaluate_prefix(formula, "g", &[vec![p], vec![]], "t", false, limits).unwrap();
assert_eq!(report.verdict, TruthValue::False);
assert_eq!(report.observed_through, Some(1));

// Three instants cover the whole window.
let trace = [vec![p], vec![p], vec![p]];
let report = evaluate_prefix(formula, "g", &trace, "t", false, limits).unwrap();
assert_eq!(report.verdict, TruthValue::True);
```

## Horizon of nested operators

The upper bounds of nested operators add.

```rust
let nodes = [
    Node::new(NodeKind::Proposition { proposition: PropositionId(0) }),
    Node::new(NodeKind::Globally {
        interval: Interval::new(0, 2).unwrap(),
        operand: NodeId(0),
    }),
    Node::new(NodeKind::Future {
        interval: Interval::new(0, 1).unwrap(),
        operand: NodeId(1),
    }),
];
let formula = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(2), &nodes).unwrap();
let horizon = analyze_horizon(formula, "nested").unwrap();
assert_eq!(
    (horizon.lookahead, horizon.propagation_delay, horizon.required_buffer),
    (3, 3, 4)
);
```

## Refusals

Each refusal is a typed error. No error carries a verdict.

```rust
let nodes = [
    Node::new(NodeKind::Proposition { proposition: PropositionId(0) }),
    Node::new(NodeKind::Future {
        interval: Interval::new(0, 1).unwrap(),
        operand: NodeId(0),
    }),
];
let limits = EvaluationLimits::default();

// evaluate_closed implements the closed-trace profile only.
let online = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(1), &nodes).unwrap();
let error = evaluate_closed(online, "f", &[], "t", limits).unwrap_err();
assert!(matches!(error, EvaluationError::UnsupportedProfile { .. }));

// Propositions within an instant must be strictly increasing.
let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(1), &nodes).unwrap();
let trace = [vec![PropositionId(1), PropositionId(0)]];
let error = evaluate_closed(closed, "f", &trace, "t", limits).unwrap_err();
assert!(matches!(error, EvaluationError::TraceNotStrictlyOrdered { .. }));
```

## Map to C2PO and compare an external verdict

This maps `p0 U[1,2] p1`, evaluates it, and compares the result with a verdict recorded from an external run. It mirrors `tests/interop.rs`. The `ExternalVerdict` here is constructed by hand to show the shape. In practice its values come from a recorded R2U2 run such as `corpus/r2u2-v4.2/`.

```rust
let nodes = [
    Node::new(NodeKind::Proposition { proposition: PropositionId(0) }),
    Node::new(NodeKind::Proposition { proposition: PropositionId(1) }),
    Node::new(NodeKind::Until {
        interval: Interval::new(1, 2).unwrap(),
        left: NodeId(0),
        right: NodeId(1),
    }),
];
let formula = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(2), &nodes).unwrap();

let source = MappingSourceIdentity {
    revision: "example".to_owned(),
    state: MappingSourceState::Clean,
};
let manifest =
    map_to_c2po(formula, "until", br#"{"formula":"bytes"}"#, source, None, 1_000).unwrap();
assert_eq!(manifest.expression, "(p0 U[1,2] p1)");
assert_eq!(manifest.proposition_ids, [0, 1]);
assert!(manifest.external_tool.is_none()); // nothing was executed

let trace = vec![vec![], vec![PropositionId(0), PropositionId(1)], vec![PropositionId(1)]];
let reference =
    evaluate_prefix(formula, "until", &trace, "t", false, EvaluationLimits::default()).unwrap();
assert_eq!(reference.verdict, TruthValue::True);

let external = ExternalVerdict {
    schema_version: "tl-mltl.external-verdict/v1".to_owned(),
    tool: ToolIdentity { name: "r2u2".to_owned(), version: "4.2-release".to_owned() },
    formula_id: "until".to_owned(),
    trace_id: "t".to_owned(),
    status: ExternalStatus::Conclusive,
    value: Some(true),
    verdict_time: Some(0),
    detail: None,
};
assert_eq!(compare_external(&reference, external).status, ComparisonStatus::Agreement);
```

The closed-trace profile has no mapping. The same table under `ClosedTraceV1` returns `MappingError::UnsupportedProfile`.

## Evaluate a past-time formula

This evaluates `O[0,1] p0` ("p0 held now or one position ago") at two anchors in a position history. It mirrors `tests/past_history.rs`.

```rust
let nodes = [
    Node::new(NodeKind::Proposition { proposition: PropositionId(0) }),
    Node::new(NodeKind::Once {
        interval: Interval::new(0, 1).unwrap(),
        operand: NodeId(0),
    }),
];
let formula = Formula::new(SemanticProfile::OriginCompleteHistoryV1, NodeId(1), &nodes).unwrap();

let observations = vec![
    PositionObservation::new(0, vec![PropositionId(0)], None),
    PositionObservation::new(1, vec![], None),
    PositionObservation::new(2, vec![], None),
];
let history = PositionHistoryDocument::new(
    "history-a", 1, 0, 2, Some(ClockBinding::EventPosition), observations,
)
.unwrap();

// The formula reads at most 1 position back.
assert_eq!(analyze_required_history(formula, "once").unwrap().required_positions, 1);

let evaluate = |anchor| {
    evaluate_past(
        formula, "once", &history, anchor, "map-a", 1,
        PastEvaluationRelationInput::Original, PastEvaluationLimits::default(),
    )
    .unwrap()
    .verdict
};
assert!(evaluate(1));  // p0 held at position 0
assert!(!evaluate(2)); // positions 1 and 2 are both false
```

## Run the command line

```sh
cargo run --quiet --bin tl-mltl -- tests/fixtures/cli-requests/analyze.json
```

```json
{
  "corpusRevision": "tl-syntax-corpus/v1",
  "formulaId": "cli-analyze-v1",
  "formulaRoot": 1,
  "lookahead": 3,
  "propagationDelay": 3,
  "requiredBuffer": 4,
  "schemaVersion": "tl-mltl.horizon/v1",
  "semanticProfile": "mltl.online-prefix/v1",
  "unit": "discrete-instants"
}
```

```sh
cargo run --quiet --bin tl-mltl -- tests/fixtures/cli-requests/map-c2po.json
```

The `expression` field of the output is `(p0 U[1,2] p1)`. `sourceRevision` and `sourceState` come from the build. See [cli.md](cli.md).

## Replay the conformance producers

```sh
cargo run --quiet --example reference_conformance
cargo run --quiet --example r2u2_differential -- --manifest corpus/r2u2-v4.2/manifest.json
make cli-conformance
```

Each writes one JSON row per obligation. See [cli.md](cli.md#conformance-producers).
