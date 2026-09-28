---
id: FR-046
title: Fuzz each TL crate at its real input boundary
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-045
    type: depends_on
---

# FR-046: Fuzz each TL crate at its real input boundary

## Description

When the nightly fuzz lane runs, each production TL crate shall exercise at
least one real public input boundary with a recorded finite execution budget
and retain any minimized failing input.

## Behavior

tl-syntax fuzzes strict document readers; tl-parse fuzzes dialect parsing and
format/parse stability; tl-rewrite fuzzes graph rewrite validation; tl-mltl
fuzzes evaluation and `c2po_map`. Inputs pass the actual reader and operation,
not a duplicate decoder. The lane records engine/toolchain, source pin,
starting corpus digest, seed, duration/execution budget, stop reason and
crash/replay identity. A crash is reproduced on the same revision before
promotion. No-crash is a bounded observation, not correctness.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-046-AC-1 | Each crate's target reaches the real reader and operation and records seed, pin, budget and actual executions. | Test (TC-181) |
| FR-046-AC-2 | Each failure is minimized and replayed; missing, stale or unconfirmed artifacts do not count as a clean campaign. | Test (TC-182) |

## Dependencies

FR-045 supplies strict reader and round-trip boundaries.
