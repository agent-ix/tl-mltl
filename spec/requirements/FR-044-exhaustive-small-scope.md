---
id: FR-044
title: Enumerate a declared finite small scope exhaustively
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-043
    type: depends_on
---

# FR-044: Enumerate a declared finite small scope exhaustively

## Description

When an exhaustive small-scope campaign runs, the verifier shall enumerate
every member of its declared finite formula, interval, trace and profile
population and compare each admitted case with tl-oracle.

## Behavior

The V1 required partition spans every depth-one operator over `{false, true,
p0}`, `0 ≤ a ≤ b ≤ 4`, all nonempty one-atom traces through length six, and each
applicable profile. Ordered binary operands are distinct and no symmetry
reductions apply. This partition has 807 formulas and 642 trace positions,
or 518,094 comparisons. V2 passes only after all 518,094 comparisons agree
with tl-oracle and the ledger reconciles without omissions or duplicates.
The enumerator also states the literal depth-three all-trees cardinality for
the same grammar and reports its unvisited count. Depth three is a disclosed
longer-range target, not a V2 exit gate. No partition earns exhaustive credit
until visited count equals its independently computed cardinality and every
production/oracle comparison passes. Refused profile combinations are counted
separately, not silently discarded.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-044-AC-1 | The required 518,094-cell partition is complete; its cardinality, visits and refusals reconcile exactly, and the depth-three domain and unvisited count are disclosed separately. | Test (TC-177) |
| FR-044-AC-2 | Every admissible generated case agrees with the independent oracle; seeded omission, duplicate and wrong verdict are detected. | Test (TC-178) |

## Dependencies

FR-043 supplies the oracle. A literal all-trees depth-three claim remains
incomplete unless its full finite population actually runs; V2 acceptance
does not make that claim.
