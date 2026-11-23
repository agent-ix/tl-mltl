---
id: SR-1520
title: "Gap analysis: ticket criteria to requirements and test rows"
type: SpecReview
analysis: gap-analysis
scope: "ticket criteria TL-427..TL-435 to requirement and matrix row"
review_set: subset
---

## Summary

Every acceptance criterion of TL-427 to TL-435 maps to a requirement and a TC row in TM-047 or TM-300; no criterion is unmapped. Gaps are in quality of mapping. Criteria mapped only to blocked or DEFERRED rows: TL-428 AC3 (TC-231, TC-232), TL-431 AC1 (TC-240, TC-242, fixtures not retrieved), TL-434 AC1 (TC-321, TC-322), TL-435 AC1 and part of AC2 (TC-323..325). TL-427 body decisions not in the ledger as criteria: LTL3 verdict names (FR-085), mission-bound closure (FR-094), next operator (tl-syntax FR-333). TL-430 scope "bounded witness search" maps to FR-300 but TM-300 ledger omits it (rows TC-300, TC-301 exist). TL-432 AC2 (refusal naming construct) maps to FR-086-AC-2 but ticket-level wording "typed error" differs from TL-431 "loss record": see FND-15201. Matrix traces: TC-319 traces to FR-308-AC-2 whose own text names TC-324 only. Coverage deltas: tl-syntax +26 rows (239 to 265), tl-parse +30 (116 to 146), tl-mltl +68 (265 to 333), tl-solve +55 (204 to 259), all unbacked.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15201 | medium | Two refusal vocabularies: `C2poImportError::UnsupportedConstruct` (FR-086, whole-file abort) versus `LossRecord` per requirement with conservation (FR-091, FR-088). Behavior of a FRET C2PO file holding one unsupported construct among several requirements is undefined. | FR-086, FR-088, FR-091 |
| FND-15202 | medium | TL-427 decision items FR-085, FR-094 lack a ticket-criterion row in the ledger. | TM-047 ledger |
| FND-15203 | low | TM-300 ledger omits TL-430 scope row for FR-300; TC-319 trace to FR-308-AC-2 not reflected in FR-308. | TM-300 |
| FND-15204 | low | No unmapped acceptance criteria found. | - |
