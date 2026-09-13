---
id: Task-010
title: "Build the property ledger and grounded runner"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: property-integration-and-mutation-test
github_issue: ix://agent-ix/tl-mltl/issues/56
resume_conditions: [ix://agent-ix/tl-mltl/issues/39, ix://agent-ix/tl-mltl/Task-009]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/39
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-009
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

# Task-010: Build the property ledger and grounded runner

## Scope

Implement the closed criterion ledger and finite exhaustive/generated property
runner with independent oracles, deterministic identities, and limitations.

## Subtasks

- [ ] Freeze exact criterion, matrix-priority, domain, and classification facts.
- [ ] Implement exhaustive and seeded generated-domain execution records.
- [ ] Add oracle-independence, vacuity, discard, shrink, and promotion controls.

## Deliverables

- `tl-mltl.property-ledger/v1` and property-run records.
- Executing requirement-tagged tests for TC-050 through TC-054.

## Notes

- Generated inputs are not canonical corpus fixtures without FR-009 promotion.
