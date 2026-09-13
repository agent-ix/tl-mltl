---
id: Task-004
title: "FR-009 — rewrite fixture family"
type: Task
status: blocked
track: B
priority: P1
owner_repository: agent-ix/tl-rewrite
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: property-and-integration-test
github_issue: ix://agent-ix/tl-rewrite/issues/37
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/Task-001, ix://agent-ix/tl-mltl/Task-002]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/TC-044
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-045
    type: verifies
---

# Task-004: FR-009 — rewrite fixture family

## Scope

Publish rewrite-equivalence pairs and counterexamples under one tl-rewrite
owner manifest, consuming W/M only through canonical primitive graphs.

## Subtasks

- [ ] Bind profile, graph, rewrite, oracle, provenance, limitation, and digest
  identities.
- [ ] Retain refusal and counterexample cases without expected-value generation
  by the production rewriter.
- [ ] Add restamping, stale-profile, and unsupported-rewrite controls.

## Deliverables

- Versioned tl-rewrite family manifest and TC-044/TC-045 vectors.

## Notes

- This task creates no alternate W/M or past evaluator semantics.
