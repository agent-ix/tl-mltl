---
id: SR-066
title: "Independent evidence-method review of the M5 verification campaign"
type: SpecReview
analysis: evidence
scope: "FR-011 through FR-015, NFR-005, MP-003 through MP-006, TM-003"
review_set: all
---

## Summary

**PASS with disclosed tooling limitation.** Quoin 0.23.1 evaluated all 40 M5
obligations with zero mismatches, uncatalogued methods, or inconclusive
recommendations. Property, fuzz, mutation, bounded analysis, replay, and
negative controls are allocated without claiming planned evidence has run.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6601 | high | Identity tests lacked executable byte-vector oracles. TC-050/051/055/059/065 now name every exact preimage, ordering, successor, and mutation condition. | FR-011 through FR-014, TM-003 |
| FND-6602 | high | TC-056 could pass a plateau from count equality or a window with no covering snapshot. It now requires cadence-valid exact feature sets at both anchors. | FR-012-AC-2, TC-056 |
| FND-6603 | high | TC-063 could not construct a proof-selected survivor without circular evidence. It now begins from the raw outcome and separately verifies final routing, duplicate roots, and risk expiry. | FR-013-AC-4, TC-063 |
| FND-6604 | medium | TC-074 named finite caps but not their values or source. It now exercises every FR-009 limit, checked conversion, and path/digest refusal. | FR-015-AC-4, TC-074 |
| FND-6605 | medium | TC-075 had no unique task/ticket manifest. It now validates PLAN-004's unique ids, exact GitHub routes, owners, consumers, methods, predecessors, and resume conditions. | PLAN-004, TC-075 |

Repo-wide advisor output still contains only the two pre-existing method
mismatches NFR-002-M-2 and NFR-003-M-11 outside M5.
