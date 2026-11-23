---
id: Task-002
title: "Reference corpus replay and direct evaluator"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-084
    type: references
  - target: ix://agent-ix/tl-mltl/TC-304
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-305
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-306
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-307
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-308
    type: verifies
---

# Task-002: Reference corpus replay and direct evaluator

## Scope

Corpus reader with provenance checks, replay of 40,820 rows, test-only direct evaluator with a mutation control.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- replay tests and provenance schema

## Notes

- TC-304 and TC-305 stay unmet until the DEFERRED generation happens.
