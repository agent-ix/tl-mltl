---
id: Task-001
title: "LTLf evaluator and verdict naming"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-000
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-083
    type: references
  - target: ix://agent-ix/tl-mltl/FR-085
    type: references
  - target: ix://agent-ix/tl-mltl/TC-227
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-228
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-229
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-230
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-236
    type: verifies
---

# Task-001: LTLf evaluator and verdict naming

## Scope

evaluate_ltlf, typed refusals, unbounded horizon result, LTL3 accessor.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- evaluator, errors, mapping table document

## Notes

- Needs tl-syntax TL-428 landed.
