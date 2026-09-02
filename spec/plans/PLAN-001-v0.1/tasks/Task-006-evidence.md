---
id: Task-006
title: "Exact-candidate evidence"
type: Task
status: in_progress
track: Evidence
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-001
    type: part_of
  - target: ix://agent-ix/tl-mltl/MP-001
    type: references
---
# Task-006: Exact-candidate evidence

## Scope

Retain the exact clean revision's local results, tool and dependency identities,
PGM-01 checks, and explicit limitations in a checksummed evidence record.

## Completion Evidence

The legacy records, including `da2c7704a534`, are explicitly retracted because
their collector inherited ambient execution controls and recorded banners rather
than executable identities. Completion now requires a clean qualification-v2
record collected under the allowlisted environment and bound to `tools.lock`.
