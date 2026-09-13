---
id: Task-017
title: "FR-008/FR-009 — syntax-owned canonical family"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-syntax
consumer_repositories: [agent-ix/tl-parse, agent-ix/tl-rewrite, agent-ix/tl-mltl]
evidence_method: integration-and-snapshot-test
github_issue: ix://agent-ix/tl-syntax/issues/44
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/issues/51]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-016
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-008
    type: references
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/TC-042
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-044
    type: verifies
---

# Task-017: FR-008/FR-009 — syntax-owned canonical family

## Scope

Publish one authoritative manifest for current Boolean/future and landed W/M
canonical graph cases without changing or restamping existing corpus bytes.

## Subtasks

- [ ] Bind each case to exact schema, semantic/operator profile, source
  revision, license, provenance, limitation, and digest fields.
- [ ] Pair W/M source notation with byte-identical primitive graphs.
- [ ] Add exact-digest consumer fixtures and hostile manifest controls.

## Deliverables

- Versioned tl-syntax family manifest and canonical cases.
- TC-042 and TC-044 replay vectors.

## Notes

- The text notation is internal representation, never a second authored source
  language.
- Unblocks Tasks 018 through 022.
