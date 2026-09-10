---
id: Task-007
title: "Close the verification-effectiveness campaign"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/Task-006
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-011
    type: references
  - target: ix://agent-ix/tl-mltl/FR-012
    type: references
  - target: ix://agent-ix/tl-mltl/FR-013
    type: references
  - target: ix://agent-ix/tl-mltl/FR-014
    type: references
  - target: ix://agent-ix/tl-mltl/FR-015
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-050
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-075
    type: verifies
---

# Task-007: Close the verification-effectiveness campaign

## Scope

Run the complete exact-head local gate, reconcile matrix status with executing
tags, falsify controls by mutation, and obtain independent closing review.

## Subtasks

- [ ] Run every applicable TC-050 through TC-075 test and retain raw results.
- [ ] Mutate every population, identity, state, bound, and authority control.
- [ ] Re-run Quire/Quoin gates and close all high/medium review findings.

## Deliverables

- Exact-head local gate evidence, closing review, and gap analysis.

## Notes

- Hosted CI remains manual-only; human review remains the release authority.
