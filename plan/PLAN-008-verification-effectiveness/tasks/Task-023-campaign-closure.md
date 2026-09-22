---
id: Task-023
title: "Close the verification-effectiveness campaign"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: complete-integration-mutation-and-assurance-replay
github_issue: ix://agent-ix/tl-mltl/issues/62
resume_conditions: [ix://agent-ix/tl-mltl/issues/39, ix://agent-ix/tl-mltl/Task-022]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/39
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-022
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-020
    type: references
  - target: ix://agent-ix/tl-mltl/FR-021
    type: references
  - target: ix://agent-ix/tl-mltl/FR-022
    type: references
  - target: ix://agent-ix/tl-mltl/FR-023
    type: references
  - target: ix://agent-ix/tl-mltl/FR-024
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-104
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-129
    type: verifies
---

# Task-023: Close the verification-effectiveness campaign

## Scope

Run the complete exact-head local gate, reconcile matrix status with executing
tags, falsify controls by mutation, and obtain independent closing review.

## Subtasks

- [ ] Run every applicable TC-104 through TC-129 test and retain raw results.
- [ ] Mutate every population, identity, state, bound, and authority control.
- [ ] Re-run Quire/Quoin gates and close all high/medium review findings.

## Deliverables

- Exact-head local gate evidence, closing review, and gap analysis.

## Notes

- Hosted CI remains manual-only; human review remains the release authority.
