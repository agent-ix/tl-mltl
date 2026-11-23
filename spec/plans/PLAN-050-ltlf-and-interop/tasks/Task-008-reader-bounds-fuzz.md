---
id: Task-008
title: "Reader bounds and fuzz targets"
type: Task
status: blocked
track: E
priority: P1
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-004
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-005
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-047
    type: references
  - target: ix://agent-ix/tl-mltl/TC-257
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-258
    type: verifies
---

# Task-008: Reader bounds and fuzz targets

## Scope

24 hostile cases and fuzz targets c2po_import, fret_mltl_import, west_import.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- hostile manifest, 3 fuzz targets, seeds

## Notes

- Fuzz campaigns beyond seeds are DEFERRED.
