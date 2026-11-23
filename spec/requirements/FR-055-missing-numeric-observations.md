---
id: FR-055
title: Apply each profile's missing-observation rule to numeric predicates
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-054
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-042
    type: depends_on
---

# FR-055: Apply each profile's missing-observation rule to numeric predicates

## Description

When a numeric signal read by a predicate has no observation at a position, tl-mltl shall resolve the predicate by the rule of the selected profile and mark it unobserved, without fabricating a value.

## Behavior

A predicate is unobserved at a position if any signal it reads is absent from
that instant's `values`, or if the position lies after the observed end.

| Profile | Rule for an unobserved predicate | Grounding |
|---|---|---|
| `mltl.closed-trace/v1` | The predicate is false (padding), as missing Boolean observations after the closed trace are false (FR-001); the cell is counted in the report. | FR-001 |
| `mltl.online-prefix/v1` | The predicate is an unobserved atom whose value is `Pending`; the existing Kleene operators decide whether the formula is still decided. | FR-003 |
| `mltl.origin-complete-history/v1` | Inside the history from origin to anchor an unobserved predicate refuses the evaluation (`IncompleteHistory`, naming position and signal); positions before the origin keep the existing proposition-false extension. | FR-042 |

- Prefix evaluation uses a three-valued atom source; with no unobserved cells the
  results are identical to the existing prefix evaluator.
- Padding and pending are counted and reported (`unobservedCells`), so a reader
  can see how much of a verdict rests on absence.
- A negated predicate under closed padding is true where the predicate is
  unobserved; this follows the existing false-proposition rule and is documented
  in the report note, not hidden.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-055-AC-1 | Under the closed profile an unobserved predicate is false, counted, and a negation of it is true; a case set covers omission at an observed position and absence after the end. | unit-testing (TC-1578) |
| FR-055-AC-2 | Under the prefix profile an unobserved predicate yields `Pending` or a decided verdict exactly as the Kleene tables dictate, covering all nine `(and, or)` operand pairs involving it. | unit-testing (TC-1579) |
| FR-055-AC-3 | Under the origin-complete profile a gap inside the history is refused, pre-origin positions are false, and no verdict is produced on refusal. | unit-testing (TC-1580) |

## Dependencies

FR-054; FR-001, FR-003, FR-042 own the profile rules.
