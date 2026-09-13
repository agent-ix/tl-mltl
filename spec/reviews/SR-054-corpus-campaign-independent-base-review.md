---
id: SR-054
title: "Independent base review of the current corpus campaign"
type: SpecReview
analysis: base
scope: "MRS-002, FR-008 through FR-010, NFR-004, MP-002, TM-002, PLAN-006"
review_set: all
---

## Summary

**PASS after remediation.** Independent current-base review found six material
gaps that the earlier author review could not cover or did not close. The
normative artifacts and PLAN-006 now define one implementable current
future/W/M baseline while retaining past/native work as explicit blocked
successors. This review does not record human acceptance or authorize code.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5401 | high | W/M was still described as blocked after its specification and all routed implementations landed. Fixed by naming the exact merge revisions and making W/M eligible while past/native rows remain blocked. | MRS-002, FR-008-AC-4, TM-002 |
| FND-5402 | high | `cellSha256` named a “canonical tuple” but no byte preimage or finite registry boundary, so implementations could hash different bytes or silently shrink the denominator. Fixed with exact JSON-array/domain preimages, ordering, set digest, successor rule, and required-class census. | FR-008-AC-1, TC-037, TC-038 |
| FND-5403 | high | Loss, availability, and comparison were called orthogonal but valid combinations and refusal versus non-conclusive classification were unstated. Fixed with a closed validity table and deterministic admission/result rules. | FR-010-AC-1, FR-010-AC-3, TC-046 |
| FND-5404 | high | Manifest hard caps and aggregate/hash preimages were unnamed, while a manifest hash appeared self-referential. Fixed with `tl-mltl.corpus-limits/v1`, checked wire conversions, exact manifest/corpus hashing, and an external manifest pin. | FR-009-AC-1, FR-009-AC-6, TC-044, TC-045 |
| FND-5405 | high | Issue #38 required routed implementation tasks, but the branch had no plan or tickets and TC-048 had no manifest to validate. Fixed with PLAN-006 and linked tickets tl-mltl #51–#55, tl-syntax #44, tl-parse #33, and tl-rewrite #37. | MRS-002, PLAN-006, TC-048 |
| FND-5406 | medium | NFR-004 said AP-001 governed the post-v0.1 campaign although AP-001 explicitly accredits only one v0.1 candidate. Fixed by retaining PGM-01 human authority and requiring a successor campaign assurance profile for any release claim. | NFR-004, AP-001 |
