---
id: Task-009
title: "Verify the QObs C00 compatibility candidate"
type: Task
status: done
track: A
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/Task-008
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-019
    type: references
  - target: ix://agent-ix/tl-mltl/TC-085
    type: verifies
---

# Task-009: Verify the QObs C00 compatibility candidate

## Scope

Complete the required Rust review, remediate its findings, and run the focused
and repository gates that establish a candidate for the formal gap analysis.

## Subtasks

- [x] Complete Rust review and fix typed-result, fixture, and dependency-policy findings.
- [x] Complete risk/evidence readiness for the formal gap gate.
- [x] Run final exact-head gates and record the native Quoin fallback exception.

## Deliverables

- Rust `SpecReview` and readiness artifacts.
- Gate results including the exact native Quoin regression issue.
- A completed PLAN-006 candidate ready for the formal gap-analysis gate.

## Notes

- The npm Quoin path is allowed only for the receipt-exit regression tracked by
  agent-ix/quoin#543; normal spec validation and advisor work use native Quoin.
- Gap analysis and PR publication occur after this plan's task-completion gate;
  no PR merge is in scope.
