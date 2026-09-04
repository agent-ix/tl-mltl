---
id: Task-002
title: Shared dependencies and contextual wire forms
type: Task
status: blocked
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

## Blocker

tl-syntax#15 and tl-rewrite#21 have reviewed published contracts but have not
both landed. No temporary pin, copied type, or implementation workaround is
permitted.
