---
id: Task-016
title: "FR-009/NFR-004 — manifest lifecycle and reproducible report"
type: Task
status: blocked
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-syntax, agent-ix/tl-parse, agent-ix/tl-rewrite, agent-ix/tl-mltl]
evidence_method: integration-mutation-and-replay-test
github_issue: ix://agent-ix/tl-mltl/issues/54
resume_conditions: [ix://agent-ix/tl-mltl/issues/38, ix://agent-ix/tl-mltl/Task-011, ix://agent-ix/tl-mltl/Task-012, ix://agent-ix/tl-mltl/Task-013, ix://agent-ix/tl-mltl/Task-014, ix://agent-ix/tl-mltl/Task-015]
relationships:
  - target: ix://agent-ix/tl-mltl/issues/38
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-011
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-012
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-013
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-014
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-015
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-004
    type: references
  - target: ix://agent-ix/tl-mltl/TC-098
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-099
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-101
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-102
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-103
    type: verifies
---

# Task-016: FR-009/NFR-004 — manifest lifecycle and reproducible report

## Scope

Integrate current owner families through the bounded loader, append-only
lifecycle, promotion/tombstone rules, deterministic replay, and MP-002 report.

## Subtasks

- [ ] Enforce exact manifest/corpus hashes, safe paths, and the closed limit
  profile before allocation or decode.
- [ ] Implement canonical/generated/retained lifecycle and mutation controls.
- [ ] Validate PLAN-007/ticket routing and emit byte-identical ordered raw
  populations through existing Quire/Quoin intake.

## Deliverables

- Bounded Rust loader and campaign report producer.
- TC-098, TC-099, TC-101, TC-102, and TC-103 evidence.

## Notes

- This task creates no local generic evidence, receipt, retention, or approval
  machinery.
