---
id: Task-018
title: "Build the property ledger and grounded runner"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: property-integration-and-mutation-test
github_issue: ix://agent-ix/tl-mltl/issues/56
resume_conditions: [ix://agent-ix/tl-mltl/issues/39, ix://agent-ix/tl-mltl/Task-017]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/39
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-017
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-020
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-104
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-105
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-106
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-107
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-108
    type: verifies
---

# Task-018: Build the property ledger and grounded runner

## Scope

Implement the closed criterion ledger and finite exhaustive/generated property
runner with independent oracles, deterministic identities, and limitations.

## Subtasks

- [ ] Freeze exact criterion, matrix-priority, domain, and classification facts.
- [ ] Implement exhaustive and seeded generated-domain execution records.
- [ ] Add oracle-independence, vacuity, discard, shrink, and promotion controls.

## Deliverables

- `tl-mltl.property-ledger/v1` and property-run records.
- Executing requirement-tagged tests for TC-104 through TC-108.

## Notes

- Generated inputs are not canonical corpus fixtures without FR-009 promotion.
