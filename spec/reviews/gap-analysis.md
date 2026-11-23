---
id: SR-1501
title: "gap-analysis review of TL-150 numeric specs (TL-150-spec-tl-mltl)"
type: SpecReview
analysis: gap-analysis
scope: "TL-422, TL-423, TL-424 (eval half), TL-425 (eval and export)"
review_set: all
---

## Summary

All ACs map to FR/TC rows; TL-422 fully; TL-423 and TL-425 differential ACs map to TCs that are blocked DEFERRED (TC-1589, 1590, 1592, 1598, 1601, 1602); TL-425 export AC is decomposition only.

Method note: independent fresh-context review, no build tools run. Skill lenses were applied manually by the reviewer in one pass; the criterion-strength lens normally uses the external Jev service, which was not used. All specs are unbuilt and ADRs are proposed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TL-423 AC1 and TL-425 AC2 differential evidence cannot be produced by this spec's build tickets without an R2U2 run and a pinned numeric profile; make.numeric-differential is not in ci until bytes exist. Needs a human owner. | FR-058; FR-060; TM-016 |
| FND-002 | medium | TL-422 AC3 "outside reference chosen in the spec": reference is exact-rational script (retained under tl-syntax corpus provenance) plus tl-oracle; script is retained but not an outside tool. | FR-054 |
| FND-003 | low | No unmapped ACs in tl-mltl scope. | TM-016 |
