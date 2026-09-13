---
id: SR-048
title: "Integrity review of the verification-effectiveness campaign"
type: SpecReview
analysis: integrity
scope: "MRS-003, FR-011 through FR-015, NFR-005, MP-003 through MP-006, TM-003, source census"
review_set: all
---

## Summary

**PASS after remediation.** The campaign has unique sequential identities,
atomic requirement responsibilities, explicit relationships, and complete
planned AC-to-TC allocation. Quire reports the full current corpus grammar-clean
without representing planned campaign rows as implemented.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4801 | medium | One effectiveness requirement would couple four independently changing evidence domains. Fixed by separating ledger/property, fuzz, mutation, Kani, and shared-intake contracts. | FR-011 through FR-015 |
| FND-4802 | high | Property and proof ledgers could be reclassified without a stable whole-ledger identity. Fixed with lifecycle-independent row identities, classification-covering ordered digests, predecessor digests, successors, and tombstones. | FR-011-AC-1, FR-014-AC-1 |
| FND-4803 | high | The draft treated the production-symbol relation from closed issue quire-rs#171 as absent. Fixed after live inspection: Quire 0.31 exports six distinct non-coverage `implements` edges for FR-001 through FR-005; mutation admission requires complete reviewed local edges without changing backed totals. | FR-013, FR-015 |
| FND-4804 | low | TC-050 through TC-075 are intentionally planned, not implemented. Their status remains planned and they provide no present coverage credit. | TM-003 |

## Traceability

| Requirement | Parent | Verification allocation |
|---|---|---|
| FR-011 | MRS-003 | TC-050 through TC-054 |
| FR-012 | MRS-003, FR-009, FR-011 | TC-055 through TC-058, TC-067, TC-068 |
| FR-013 | MRS-003, FR-011 | TC-059 through TC-064 |
| FR-014 | MRS-003, FR-011 | TC-065 through TC-067, TC-069, TC-070 |
| FR-015 | MRS-003, FR-006, FR-011 through FR-014 | TC-067, TC-068, TC-071 through TC-075 |
| NFR-005 | FR-011 through FR-015 | TC-053, TC-060, TC-064, TC-066 through TC-068, TC-071, TC-073 through TC-075 |
