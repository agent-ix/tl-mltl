---
id: Task-006
title: "FRET C2PO, variable map and trace header"
type: Task
status: blocked
track: D
priority: P1
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-050
    type: part_of
  - target: ix://agent-ix/tl-mltl/Task-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-005
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-088
    type: references
  - target: ix://agent-ix/tl-mltl/TC-242
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-243
    type: verifies
---

# Task-006: FRET C2PO, variable map and trace header

## Scope

Bundle importer binding variables to catalog signals and carrying header fields.

## Subtasks

- [ ] Write the failing tests for the listed test cases first, then implement.
- [ ] Tag each test with its criterion id; run the focused tests, then the gate once at the end.

## Deliverables

- bundle importer

## Notes

- Needs Task-003 reader.
