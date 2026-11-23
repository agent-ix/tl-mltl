# Command line

The `tl-mltl` binary reads one JSON request and prints one JSON record. Build it with `cargo build --bin tl-mltl`.

```sh
tl-mltl REQUEST.json     # read the request from a file
tl-mltl -                # read the request from standard input
```

## Request

A request is a `tl-mltl.command/v1` document. Unknown fields are refused.

| Field | Meaning |
| --- | --- |
| `schemaVersion` | `tl-mltl.command/v1`. |
| `operation` | `evaluate`, `analyze` or `map_c2po`. |
| `formulaId` | A caller-chosen identity. |
| `formula` | A tl-syntax formula document (`tl-syntax.formula/v1` or `v2`). |
| `trace` | A trace document, required for `evaluate`, otherwise `null`. |

A trace document has the schema `tl-mltl.trace/v1`, a `traceId`, a `closed` flag and `instants`, a list of the true proposition ids at each instant.

```json
{
  "schemaVersion": "tl-mltl.command/v1",
  "operation": "evaluate",
  "formulaId": "cli-evaluate-closed-v1",
  "formula": {
    "schema_version": "tl-syntax.formula/v1",
    "semantic_profile": "mltl.closed-trace/v1",
    "root": 1,
    "nodes": [
      {"kind": "proposition", "proposition": 0},
      {"kind": "globally", "interval": {"start": 0, "end": 1}, "operand": 0}
    ]
  },
  "trace": {
    "schemaVersion": "tl-mltl.trace/v1",
    "traceId": "cli-evaluate-closed-trace-v1",
    "closed": true,
    "instants": [[0], [0]]
  }
}
```

This is `tests/fixtures/cli-requests/evaluate-closed.json`. Its output is an `EvaluationReport`:

```json
{
  "formulaId": "cli-evaluate-closed-v1",
  "formulaRoot": 1,
  "horizon": 1,
  "observedThrough": 1,
  "propositionIds": [0],
  "schemaVersion": "tl-mltl.evaluation/v1",
  "semanticProfile": "mltl.closed-trace/v1",
  "traceClosed": true,
  "traceId": "cli-evaluate-closed-trace-v1",
  "traceLength": 2,
  "verdict": "true",
  "verdictTime": 0
}
```

The program prints its fields in sorted order and indents them. The example above has the array on one line for brevity.

## Operations

| Operation | Calls | Output schema |
| --- | --- | --- |
| `evaluate` | `evaluate_closed` for a closed-trace formula, `evaluate_prefix` for an online-prefix formula. | `tl-mltl.evaluation/v1` |
| `analyze` | `analyze_horizon` | `tl-mltl.horizon/v1` |
| `map_c2po` | `map_to_c2po` with the build's source revision and state and a work limit of 100,000. | `tl-mltl.monitor-mapping/v1` |

The CLI evaluates at verdict time 0 with the default `EvaluationLimits`. A closed-trace formula requires `closed: true` in the trace. For an online-prefix formula the `closed` flag of the trace is passed to `evaluate_prefix`. Past-time and infinite-trace formulas are refused for `evaluate`, because those evaluators need the typed library API.

## Exit status

The command exits 0 after printing a record. It exits 2 after printing one line to standard error that begins with `tl-mltl:`. Refusals include an unreadable file, invalid JSON, an unknown field, an unknown command schema, an invalid formula, a missing trace, a closed-trace formula with an open trace, and an error from the selected operation.

## Conformance producers

The examples under `examples/` are producers. Each runs the real crate or binary over recorded inputs and writes one JSON row per obligation to standard output. A row has an `outcome` of `pass`, `fail`, `malformed` or `unsupported`, depending on the producer, and the identifiers of the obligation it checks.

| Command | What it checks |
| --- | --- |
| `make conformance` | Runs `reference_conformance`. Replays the shared corpus from tl-syntax through the evaluators and checks the three declared malformed fixtures. |
| `make differential` | Runs `r2u2_differential`. Replays the retained R2U2 exchange through `compare_external`. |
| `make cli-conformance` | Builds the binary, then runs `cli_conformance`. Drives `tl-mltl` over the requests in `tests/fixtures/cli-requests/manifest.json`, checks exit status, required output fields and the refusal message of each case, and runs each case twice to confirm identical output. |

## ci_guard

`ci_guard` is the second binary. `make guarded-ci` runs `ci_guard ci`, which refuses to start `make ci` when the Makefile or the `MAKEFLAGS` environment can hide a failing gate, runs it, and compares the gates that wrote a completion record with the declared set, independent of Make's exit code. `ci_guard record GATE` is the last step of each gate recipe.

## Evaluation record

The `evaluation_record` example turns one `evaluate` request file into one `tl-mltl.evaluation-record/v1` JSON record, for consumers that must not scrape printed text.

```sh
cargo run --example evaluation_record -- tests/fixtures/evaluation-records/pending.json
cargo run --example evaluation_record -- REQUEST.json --out RECORD.json
```

The request is the same `tl-mltl.command/v1` document the CLI reads, with `operation` set to `evaluate`. The record is written to standard output or to the `--out` file; a refusal prints one line beginning `evaluation_record:` to standard error and exits 2. The schema is `schemas/evaluation-record-v1.schema.json`.

| Field | Meaning |
| --- | --- |
| `schemaVersion` | `tl-mltl.evaluation-record/v1`. |
| `toolchain` | `name` (`tl-mltl`) and `version` of the build that evaluated. |
| `semanticProfile` | `ClosedTraceV1` or `OnlinePrefixV1`; `semanticProfileId` is its wire identity. |
| `formulaId` | The request's formula identity. |
| `inputSha256` | SHA-256 of the request file bytes. |
| `trace` | `traceId`, `closed` and `instants` (true proposition ids per instant), copied from the request. |
| `window` | Inclusive `[evaluationPosition, evaluationPosition + horizon]`: the instants the evaluation can read. It is not the formula's own interval. |
| `formulaInterval` | `start` and `end` of the root temporal operator's own interval, or null when the root is not temporal. |
| `evaluationPosition` | The formula-time index of the verdict (0). |
| `verdict` | `True`, `False` or `Pending`. |
| `observedThrough`, `horizon` | Last observed instant (or null) and the static decision horizon. |
| `witness` | A witness or counterexample when the evaluator reports one. The finite evaluators report none, so the field is null. |
