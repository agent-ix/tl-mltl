---
id: Task-002
title: Exact-head gate and PR-time review
type: Task
status: in_progress
track: Verification
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-005
    type: part_of
  - target: ix://agent-ix/tl-mltl/FR-016
    type: references
---

# Task-002: Exact-head gate and PR-time review

## Scope

Run the full local gate, open the PR, run PR-time Rust review and gap analysis,
fix verified findings, and re-run the gate at the pushed head.

## Completion Evidence

`make ci` exits 0 at the exact pushed head after review fixes, and the PR
records the review and gate result.
