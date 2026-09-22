---
id: Task-021
title: "Build the Kani candidate ledger and harnesses"
type: Task
status: blocked
track: D
priority: P1
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: bounded-analysis-integration-and-mutation-test
github_issue: ix://agent-ix/tl-mltl/issues/59
resume_conditions: [ix://agent-ix/tl-mltl/issues/39, ix://agent-ix/tl-mltl/Task-018, ix://agent-ix/quoin/issues/363, ix://agent-ix/quoin/issues/364]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/39
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-018
    type: depends_on
  - target: ix://agent-ix/quoin/issues/363
    type: depends_on
  - target: ix://agent-ix/quoin/issues/364
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-023
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-119
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-120
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-121
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-123
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-124
    type: verifies
---

# Task-021: Build the Kani candidate ledger and harnesses

## Scope

Implement the digested proof-candidate ledger and narrow Kani harnesses whose
results remain bound to exact assumptions, domains, checks, and non-claims.

## Subtasks

- [ ] Classify every admitted trigger and retain successor/tombstone lineage.
- [ ] Implement exact cover, vacuity, supported-path, and unwind controls.
- [ ] Prove verifier-only configuration cannot alter the production subject.

## Deliverables

- Candidate-ledger and `tl-mltl.bounded-proof/v1` records.
- Executing requirement-tagged tests for the assigned matrix rows.

## Notes

- PR #35 is feasibility input only and supplies no current proof evidence.
