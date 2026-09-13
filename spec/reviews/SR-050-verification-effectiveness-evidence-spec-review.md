---
id: SR-050
title: "Evidence-method review of the verification-effectiveness campaign"
type: SpecReview
analysis: evidence
scope: "FR-011 through FR-015, NFR-005, MP-003 through MP-006, TM-003"
review_set: all
---

## Summary

**PASS after remediation.** The pinned Quoin advisor reports zero
uncatalogued or inconclusive obligations. All MRS-003 acceptance and measurement
obligations match their authored evidence classes; the two remaining advisor
mismatches belong to pre-existing NFR-002/NFR-003 rows outside this review.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5001 | medium | The automated-authority metric was initially authored as Inspection despite being a zero-occurrence invariant that a refusal test can exercise. Fixed by selecting Test. | NFR-005-M-11, TC-068 |
| FND-5002 | high | Property evidence could be vacuous or self-oracled. Fixed with complete domain/partition records, accepted/discarded counts, independent oracle provenance, and mutation controls. | FR-011-AC-2 through FR-011-AC-4, MP-003 |
| FND-5003 | medium | Fuzz duration and plateau were at risk of being read as coverage or correctness measures. Fixed: the measure reports exact feature identities and bounded growth only, with no reachable-feature denominator. | FR-012, MP-004 |
| FND-5004 | medium | A single blended effectiveness score would erase distinct uncertainty models. Fixed with four separate MeasurementPlans and no aggregate proof ratio or release threshold. | MP-003 through MP-006 |
| FND-5005 | high | Current cargo-mutants summary evidence loses identities and dispositions. Fixed: FR-013 requires lossless raw domain evidence or a future accepted shared adapter; absent capability blocks intake. | FR-013, FR-015 |
