---
id: Task-012
title: "FR-009 — parser fixture family"
type: Task
status: blocked
track: B
priority: P1
owner_repository: agent-ix/tl-parse
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: round-trip-property-and-integration-test
github_issue: ix://agent-ix/tl-parse/issues/33
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/Task-010, ix://agent-ix/tl-mltl/Task-011]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-010
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-011
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/TC-098
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-099
    type: verifies
---

# Task-012: FR-009 — parser fixture family

## Scope

Publish the internal text parse/format/span family for current future/W/M
profiles under the campaign manifest contract.

## Subtasks

- [ ] Bind exact dialect, graph/profile, span, result/refusal, provenance, and
  digest identities.
- [ ] Cover canonical and malformed/profile-incompatible cases.
- [ ] Add mutation controls for stale owner pins and restamped bytes.

## Deliverables

- Versioned tl-parse family manifest and TC-098/TC-099 vectors.

## Notes

- The `tl-parse` text dialect is an internal representation of the same
  canonical `tl-syntax` graph, not a second source authority: a parse fixture
  binds the graph identity its text lowers to.
