---
id: SR-065
title: "Independent dependency review of the M5 verification campaign"
type: SpecReview
analysis: dependency
scope: "MRS-003, FR-011 through FR-015, NFR-005, PLAN-004, linked tickets"
review_set: all
---

## Summary

**PASS after remediation.** The local critical path is acyclic, landed M0/W-M
work is no longer a false blocker, M4 and shared capabilities remain real
external gates, and result-driven proof selection no longer depends on its own
final disposition.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6501 | high | Stale M0 and W/M prerequisites obscured the actual gate: human-accepted and landed M4 plus accepted M5. Corrected without admitting past/native rows. | MRS-003, Task-009 |
| FND-6502 | high | Mutation equivalence created a cycle between final survivor disposition and proof-candidate selection. Raw immutable missed/timeout results now supply the conditional Task-012 to Task-013 edge. | FR-013, FR-014, PLAN-004 |
| FND-6503 | high | Quoin #363/#364 are still open and remain narrow gates only for records needing binary attachment or truthful non-release profiles; no local substitute or whole-campaign false dependency was added. | FR-015, Tasks 011/013/014 |
| FND-6504 | medium | Existing ecosystem tickets cover sibling repository campaigns, while seven new blocked tl-mltl tickets cover the local task DAG without broadening historical #31. | MRS-003, PLAN-004 |

## Topological order

`landed M4 + accepted M5 -> Task-009 -> Task-010 -> {Task-011 || Task-012 || Task-013} -> Task-014 -> Task-015`.
Task-013 may additionally consume a raw Task-012 outcome; applicable shared
intake waits on #363/#364 without blocking domain records that need neither
capability.
