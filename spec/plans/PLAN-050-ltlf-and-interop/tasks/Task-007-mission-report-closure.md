---
id: Task-007
title: "Mission-bounding report and closure refusal"
type: Task
status: blocked
track: D
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-005
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-090
    type: references
  - target: ix://agent-ix/tl-mltl/FR-094
    type: references
  - target: ix://agent-ix/tl-mltl/TC-318
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-319
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-327
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-328
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-329
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-333
    type: verifies

---

# Task-007: Mission-bounding report and closure refusal

## Scope

Report classes, MissionClosureRequired refusal, the closure report API, known-case table including Xs and Xw. The closure conversion and ClosurePolicy are built only if Peter decides so (ADR-045 Open decision).

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- report, refusal, known-case table

## Notes

- Peter decides whether the closure conversion itself is built (ADR-045); the recommendation is refusal and report only, in which case TC-333 is withdrawn. export_safety_monitor stays unchanged (FR-094).
