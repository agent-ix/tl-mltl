---
id: Task-005
title: "Contract export and retained differential plumbing"
type: Task
status: todo
track: A
priority: P1
relationships:
  - target: ix://agent-ix/tl-mltl/FR-058
    type: references
  - target: ix://agent-ix/tl-mltl/FR-060
    type: references
  - target: ix://agent-ix/tl-mltl/Task-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-004
    type: depends_on
  - target: ix://agent-ix/tl-mltl/TC-1591
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-1596
    type: verifies
---
# Task-005: Contract export and retained differential plumbing

## Scope

FR-058, FR-060: decomposition export, replay target, campaign member, live target scaffold; fresh evidence generation remains DEFERRED.

## Subtasks

- [ ] numeric-differential replay and digest checks.
- [ ] Planned campaign member for the live target.
- [ ] Decomposition export of contracts.

## Deliverables

The refusal-until-pinned path for contracts and the decomposition export under a labelled
synthetic profile (TC-1596, TC-1597). The replay target, digest checks and campaign
member are scaffolding: no retained bytes exist and none is produced by this task, so
TC-1589, TC-1590, TC-1592 and TC-1598 stay OPEN/BLOCKED until a human runs R2U2 and
retains the exchange. This task never marks TL-423 AC1 or TL-425 AC2 met.

## Notes

Specification-only session: no code was written or run.
