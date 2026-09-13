---
id: Task-006
title: "FR-010 — interoperability dispositions"
type: Task
status: blocked
track: B
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl, agent-ix/quire-contract-ir]
evidence_method: integration-snapshot-and-negative-test
github_issue: ix://agent-ix/tl-mltl/issues/53
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/Task-001, ix://agent-ix/tl-mltl/Task-002]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-010
    type: references
  - target: ix://agent-ix/tl-mltl/TC-046
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-048
    type: verifies
---

# Task-006: FR-010 — interoperability dispositions

## Scope

Implement the closed semantic-loss, target-availability, and comparison record
model over retained C2PO/R2U2 observations without executing a target runtime.

## Subtasks

- [ ] Encode the exact validity table and stable refusal/non-conclusive reasons.
- [ ] Bind every retained observation and comparison input identity.
- [ ] Reject all invalid combinations and overbroad qualification claims.

## Deliverables

- Strict Rust record types and TC-046/TC-048 evidence.

## Notes

- FRETish remains output-only under its quire-contract-ir owner.
- Unblocks Task-007.
