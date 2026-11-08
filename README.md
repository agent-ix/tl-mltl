# tl-mltl

[![Discord](https://img.shields.io/badge/Discord-Join%20us-5865F2?logo=discord&logoColor=white)](https://discord.gg/k8DVhuYBR2)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](#license)
[![Rust](https://img.shields.io/badge/rust-1.98.1%2B-orange?logo=rust)](Cargo.toml)

tl-mltl tells you whether a Mission-time Linear Temporal Logic (MLTL) formula holds
on a trace. It evaluates a closed trace to true or false and an open prefix to
true, false or pending, and reports how far ahead the formula looks and how many
instants a monitor must buffer.

It also evaluates bounded past-time formulas over a position history, maps formulas
to C2PO expressions for the R2U2 runtime monitor, and compares R2U2's verdict with
its own, classifying the pair as agreement or mismatch.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/hero-dark.svg">
  <img alt="A formula and a trace go into tl-mltl. evaluate_closed or evaluate_prefix returns a verdict, here true at time 0, and pending when the prefix does not decide it. analyze_horizon returns a lookahead of 2 and a buffer of 3 instants. map_to_c2po returns a manifest with the C2PO expression and SHA-256 digests of its input and output. C2PO and R2U2 run outside this crate, and compare_external classifies their verdict against the reference verdict, here 8 of 8 supported cases agree." src="docs/images/hero-light.svg">
</picture>

## Getting started

tl-mltl is not on crates.io. Depend on it through git:

```toml
[dependencies]
tl-mltl = { git = "https://github.com/agent-ix/tl-mltl" }
```

Add `rev = "<commit>"` to pin a commit. It needs the Rust version in `rust-version` in `Cargo.toml`, or newer.

Enable the `infinite-trace` feature for the infinite-trace evaluator: `features = ["infinite-trace"]`.

To install the command-line tool:

```sh
cargo install --git https://github.com/agent-ix/tl-mltl --bin tl-mltl
```

To build from a clone:

```sh
git clone https://github.com/agent-ix/tl-mltl.git
cd tl-mltl
cargo build --release
```

## Usage

This evaluates `F[1,2] p0` over a closed trace, evaluates it over an open
prefix, and reports its horizon.

```rust
use tl_mltl::{analyze_horizon, evaluate_closed, evaluate_prefix, EvaluationLimits, TruthValue};
use tl_syntax::{Formula, Interval, Node, NodeId, NodeKind, PropositionId, SemanticProfile};

let nodes = [
    Node::new(NodeKind::Proposition { proposition: PropositionId(0) }),
    Node::new(NodeKind::Future {
        interval: Interval::new(1, 2).unwrap(),
        operand: NodeId(0),
    }),
];
let limits = EvaluationLimits::default();

// A closed trace: p0 holds at instant 1, inside the window [1,2].
let closed = Formula::new(SemanticProfile::ClosedTraceV1, NodeId(1), &nodes).unwrap();
let trace = vec![vec![], vec![PropositionId(0)]];
let report = evaluate_closed(closed, "request-ready", &trace, "run-1", limits).unwrap();
assert_eq!(report.verdict, TruthValue::True);

// An open prefix with only instant 0 observed does not decide the formula yet.
let online = Formula::new(SemanticProfile::OnlinePrefixV1, NodeId(1), &nodes).unwrap();
let seen = vec![vec![]];
let open = evaluate_prefix(online, "request-ready", &seen, "run-2", false, limits).unwrap();
assert_eq!(open.verdict, TruthValue::Pending);

// The formula looks 2 instants ahead and needs a buffer of 3.
let horizon = analyze_horizon(closed, "request-ready").unwrap();
assert_eq!((horizon.lookahead, horizon.required_buffer), (2, 3));
```

More examples, including mapping, differential comparison, past-time evaluation
and the command-line tool, are in [docs/examples.md](docs/examples.md).

## A worked example

A deadline requirement, `F[0,3] p2`, is false on a recorded trace. [docs/scenario.md](docs/scenario.md) walks through finding out why, using tl-parse, tl-syntax, tl-rewrite, tl-mltl, R2U2, tl-oracle and quire-mltl on the retained R2U2 case `r2u2-future-deadline-v1`. It shows what to look at in each step and ends with a guide for results that disagree.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/scenario-sequence-dark.svg">
  <img alt="A deadline requirement fails on a recorded trace, and the engineer asks why. The engineer sends F[0,3] p2 to tl-parse and gets a formula document, binds p2 to signal s2 with tl-syntax, and sends the document to tl-rewrite, which reports Unchanged under the closed-trace profile. tl-mltl evaluates the formula and trace and returns false at time 0 with a buffer of 4 instants, and R2U2, which runs outside the crates, returns false at time 0 for the C2PO expression F[0,3](p2). tl-oracle evaluates the same formula and trace independently and returns false. quire-mltl maps the result to Value false. Numbered rings mark where to look at each step." src="docs/images/scenario-sequence-light.svg">
</picture>

## Reference

| Module | What it holds |
| --- | --- |
| `future` | `evaluate_closed`, `evaluate_closed_at`, `evaluate_prefix`, `evaluate_prefix_at`, `analyze_horizon`, their `_with_context` forms, `TruthValue`, `EvaluationLimits` and the report types. |
| `past` | `evaluate_past`, `analyze_required_history`, position histories, clock bindings and anchored results for `mltl.origin-complete-history/v1`. |
| `infinite` | Lasso-trace evaluation, finite-prefix safety refutation and safety export for `mltl.infinite-trace/v1` (feature `infinite-trace`). |
| `mapping` | `map_to_c2po`, `map_to_c2po_with_context` and `map_past_to_c2po`, with the mapping manifests. |
| `wire` | Strict readers and canonical writers for the owner documents `tl-mltl.command/v1` and `tl-mltl.trace/v1`, and the schemas of the past-time documents. |
| `clock` | Exact event-position and fixed-sample clock primitives. |
| `differential` (crate root) | `compare_external`, `compare_external_with_context` and the external-verdict and report types. |

| Binary | What it does |
| --- | --- |
| `tl-mltl` | Reads one `tl-mltl.command/v1` JSON request and prints an evaluation, horizon or mapping record. See [docs/cli.md](docs/cli.md). |

| Feature | Default | Adds |
| --- | ---: | --- |
| none | yes | Closed and prefix evaluation, horizon analysis, past-time evaluation, mapping and comparison. |
| `infinite-trace` | no | The `infinite` module. |

See [docs/api-reference.md](docs/api-reference.md).

## The TL stack

tl-mltl evaluates tl-syntax formulas over traces. tl-rewrite and quire-mltl build on it.

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/tl-stack-dark.svg">
  <img alt="The TL crates. tl-syntax defines the formula graph, profiles and signal catalog, and tl-parse, tl-oracle, tl-mltl, tl-rewrite and quire-mltl all depend on it. tl-rewrite depends on tl-mltl. quire-mltl depends on tl-mltl and quire-observation, and is the single bridge to Quire. Dashed lines are development-only: tl-mltl and tl-rewrite test against tl-oracle, and tl-rewrite tests against tl-parse." src="docs/images/tl-stack-light.svg">
</picture>

## Building and testing

```sh
make test             # cargo test --all-targets --all-features
make conformance      # replay the shared corpus through the evaluator
make differential     # replay the retained R2U2 4.2 exchange
make cli-conformance  # drive the built CLI over its declared requests
make guarded-ci       # every local gate, with the gate-set check
```

Run `make guarded-ci` for the full set.

### Nightly small-scope sweep

`make test` runs a fast subset of the small-scope comparison against the
independent `tl-oracle` (depth one over all leaves; depth two over one
proposition). `make sweep-full` runs the full sweep in release mode: depth one
with intervals up to `[0,4]` and traces up to length 6, depth two with intervals
up to `[0,2]` and traces up to length 6 (both exhaustive), and a seeded
depth-three sample (exhaustive depth three is not enumerable). Hosted CI is
manual-only, so the nightly is a local run: execute `make sweep-full` on a
schedule of your choosing (about 30 minutes on a laptop). Evidence JSON (tool
versions, seed, case counts, admitted/refused split, runtime, and any minimised
disagreement) is written to `$TL_MLTL_EVIDENCE_DIR`, default
`target/tmp/small-scope-evidence`. A disagreement is reported as a minimised
regression fixture to commit under `tests/fixtures/small-scope-regressions/`.

## Contributing

Read [CONTRIBUTING.md](CONTRIBUTING.md) and [CLA.md](CLA.md). The change history is in
[CHANGELOG.md](CHANGELOG.md).

## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or the
[MIT license](LICENSE-MIT) at your option.
