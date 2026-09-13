---
id: Task-007
title: "FR-009/NFR-004 — manifest lifecycle and reproducible report"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-syntax, agent-ix/tl-parse, agent-ix/tl-rewrite, agent-ix/tl-mltl, agent-ix/quire-contract-ir]
evidence_method: integration-mutation-and-replay-test
github_issue: ix://agent-ix/tl-mltl/issues/54
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/Task-002, ix://agent-ix/tl-mltl/Task-003, ix://agent-ix/tl-mltl/Task-004, ix://agent-ix/tl-mltl/Task-005, ix://agent-ix/tl-mltl/Task-006]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-004
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-005
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-006
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-004
    type: references
  - target: ix://agent-ix/tl-mltl/TC-044
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-045
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-047
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-048
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-049
    type: verifies
---

# Task-007: FR-009/NFR-004 — manifest lifecycle and reproducible report

## Scope

Integrate current owner families through the bounded loader, append-only
lifecycle, promotion/tombstone rules, deterministic replay, and MP-002 report.

## Subtasks

- [ ] Enforce exact manifest/corpus hashes, safe paths, and the closed limit
  profile before allocation or decode.
- [ ] Implement canonical/generated/retained lifecycle and mutation controls.
- [ ] Validate PLAN-006/ticket routing and emit byte-identical ordered raw
  populations through existing Quire/Quoin intake.

## Deliverables

- Bounded Rust loader and campaign report producer.
- TC-044, TC-045, TC-047, TC-048, and TC-049 evidence.

## Notes

- This task creates no local generic evidence, receipt, retention, or approval
  machinery.
