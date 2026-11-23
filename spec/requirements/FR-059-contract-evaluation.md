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
  outcome, padded} ], summary }`. `padded` is true when the position's outcome
  depends on a closed-trace padding value (see Behavior). `summary` counts
  `inactive`, `invalid`, `verified` and `pending` separately for unpadded and padded
  positions, and records the first `invalid` time and the first `inactive` time over
  unpadded positions only (`null` if none). Strict reader and schema as for other
  owner contracts. There is no trace-level status field: a whole-trace answer is a
  function of the series that a consumer computes (see tl-syntax ADR-017, Open
  decisions).

## Behavior

- For each evaluation time `t` from 0 through the last observed position, the
  assumption and guarantee roots are evaluated at `t` by the existing evaluator
  of the profile (closed: padding applies; prefix: `Pending` possible;
  origin-complete: the anchor is `t`).
- **Closed profile never yields `pending`.** A closed trace pads every unobserved
  position with false, so `a` and `g` are always decided and only `inactive`, `invalid`
  and `verified` occur; the `pending` rows of the tl-syntax truth table are reachable
  only on the prefix profile. Tests of those rows run on the prefix profile (and
  directly on the tl-syntax function), never on the closed profile.
- **Tail padding.** A position `t` is `padded` when the lookahead of the assumption
  or of the guarantee from `t` (tl-mltl horizon analysis) reaches beyond the last
  observed position. On the closed profile such a position is still reported with its
  decided outcome, flagged `padded`, because padded-false can make a guarantee with a
  future operator fail (a spurious `invalid`) or an assumption fail (a spurious
  `inactive`) at the last positions. Padded outcomes never set `firstInvalid` or
  `firstInactive`. On the prefix profile no position is padded: the same positions are
  `pending` unless Kleene evaluation already decides them. A reader who needs the
  tail answered without padding artefacts uses the prefix profile.
- **Empty trace.** A trace with zero instants has no last observed position, so the
  series is empty, all counts are zero and both first times are `null`; no outcome is
  fabricated for position 0 and the report states `emptyTrace: true`.
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
| FR-059-AC-1 | On the prefix profile all nine `(assumption, guarantee)` combinations (`True`, `False`, `Pending` for each) are produced at some position of a declared case set and reproduce the tl-syntax truth table; on the closed profile only the four decided combinations occur, each reproduces the table, and no `pending` outcome is ever emitted. | unit-testing (TC-1593) |
| FR-059-AC-2 | A contract whose assumption never holds reports only `inactive`, one whose guarantee fails while assumed reports `invalid` at the first such time, and one that holds reports `verified`, on a declared corpus. | integration-testing (TC-1594) |
| FR-059-AC-3 | Mismatched graph identity is refused and the report round-trips through its strict reader. | unit-testing (TC-1595) |
| FR-059-AC-4 | For guarantee `G[0,1] p` with `p` true at every position of a 3-step trace and a true assumption, the closed profile reports `invalid` flagged `padded` at the last position and no `firstInvalid`, the prefix profile reports `pending` there; an assumption `F[0,1] q` failing only by padding reports a flagged `inactive` and no `firstInactive`. | unit-testing (TC-1603) |
| FR-059-AC-5 | A trace of zero instants yields an empty series, zero counts, `null` first times and `emptyTrace: true` on every profile, with no fabricated outcome. | unit-testing (TC-1604) |

## Dependencies

tl-syntax FR-309 and FR-310.
