---
id: Task-001
title: Specification and parity controls
type: Task
status: done
track: Verification
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-005
    type: part_of
  - target: ix://agent-ix/tl-mltl/FR-016
    type: references
---

# Task-001: Specification and parity controls

## Scope

Advance the tl-syntax pin to the landed lowering, specify FR-016 and
TC-076 through TC-080, and implement the direct W/M reference with closed,
prefix, progress, horizon, resource, mutation, and no-derived-branch controls.

## Completion Evidence

`tests/future_parity.rs` implements TC-076 through TC-080, runs under
`make test`, and is bound by `make test-census`.
