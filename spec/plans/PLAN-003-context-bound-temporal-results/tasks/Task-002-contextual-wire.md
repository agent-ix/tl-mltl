---
id: Task-002
title: Shared dependencies and contextual wire forms
type: Task
status: in_progress
track: Core
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-003
    type: part_of
  - target: ix://agent-ix/tl-mltl/FR-007
    type: references
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-rewrite/FR-007
    type: depends_on
---

# Task-002: Shared dependencies and contextual wire forms

## Scope

After both upstream changes land, adopt the exact reviewed tl-syntax revision,
capture v1 snapshots, and implement the closed contextual v2 forms and separated
content/request/result/comparison digest helpers.

## Completion Evidence

All v2 positive/negative construction and serde controls pass, v1 snapshots are
still exact, and the dependency/provenance check confirms the compiled pin and
published revision constant agree.

## Prerequisites

The reviewed shared syntax revision is compiled at `6ad7499`, and tl-rewrite
#21 landed as tl-rewrite main `1ccab45`. Contextual wire implementation may now
proceed against those reachable revisions. No copied type or temporary
implementation workaround is permitted.
