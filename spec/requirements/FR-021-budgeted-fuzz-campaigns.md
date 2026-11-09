---
id: FR-021
title: "Run fuzz targets under multiple seeds with a fixed budget and a plateau stop"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-020
    type: depends_on
---

# FR-021: Run fuzz targets under multiple seeds with a fixed budget and a plateau stop

## Description

When fuzz targets are run, tl-mltl shall run every target under several seeds with
a fixed budget, stop a run when coverage plateaus or the budget ends, keep the
grown corpus, and turn every crash into a committed regression test, without
reading no-crash or a plateau as proof of correctness.

## Inputs

- The fuzz targets: `c2po_map`, `evaluator_differential`, `history_intake`,
  `wire_cli_decode`, `lasso_differential`, and `partial_valuation_monotone`.
- At least three distinct seeds per target and a fixed per-run budget, both
  recorded before the run.
- The checked-in seed corpus for each target.
- The nightly toolchain, cargo-fuzz and libFuzzer, and the sanitizer in use.

## Outputs

- A run record per target and seed with: target, seed, requested and observed
  duration, executions, coverage reached (edges and features), corpus size, the
  stop reason, and any crash.
- The grown corpus kept in the repository, and a regression test for each crash.
- A statement of what the run does not establish.

## Behavior

A target is a boundary that takes arbitrary bytes or a structured input built
from bytes. Differential targets (`evaluator_differential`, `lasso_differential`,
`partial_valuation_monotone`) compare production with `tl-oracle`; boundary
targets (`wire_cli_decode`, `history_intake`, `c2po_map`) check that a decoder
returns a typed error or a value that round-trips, and never panics.

Each run uses its seed and stops at the first of the time budget, a crash, or a
coverage plateau. A plateau is reached when no new coverage feature appears over
the stated trailing window. The run record states the window. A run that reaches
the budget while coverage still grows is recorded as growth observed, not as a
plateau. A run that ends from a tool error, a resource refusal, or cancellation is
recorded under that name, never as a pass. Time before the target starts, such as
build time, is recorded apart from execution time.

A crash, timeout, or out-of-memory input is minimised and committed to the
target's corpus with a test that replays it under `cargo test`, so the defect
stays covered without cargo-fuzz. The checked-in seeds are replayed the same way
and must reach both accepting and refusing paths.

The record reports coverage reached and its growth, not a coverage percentage,
because the reachable denominator is unknown. A plateau is a reason to review the
seeds or to look for a bounded analysis, not proof that undiscovered inputs are
safe. Fuzz discoveries are generated evidence and enter the canonical corpus only
under FR-009. The campaign never runs a target that needs Java, Node, Electron,
R2U2, or C2PO.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-021-AC-1 | Every target is run under at least three distinct recorded seeds with a fixed budget, and each run record carries the target, seed, requested and observed duration, executions, coverage, and corpus size. | Test (TC-109) |
| FR-021-AC-2 | A run stops at the first of its budget, a crash, or a coverage plateau over the stated window, and a run that reaches the budget while still growing is recorded as growth observed rather than plateau. | Test (TC-110, TC-111) |
| FR-021-AC-3 | Crash, timeout, out-of-memory, tool error, resource refusal, cancellation, and not-run outcomes are recorded as distinct states, and a crash is replayed from its committed input. | Test (TC-111, TC-112) |
| FR-021-AC-4 | Every crash becomes a committed regression test, the checked-in seeds replay under `cargo test` and reach accepting and refusing paths, and the campaign report names the source revision, tool versions, and seeds so a run can be repeated. | Test (TC-109, TC-112, TC-121) |
| FR-021-AC-5 | No-crash, duration, execution count, corpus growth, and plateau records remain bounded observations and never become correctness, release, qualification, or certification verdicts. | Test (TC-122) |

## Dependencies

Depends on FR-020 for the property laws that fuzzing complements, and on FR-009
for the generated and canonical boundary. The owner of the fuzzed entry point
owns its harness. A target that needs a foreign runtime is unavailable, not
silently substituted.
