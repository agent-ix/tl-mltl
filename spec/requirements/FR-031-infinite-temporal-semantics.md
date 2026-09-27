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
offset. `U[a,)` requires a right witness and left at every preceding offset
starting at zero; `R[a,)` is its Boolean dual. The bounded forms quantify
only their inclusive closed offsets on the infinite trace, without the
closed-finite rule that absent future observations are false. Past operators
look toward origin zero, so unbounded past intervals remain finite at each
selected position and never wrap a lasso backward before origin. `O/H/S/T`
follow the corresponding FR-161 inductive/dual clauses with the same origin
boundary. Derived W/M lower exactly as tl-syntax FR-008 specifies. Interval
translation from TL `[a,)` to QSL `[a,*]` is a correspondence of denotations,
not a change to the TL wire spelling or profile identity.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-031-AC-1 | F/G/U/R closed and unbounded intervals match independent inductive oracles on lasso witnesses, including lower bound zero and nonzero. | Test (TC-145, TC-146) |
| FR-031-AC-2 | O/H/S/T and Y match independent origin-based past oracles on mixed future/past graphs; no backward loop wrap or finite-closure substitution occurs. | Test (TC-147, TC-148) |
| FR-031-AC-3 | Boolean duals, U/R and S/T duals, W/M lowering and TL-to-QSL interval correspondence preserve verdicts under their stated preconditions. | Test (TC-149) |

## Dependencies

FR-030 owns partial truth; tl-syntax FR-289 owns graph grammar and edition;
QSpec FR-161 owns the inductive semantic authority.
