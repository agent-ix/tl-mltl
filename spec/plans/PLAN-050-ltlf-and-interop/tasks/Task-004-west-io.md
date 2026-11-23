---
id: Task-004
title: "WEST import and export"
type: Task
status: blocked
track: C
priority: P1
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-000
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-092
    type: references
  - target: ix://agent-ix/tl-mltl/TC-249
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-250
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-251
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-252
    type: verifies
---

# Task-004: WEST import and export

## Scope

interop::west reader and writer, byte-copied corpus with licence and digest.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- reader, writer, corpus copy

## Notes

- Digest must match tl-rewrite's copy.
