---
id: Task-014
title: "Integrate shared intake and receipts"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/quoin]
evidence_method: integration-process-spawn-and-negative-test
github_issue: ix://agent-ix/tl-mltl/issues/57
resume_conditions: [ix://agent-ix/tl-mltl/issues/39, ix://agent-ix/tl-mltl/Task-010, ix://agent-ix/tl-mltl/Task-011, ix://agent-ix/tl-mltl/Task-012, ix://agent-ix/tl-mltl/Task-013, ix://agent-ix/quoin/issues/363, ix://agent-ix/quoin/issues/364]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/39
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-010
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-011
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-012
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-013
    type: depends_on
  - target: ix://agent-ix/quoin/issues/363
    type: depends_on
  - target: ix://agent-ix/quoin/issues/364
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-015
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-067
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-068
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-071
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-072
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-073
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-074
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-075
    type: verifies
---

# Task-014: Integrate shared intake and receipts

## Scope

Wrap admitted native records in exact active-plan MeasurementCollection input,
preserve domain/shared states and artifacts losslessly, and retain receipts.

## Subtasks

- [ ] Validate exact source, static-fact, environment, and artifact bindings.
- [ ] Exercise safe paths, finite caps, state mapping, and corrupt-input refusal.
- [ ] Prove Quire/Quoin do not execute producers or make authority decisions.

## Deliverables

- Lossless shared collection fixtures and receipts for admitted domains.
- Executing requirement-tagged tests for TC-067, TC-068, and TC-071 through TC-075.

## Notes

- Wait for accepted Quoin #363 attachment and #364 truthful build-profile
  support where needed.
