---
id: Task-016
title: "FR-008 — campaign schema and fail-closed census"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-syntax, agent-ix/tl-parse, agent-ix/tl-rewrite, agent-ix/tl-mltl, agent-ix/quire-contract-ir]
evidence_method: integration-and-property-test
github_issue: ix://agent-ix/tl-mltl/issues/51
resume_conditions: [ix://agent-ix/tl-mltl/issues/38]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-008
    type: references
  - target: ix://agent-ix/tl-mltl/TC-037
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-038
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-041
    type: verifies
---

# Task-016: FR-008 — campaign schema and fail-closed census

## Scope

Implement the Rust campaign/cell wire types, reviewed obligation registry,
exact cell/set digests, closed status model, and typed census refusals.

## Subtasks

- [ ] Add strict serde types and stable refusal codes for every closed field.
- [ ] Implement exact tuple/set preimages and ordered registry validation.
- [ ] Add property and negative controls for every census mutation and resource
  boundary.

## Deliverables

- Rust public API and documentation for `tl-mltl.corpus-campaign/v1`.
- TC-037, TC-038, and TC-041 trace-tagged tests.

## Notes

- Blocked until the exact PR #44 revision is human-accepted on issue #38.
- This task unblocks every fixture producer and campaign consumer.
