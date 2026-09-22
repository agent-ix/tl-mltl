---
id: Task-022
title: "Integrate shared intake and receipts"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/quoin]
evidence_method: integration-process-spawn-and-negative-test
github_issue: ix://agent-ix/tl-mltl/issues/57
resume_conditions: [ix://agent-ix/tl-mltl/issues/39, ix://agent-ix/tl-mltl/Task-018, ix://agent-ix/tl-mltl/Task-019, ix://agent-ix/tl-mltl/Task-020, ix://agent-ix/tl-mltl/Task-021, ix://agent-ix/quoin/issues/363, ix://agent-ix/quoin/issues/364]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/39
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-018
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-019
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-020
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-021
    type: depends_on
  - target: ix://agent-ix/quoin/issues/363
    type: depends_on
  - target: ix://agent-ix/quoin/issues/364
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-024
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-121
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-122
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-125
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-126
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-127
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-128
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-129
    type: verifies
---

# Task-022: Integrate shared intake and receipts

## Scope

Wrap admitted native records in exact active-plan MeasurementCollection input,
preserve domain/shared states and artifacts losslessly, and retain receipts.

## Subtasks

- [ ] Validate exact source, static-fact, environment, and artifact bindings.
- [ ] Exercise safe paths, finite caps, state mapping, and corrupt-input refusal.
- [ ] Prove Quire/Quoin do not execute producers or make authority decisions.

## Deliverables

- Lossless shared collection fixtures and receipts for admitted domains.
- Executing requirement-tagged tests for TC-121, TC-122, and TC-125 through TC-129.

## Notes

- Wait for accepted Quoin #363 attachment and #364 truthful build-profile
  support where needed.
