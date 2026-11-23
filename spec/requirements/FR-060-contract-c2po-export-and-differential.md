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

- **Decomposition export (design only, UNGROUNDED).** Each contract `c` exports
  three ordinary formulas whose per-step truth is intended to decode the status: `A`,
  `A && G`, `A && !G` (inactive is `!A`; verified is `A && G`; invalid is `A && !G`).
  The Boolean and temporal constructs are in the retained export partition, but nothing
  in the retained corpus (`corpus/r2u2-v4.2`, `past-c2po-v1`, `r2u2-live-v1`) shows how
  R2U2 reports three formulas per contract or that this decoding matches R2U2's own
  contract status, so it is a design, not a grounded fact. Every numeric constraint of
  FR-056 and FR-057 applies, which means that with no pinned profile every contract that
  contains a numeric predicate is refused and the export is empty.
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
  outcome or `none`, and `agree`, `disagree`, `refused-at-export` or
  `no-r2u2-verdict` (R2U2 emitted no verdict for any of the three decomposed formulas
  at that step; never counted as agreement). Generation is DEFERRED as in FR-058.
- **Status.** The differential half of this requirement is OPEN/BLOCKED exactly as
  FR-058 (TL-425 AC2): no specified work produces the exchange, and the only
  deliverable is the refusal-until-pinned path.
- Contract status is per evaluation position; the differential compares per
  position and never aggregates over a trace.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-060-AC-1 | Under the synthetic profile the decomposition export of each contract in the grid is read back by tl-mltl's own reader as the three expected formulas, with legacy manifests unchanged; with no pinned profile every contract in the grid is refused with `TargetNumericProfileUnpinned` and nothing is exported. | unit-testing (TC-1596) |
| FR-060-AC-2 | An unexportable clause refuses the contract with the typed loss record and no partial output. | unit-testing (TC-1597) |
| FR-060-AC-3 | The retained contract differential (192 per-step records) replays and counts agree, disagree, refused and no-R2U2-verdict separately. OPEN/BLOCKED: it cannot pass until a human R2U2 run produces the bytes. | demonstration (TC-1598) |

## Dependencies

FR-059 supplies outcomes; FR-056/057 export; FR-058 retention.
