---
id: FR-031
title: Evaluate infinite future and past temporal operators
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-030
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-289
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-161
    type: references
---

# FR-031: Evaluate infinite future and past temporal operators

## Description

For a validated `tl-syntax.formula-unbounded/v1` graph and admitted infinite
trace, the provider shall evaluate Boolean operators and F/G/U/R/O/H/S/T
with closed or unbounded intervals using QSpec FR-161's inductive semantics
at a caller-selected event position.

## Behavior

Future positions extend without an end-of-trace sentinel. `F[a,)` has a witness
at some offset at least `a`; `G[a,)` requires its operand at every such
offset. `p U[a,b] q` holds at `t` when `q` holds at some `t+i` with
`a <= i <= b` and `p` holds at every `t+j` with `a <= j < i`; `p U[a,)` is the
same with no upper bound. The left operand is never read below the lower
bound, which is the closed-trace rule of FR-001. `R[a,b]` and `R[a,)` are the
Boolean duals, `not(not p U not q)`. The bounded forms quantify
only their inclusive closed offsets on the infinite trace, without the
closed-finite rule that absent future observations are false. A bounded
future formula therefore has the same verdict from the infinite provider on a
lasso as from the closed-trace evaluator on a prefix that holds every offset
the formula reads, for every lower bound. Past operators
look toward origin zero, so unbounded past intervals remain finite at each
selected position and never wrap a lasso backward before origin. `O/H/S/T`
follow the corresponding FR-161 inductive/dual clauses with the same origin
boundary. Derived W/M lower exactly as tl-syntax FR-008 specifies. Interval
translation from TL `[a,)` to QSL `[a,*]` is a correspondence of denotations,
not a change to the TL wire spelling or profile identity.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-031-AC-1 | F/G/U/R closed and unbounded intervals match independent inductive oracles on lasso witnesses, including lower bound zero and nonzero; for a nonzero lower bound the `U` and `R` guard excludes the offsets below it. | property-based-testing (TC-145, TC-146) |
| FR-031-AC-2 | O/H/S/T and Y match independent origin-based past oracles on mixed future/past graphs; no backward loop wrap or finite-closure substitution occurs. | unit-testing (TC-147, TC-148) |
| FR-031-AC-3 | Boolean duals, U/R and S/T duals, W/M lowering and TL-to-QSL interval correspondence preserve verdicts under their stated preconditions, and a bounded future formula gets the same verdict from the infinite provider as from the closed-trace evaluator at every lower bound. | property-based-testing (TC-149) |

## Dependencies

FR-030 owns partial truth; tl-syntax FR-289 owns graph grammar and edition;
QSpec FR-161 owns the inductive semantic authority.
