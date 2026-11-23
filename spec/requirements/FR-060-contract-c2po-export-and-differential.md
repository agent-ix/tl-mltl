---
id: FR-060
title: Export contracts to C2PO and record the R2U2 contract differential
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-059
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-056
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-058
    type: depends_on
---

# FR-060: Export contracts to C2PO and record the R2U2 contract differential

## Description

When a contract set is exported, tl-mltl shall emit C2PO specifications from
which each contract's per-step status is decodable, and shall record the R2U2
differential for contract statuses by the FR-058 retained-exchange pattern.

## Behavior

- **Decomposition export (grounded).** Each contract `c` exports three ordinary
  formulas whose per-step truth decodes the status: `A`, `A && G`, `A && !G`
  (inactive is `!A`; verified is `A && G`; invalid is `A && !G`). This needs only
  Boolean and temporal constructs already in the export partition and every
  numeric constraint of FR-056/FR-057.
- **Native C2PO contract construct (DEFERRED).** The exact C2PO contract syntax
  and R2U2's contract output encoding are not retained in this repository and
  cannot be verified without running or reading the pinned checkout. A native
  export is added only after the syntax is pinned in the retained exchange; until
  then the decomposition export is the only form, and the report says so.
- An unrepresentable assumption or guarantee (FR-056 loss) refuses the whole
  contract with the loss record; no partial contract is exported.
- **Differential.** The retained exchange `corpus/r2u2-numeric-v1/` includes a
  contract grid: assumptions `x > -1`, `x > 0`, `x > 1` (Integer, domain
  [-2, 1]) by guarantees `d >= 0.2` and `G[0,1] (d >= 0.1)` (Decimal scale 1,
  coefficients [0, 3]) = 6 contracts; 8 traces of 4 steps; 6 x 8 x 4 = 192
  per-step status records, each with tl-mltl's outcome, the decoded R2U2
  outcome, and `agree`, `disagree` or `refused-at-export`. Generation is
  DEFERRED as in FR-058.
- Contract status is per evaluation position; the differential compares per
  position and never aggregates over a trace.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-060-AC-1 | The decomposition export of each contract in the grid parses to the three expected formulas, with legacy manifests unchanged. | unit-testing (TC-1596) |
| FR-060-AC-2 | An unexportable clause refuses the contract with the typed loss record and no partial output. | unit-testing (TC-1597) |
| FR-060-AC-3 | The retained contract differential (192 per-step records) replays and counts agree, disagree and refused separately; generation is DEFERRED. | demonstration (TC-1598) |

## Dependencies

FR-059 supplies outcomes; FR-056/057 export; FR-058 retention.
