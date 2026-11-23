---
id: FR-059
title: Evaluate contracts with inactive, invalid, verified and pending outcomes
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-054
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-310
    type: depends_on
---

# FR-059: Evaluate contracts with inactive, invalid, verified and pending outcomes

## Description

When a contract set is evaluated over a trace, tl-mltl shall report per position
the outcome given by the tl-syntax status function applied to the assumption
and guarantee verdicts at that position.

## Inputs

- A bound formula with its predicate table and catalog (optional), a contract
  set bound to the formula by graph identity, a trace, a profile, limits.

## Outputs

- `tl-mltl.contract-evaluation/v1`: per contract `{ id, name, outcomes: [ {time,
  outcome} ], summary }`; `summary` counts `inactive`, `invalid`, `verified`,
  `pending` and records the first `invalid` time if any. Strict reader and
  schema as for other owner contracts.

## Behavior

- For each evaluation time `t` from 0 through the last observed position, the
  assumption and guarantee roots are evaluated at `t` by the existing evaluator
  of the profile (closed: padding applies; prefix: `Pending` possible;
  origin-complete: the anchor is `t`).
- The outcome is the tl-syntax truth table: assumption false is `inactive`
  regardless of the guarantee; assumption true and guarantee false is `invalid`;
  both true is `verified`; otherwise `pending`.
- A contract set whose graph identity does not match the supplied formula is
  refused (`GraphIdentityMismatch`) with no outcomes.
- Work is bounded: positions x contracts x node evaluations, checked first.
- The report keeps the whole per-position series so a certification reader can
  see a never-exercised contract (all `inactive`) distinctly from a verified one.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-059-AC-1 | All nine `(assumption, guarantee)` combinations reproduce the tl-syntax truth table at a position, on closed and prefix profiles. | unit-testing (TC-1593) |
| FR-059-AC-2 | A contract whose assumption never holds reports only `inactive`, one whose guarantee fails while assumed reports `invalid` at the first such time, and one that holds reports `verified`, on a declared corpus. | integration-testing (TC-1594) |
| FR-059-AC-3 | Mismatched graph identity is refused and the report round-trips through its strict reader. | unit-testing (TC-1595) |

## Dependencies

tl-syntax FR-309 and FR-310.
