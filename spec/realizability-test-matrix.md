---
id: TM-007
title: Requirement realizability reference play-out test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: covers
---

# Requirement realizability reference play-out test matrix

Rows for FR-043 and StR-004 (TL-413 in project P-TL-149). The population the
property rows range over is declared once, in tl-solve TM-100
(`ix://agent-ix/tl-solve/TM-100`, section Declared small population): 126960
cases (2160 single-formula requirement sets times 16 input/output splits, plus
15400 two-formula sets times 6 splits). No row is implemented at
the time of writing; every status is planned.

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-188 | Trace length and refusals over the declared population: length is one plus the maximum lookahead, non-closed-trace profiles are refused, no caller length exists | Property | P0 | FR-043-AC-1, FR-043-AC-2, StR-004-VC-1 | 🚧 planned, not yet written |
| TC-189 | Verdict and the forall-exists relaxation match an independent quantifier enumeration over the declared population, including the cases that pass the relaxation but are unrealizable | Property | P0 | FR-043-AC-2, FR-043-AC-3, StR-004-VC-1 | 🚧 planned, not yet written |
| TC-190 | All-output partitions agree with satisfiability by enumeration and all-input partitions with validity | Property | P1 | FR-043-AC-4, StR-004-VC-2 | 🚧 planned, not yet written |
| TC-191 | Trace-length, alphabet-width and play-count limits admit the limit and refuse one past it; unpartitioned proposition is refused | Unit | P0 | FR-043-AC-5, StR-004-VC-2 | 🚧 planned, not yet written |
| TC-192 | Deterministic reports and evaluation failure returns no verdict | Unit | P1 | FR-043-AC-6, StR-004-VC-1 | 🚧 planned, not yet written |
