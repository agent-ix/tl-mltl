---
id: Task-008
title: "FR-008/FR-009/FR-010 — native bridge successor rows"
type: Task
status: blocked
track: C
priority: P1
owner_repository: agent-ix/tl-mltl
producer_repository: agent-ix/quire-contract-ir
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: cross-repository-integration-and-digest-replay
github_issue: ix://agent-ix/tl-mltl/issues/55
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/Task-001, ix://agent-ix/quire-contract-ir/issues/63, ix://agent-ix/quire-contract-ir/issues/64]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-001
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/issues/63
    type: depends_on
  - target: ix://agent-ix/quire-contract-ir/issues/64
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-008
    type: references
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/FR-010
    type: references
  - target: ix://agent-ix/tl-mltl/TC-043
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-046
    type: verifies
---

# Task-008: FR-008/FR-009/FR-010 — native bridge successor rows

## Scope

Consume accepted native predicate/temporal owner families and move their exact
campaign rows from blocked to applicable through a successor manifest.

## Subtasks

- [ ] Pin accepted #63/#64 producer and TL profile/evaluator revisions.
- [ ] Preserve source, model, predicate, capture, clock, history, result, loss,
  and refusal identities through replay.
- [ ] Prove the transition does not erase or re-key the prior blocked rows.

## Deliverables

- Native correspondence consumer overlay and TC-043/TC-046 evidence.

## Notes

- Contract IR owns the native grammar and correspondence; this task consumes
  it and cannot invent a Boolean or FRETish fallback.
