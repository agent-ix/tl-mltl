---
id: Task-003
title: "Horizon and prefix behavior"
type: Task
status: done
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-001
    type: part_of
---
# Task-003: Horizon and prefix behavior

## Scope

Implement checked lookahead, delay, and buffer analysis plus pending-aware
prefix evaluation.

## Completion Evidence

Shared-corpus and unit tests cover checked arithmetic, deadlines, early
decisions, closure equivalence, and resource limits.
