---
id: SR-040
title: "Integrity review of the corpus campaign"
type: SpecReview
analysis: integrity
scope: "MRS-002, FR-008 through FR-010, NFR-004, MP-002, TM-002, source census"
review_set: all
---

## Summary

**PASS after remediation.** The campaign uses unique sequential identities,
explicit requirement relationships, complete planned AC-to-TC mappings, and a
direct master-specification reference. Quire reports the full corpus grammar-
clean without a planned row claiming implementation.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4001 | medium | Coverage, fixture lifecycle, target disposition, and reproducibility were initially one change surface. Fixed by separating FR-008, FR-009, FR-010, and NFR-004 with explicit prerequisite edges. | FR-008, FR-009, FR-010, NFR-004 |
| FND-4002 | medium | The new profile introduced seven live specification artifacts and eight archival reviews without updating the exact source population. Fixed by changing the spec-area census from 81 to 96 and total reviewed population from 158 to 173; one-count mutations make the gate red. | tests/shared_assurance.rs |
| FND-4003 | low | The post-v0.1 campaign was not discoverable from MRS-001. Fixed with a direct reference while retaining the v0.1 scope unchanged. | MRS-001, MRS-002 |
| FND-4004 | low | TC-037 through TC-049 are intentionally unimplemented. Retained as planned; coverage reports them unbacked and reports zero status lies. | TM-002 |
| FND-4005 | medium | The sealed Quire snapshot still expected 90 rows and named only v0.1 requirements. Fixed to enumerate FR-008 through FR-010/NFR-004 and the measured 122 rows: 66 criteria, 48 test cases, and 8 suite rows, with 34 deliberate unbacked rows. | tests/shared_assurance.rs |

## Traceability

| Requirement | Parent | Verification allocation |
|---|---|---|
| FR-008 | MRS-002 | TC-037 through TC-043, TC-045, TC-047 |
| FR-009 | MRS-002, FR-008 | TC-044, TC-045, TC-047, TC-048 |
| FR-010 | MRS-002, FR-004, FR-005, FR-007, FR-009 | TC-039, TC-043, TC-046, TC-048 |
| NFR-004 | FR-008 through FR-010 | TC-045 through TC-049 |
