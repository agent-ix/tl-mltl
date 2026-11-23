---
id: Task-005
title: "Loss records, FRET MLTL import and attribution"
type: Task
status: blocked
track: D
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-000
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-091
    type: references
  - target: ix://agent-ix/tl-mltl/FR-087
    type: references
  - target: ix://agent-ix/tl-mltl/FR-089
    type: references
  - target: ix://agent-ix/tl-mltl/TC-240
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-241
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-244
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-247
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-248
    type: verifies
---

# Task-005: Loss records, FRET MLTL import and attribution

## Scope

LossRecord type, FRET MLTL reader, attribution fields, fixture manifest from retrieved FRET examples.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- loss-record schema, reader, fixtures

## Notes

- Fixture retrieval is DEFERRED; implement the loss kinds and bare-next cases first.
