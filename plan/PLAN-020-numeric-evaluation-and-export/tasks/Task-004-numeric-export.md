---
id: Task-004
title: "Numeric C2PO export, loss record and admission rule"
type: Task
status: todo
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/FR-056
    type: references
  - target: ix://agent-ix/tl-mltl/FR-057
    type: references
  - target: ix://agent-ix/tl-mltl/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/TC-1581
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-1583
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-1585
    type: verifies
---
# Task-004: Numeric C2PO export, loss record and admission rule

## Scope

FR-056, FR-057: map_numeric_to_c2po, loss record, partition table, admission test with synthetic profile.

## Subtasks

- [ ] Loss record schema.
- [ ] Admission test in exact rationals.
- [ ] Partition table extension.

## Deliverables

Export module and tests TC-1581..TC-1588.

## Notes

Specification-only session: no code was written or run.
