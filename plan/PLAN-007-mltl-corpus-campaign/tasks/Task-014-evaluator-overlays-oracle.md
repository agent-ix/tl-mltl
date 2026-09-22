---
id: Task-014
title: "FR-008/FR-009 — evaluator overlays and independent oracle"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: property-integration-and-mutation-test
github_issue: ix://agent-ix/tl-mltl/issues/52
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/Task-010, ix://agent-ix/tl-mltl/Task-011]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-010
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-011
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-008
    type: references
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/TC-093
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-094
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-099
    type: verifies
---

# Task-014: FR-008/FR-009 — evaluator overlays and independent oracle

## Scope

Publish tl-mltl evaluation/prefix/horizon/result/CLI overlays and independently
derived stored outcomes for current future/W/M cells.

## Subtasks

- [ ] Implement the isolated Rust oracle using public input types only.
- [ ] Cover each interval, trace, witness, counterexample, and progress class.
- [ ] Prove self-oracle, stale-oracle, generated-as-canonical, and resource
  mutations fail.

## Deliverables

- Versioned tl-mltl overlay family and TC-093/TC-094/TC-099 evidence.

## Notes

- The oracle cannot call or read any production or expected-result path it
  verifies.
- Unblocks Task-016.
