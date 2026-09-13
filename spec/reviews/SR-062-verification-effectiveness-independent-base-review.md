---
id: SR-062
title: "Independent base review of the M5 verification-effectiveness campaign"
type: SpecReview
analysis: base
scope: "MRS-003, FR-011 through FR-015, NFR-005, MP-003 through MP-006, TM-003, PLAN-004"
review_set: all
---

## Summary

**PASS after remediation.** The exact-head review found stale dependency facts,
non-implementable identity prose, a mutation/proof routing cycle, ambiguous fuzz
plateau sampling, and plan/ticket identity gaps. The specification and PLAN-004
now define a bounded M5 campaign without rewriting agent-e's domain allocation
or advancing any planned implementation status.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6201 | high | M0 and W/M were still described as pending after they landed. Fixed by admitting exact current future/W-M work while retaining M4 acceptance, past/history, and native bridge gates. | MRS-003, PLAN-004 |
| FND-6202 | high | Property, mutant, fuzz, and proof identities named domain-separated hashes without byte-exact preimages or deterministic row order. Fixed with compact JSON arrays, explicit domains, ordering, external digests, and checked path/span fields. | FR-011 through FR-014 |
| FND-6203 | high | A mutant could become a proof candidate only through a final disposition whose `equivalent` state required that same proof. Fixed by routing immutable raw missed/timeout outcomes into FR-014 before final disposition. | FR-013-AC-4, FR-014, PLAN-004 |
| FND-6204 | high | PLAN-004 reused Task-001 through Task-007 and did not name task tickets, owners, consumers, methods, or resume conditions. Fixed with unique Task-009 through Task-015 and tl-mltl #58/#56/#60/#61/#59/#57/#62. | PLAN-004, TC-075 |
| FND-6205 | medium | Fuzz snapshot cadence and the two plateau anchors allowed incompatible implementations. Fixed with monotonic time, post-threshold sampling, mandatory final snapshot, and explicit missing-window non-conclusion. | FR-012-AC-1, FR-012-AC-2 |
| FND-6206 | medium | Shared artifact intake repeated ambiguous normalized paths and unnamed caps. Fixed by consuming the exact FR-009 path, checked-integer, digest, and resource profile. | FR-015-AC-4, TC-074 |
