---
id: SR-045
title: "EARS-conformance review of the corpus campaign"
type: SpecReview
analysis: ears-conformance
scope: "FR-008 through FR-010 and NFR-004"
review_set: all
---

## Summary

**PASS after remediation.** Quire reports every document grammar-clean. Manual
review confirms distinct event triggers and concrete system responses for cell
validation, fixture admission, target mapping, and reproducibility.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4501 | low | FR-009 initially split its event subject and `shall` across lines, which the deterministic grammar could not classify. Fixed by keeping the canonical `When ..., the corpus campaign shall ...` clause intact. | FR-009 |
| FND-4502 | medium | One broad campaign obligation would conflate independently changing census, fixture-lifecycle, target-disposition, and reproducibility responses. Fixed by FR-008, FR-009, FR-010, and NFR-004. | FR-008, FR-009, FR-010, NFR-004 |
| FND-4503 | low | No remaining trigger, subject, vague-response, optionality, or grammar finding remains after remediation and strict full-corpus validation. | spec/requirements/ |
