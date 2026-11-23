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
  - target: ix://agent-ix/tl-mltl/TC-300
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-301
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-302
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-303
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-309
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

- Needs the tl-syntax TL-428 change landed and released. First step of this task is the pin bump: bump the tl-syntax and tl-parse git pins in Cargo.toml (currently tl-syntax 9f7b1c8 and tl-parse 9607722) in a PR that adds the explicit `NodeKind` arms for the new LTLf kinds and changes nothing else (tl-syntax ADR-044 type shape, PLAN-051).
