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

The V1 target spans all operators, `0 ≤ a ≤ b ≤ 4`, traces through length
six and each applicable profile. Formula depth three is a target; the
enumerator states its exact atom basis, grammar, symmetry reductions and
cardinality before execution. A local run cannot call an operator sample
"every formula". If the literal population exceeds the local budget, it
reports the unvisited count and remains incomplete while a reviewed finite
partition runs locally and deeper partitions run nightly. No partition earns
exhaustive credit until visited count equals independently computed cardinality
and every production/oracle comparison passes. Refused profile combinations
are counted separately, not silently discarded.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-044-AC-1 | Domain cardinality, actual visits and refused cells reconcile exactly for every completed partition. | Test (TC-177) |
| FR-044-AC-2 | Every admissible generated case agrees with the independent oracle; seeded omission, duplicate and wrong verdict are detected. | Test (TC-178) |

## Dependencies

FR-043 supplies the oracle. A literal all-trees depth-three claim remains
incomplete unless its full finite population actually runs.
