---
id: FR-003
title: Evaluate incomplete prefixes without guessing
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
---

# FR-003: Evaluate incomplete prefixes without guessing

## Description

The library shall evaluate an open trace prefix using three-valued semantics
and shall emit a Boolean verdict only when every continuation preserves it.

## Behavior

- Unknown future observations produce `pending`, never a successful Boolean.
- Boolean and temporal operators may decide early when a decisive witness or
  counterexample is already present.
- Closing the trace evaluates missing observations using the declared
  closed-trace profile.
- Callers may evaluate an explicit discrete verdict time; the convenience
  entry point evaluates time zero and the report preserves the requested time.
- Each report names its prefix length and worst-case decision horizon.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-003-AC-1 | Insufficient prefixes remain pending while decisive witnesses and counterexamples resolve early. | Test (TC-008, TC-009) |
| FR-003-AC-2 | Closing any prefix yields the same verdict as closed evaluation of those bytes. | Test (TC-010) |
| FR-003-AC-3 | No pending, malformed, or resource-failure state is serialized as conclusive. | Test (TC-009) |

## Dependencies

Depends on FR-001 and FR-002 and supplies verdict timing to FR-005.
