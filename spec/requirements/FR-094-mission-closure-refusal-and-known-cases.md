---
id: FR-094
title: "Refuse implicit mission-bound closure and classify the known meaning-changing cases"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-045
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-083
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-091
    type: depends_on
---

# FR-094: Refuse implicit mission-bound closure and classify the known meaning-changing cases

## Description

If any import, export or evaluation path would turn an unbounded formula into a
bounded one, then tl-mltl shall refuse with `MissionClosureRequired` unless an
acknowledgement bound to the closure report digest is supplied.

## Behavior

The known-case table is computed, not asserted: for each depth-one temporal
formula over two atoms (`F`, `G` on the 4 atom and constant operands = 8;
`U`, `R` on the 16 ordered operand pairs = 32; 40 formulas), each mission time
`M` in `{1, 2}` (80 closures), and each of the 340 traces of length 1 to 4, the
LTLf verdict is compared with the closed-trace verdict of the bounded closure
`[0,M]` (27,200 comparisons). The table records per operator and `M` whether the
closure agreed on all, some or no traces, and is the reference for FR-090's
citations.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-094-AC-1 | An unbounded formula presented to the C2PO exporter, the closure API or any importer is refused with `MissionClosureRequired` and no artifact, unless a matching acknowledgement is supplied. | Test (TC-254) |
| FR-094-AC-2 | The known-case table over 27,200 comparisons matches an independently written enumerator, and at least one operator is classified as changing meaning (G on a trace shorter than M+1). | Test (TC-255) |
| FR-094-AC-3 | An acknowledgement for a different formula, mission time or report digest is refused. | Test (TC-256) |

## Dependencies

FR-083 supplies the LTLf side; ADR-045 owns the policy.
