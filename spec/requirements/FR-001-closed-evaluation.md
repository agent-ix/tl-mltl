---
id: FR-001
title: Evaluate closed finite traces
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-001
    type: implements
---

# FR-001: Evaluate closed finite traces

## Description

The library shall evaluate a validated bounded MLTL graph at time zero over a
complete finite trace using `mltl.closed-trace/v1`.

## Behavior

- Missing observations after the closed trace are false; Boolean constants
  remain constant.
- Future and Globally quantify every inclusive interval offset.
- Until requires a right-hand witness in the interval and the left operand at
  every earlier offset; Release is its Boolean dual.
- Malformed trace ordering, unsupported profiles, and resource exhaustion are
  errors rather than Boolean verdicts.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-001-AC-1 | All Boolean and temporal operators match declared primitive, nested, boundary, and short-trace oracles. | Test (TC-001, TC-002) |
| FR-001-AC-2 | Evaluation is deterministic and preserves formula, profile, trace, and referenced proposition identities. | Test (TC-003) |
| FR-001-AC-3 | Unsupported profiles and impractical work limits return explicit non-verdict errors. | Test (TC-004) |

## Dependencies

Depends on the exact tl-syntax Formula contract and constrains FR-002 and FR-003.
