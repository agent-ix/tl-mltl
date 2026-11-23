---
id: Task-003
title: "C2PO import"
type: Task
status: blocked
track: B
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-000
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-086
    type: references
  - target: ix://agent-ix/tl-mltl/TC-310
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-311
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-312
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-332
    type: verifies

---

# Task-003: C2PO import

## Scope

interop::c2po reader, 12-formula identity test, 10 construct refusals.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- reader and typed error

## Notes

- Reconcile the 10 constructs with the C2PO 4.1.0 grammar first.
