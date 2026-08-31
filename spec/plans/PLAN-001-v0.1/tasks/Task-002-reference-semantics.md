---
id: Task-002
title: "Reference semantics"
type: Task
status: done
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-001
    type: part_of
---
# Task-002: Reference semantics

## Scope

Implement closed and caller-selected-time evaluation for every tl-syntax v1
operator with explicit identities and limits.

## Completion Evidence

Requirement-tagged tests cover Boolean, unary temporal, Until, Release,
nesting, lower-bound, nonzero-time, and boundary behavior.
