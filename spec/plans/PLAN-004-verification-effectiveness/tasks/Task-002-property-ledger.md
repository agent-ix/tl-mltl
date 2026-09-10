---
id: Task-002
title: "Build the property ledger and grounded runner"
type: Task
status: blocked
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-011
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-050
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-051
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-052
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-053
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-054
    type: verifies
---

# Task-002: Build the property ledger and grounded runner

## Scope

Implement the closed criterion ledger and finite exhaustive/generated property
runner with independent oracles, deterministic identities, and limitations.

## Subtasks

- [ ] Freeze exact criterion, matrix-priority, domain, and classification facts.
- [ ] Implement exhaustive and seeded generated-domain execution records.
- [ ] Add oracle-independence, vacuity, discard, shrink, and promotion controls.

## Deliverables

- `tl-mltl.property-obligation-ledger/v1` and property-run records.
- Executing requirement-tagged tests for TC-050 through TC-054.

## Notes

- Generated inputs are not canonical corpus fixtures without FR-009 promotion.
