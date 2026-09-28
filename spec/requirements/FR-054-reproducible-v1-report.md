---
id: FR-054
title: Emit one reproducible V1 verification report
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-043
    type: depends_on
---

# FR-054: Emit one reproducible V1 verification report

## Description

When a V1 verification run finishes, the campaign shall write one
machine-readable report that binds every executed lane to its actual source,
tool, input, result and limitation.

## Behavior

The report lists all four production revisions, tl-oracle revision,
feature/dependency pins, tool versions, seeds, corpus digests, domain
cardinalities, accepted/discarded counts, mutation populations, proof bounds,
coverage, performance and raw artifact paths/digests. It distinguishes
passed, failed, incomplete, blocked and not-run lanes. It is written from the
actual run outputs; a missing lane is not filled from a previous report or
exit code. Repeating deterministic lanes with identical inputs yields the
same semantic payload; volatile wall-clock fields are outside that payload.
No standalone ledger, intake wrapper or approval state is required.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-054-AC-1 | The report binds actual per-lane outputs and exact revisions, seeds, pins and digests, with no stale or missing result reported as passing. | Test (TC-195) |
| FR-054-AC-2 | Identical deterministic runs reproduce the same semantic payload and all non-conclusive populations stay distinguishable. | Test (TC-196) |

## Dependencies

FR-043 through FR-053 supply measured results; this requirement records them.
