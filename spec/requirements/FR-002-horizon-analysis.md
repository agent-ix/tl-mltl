---
id: FR-002
title: Compute checked horizon and buffer bounds
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
---

# FR-002: Compute checked horizon and buffer bounds

## Description

The library shall compute maximum future lookahead, earliest worst-case
propagation delay, and required observation-buffer length for a validated
formula without wrapping arithmetic.

## Behavior

- Constants and propositions have horizon zero.
- Boolean operators inherit the maximum operand horizon.
- Each temporal operator adds its inclusive upper bound to the maximum relevant
  operand horizon.
- Buffer length is horizon plus one observation and is checked independently.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-002-AC-1 | Static horizons match shared-corpus expectations and prefix decision deadlines. | Test (TC-005, TC-008) |
| FR-002-AC-2 | Nested bounds use checked arithmetic and never wrap into a smaller claim. | Test (TC-006) |
| FR-002-AC-3 | Reports retain formula root, profile, corpus revision, and resource units. | Test (TC-007) |

## Dependencies

Depends on FR-001 operator semantics and enables FR-003 decision deadlines.
