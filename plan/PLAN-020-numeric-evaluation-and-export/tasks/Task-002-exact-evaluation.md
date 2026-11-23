---
id: Task-002
title: "Exact evaluation and missing-observation rules"
type: Task
status: todo
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/FR-054
    type: references
  - target: ix://agent-ix/tl-mltl/FR-055
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-017
    type: references
  - target: ix://agent-ix/tl-mltl/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/TC-1573
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-1578
    type: verifies
---
# Task-002: Exact evaluation and missing-observation rules

## Scope

FR-054, FR-055: exact predicate truth, reduction to existing evaluators, three-valued atom source, aggregation equivalence run.

## Subtasks

- [ ] Predicate truth and wrapper report.
- [ ] Per-profile missing rules.
- [ ] Equivalence run over 567 traces.

## Deliverables

Evaluator module and tests TC-1573..TC-1580, TC-1599.

## Notes

Specification-only session: no code was written or run.
