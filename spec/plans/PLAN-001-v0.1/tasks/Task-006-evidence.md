---
id: Task-006
title: "Exact-candidate evidence"
type: Task
status: done
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

The retained `da2c7704a534` record has a passing post-seal collection summary,
two passing sealed PGM-01 validations, and a checksum manifest that verifies
every artifact. The envelope itself remains non-self-attesting; the post-seal
summary records the external validation result.
