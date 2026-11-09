---
id: SR-355
title: "Base spec review: re-derive MRS-002 and MRS-003 to the delivered campaign (TL-355)"
type: SpecReview
analysis: base
scope: "MRS-002, MRS-003, FR-008 through FR-010, FR-020-AC-5, FR-021 through FR-023, NFR-004, NFR-005, TM-002, TM-003, and the FR-029 withdrawals"
review_set: base
---

## Summary

The rows that failed `quire coverage --strict` were written before the September
rescopes and described a digest, registry, and ledger machinery that was never
built. They are rewritten to the shape that was delivered: a plain cell list with
a gap check and oracle agreement (FR-008), agree, disagree, or not-comparable
outcomes (FR-010), multi-seed fuzz runs with a plateau stop (FR-021), mutation
thresholds with a green-baseline rule and listed survivors (FR-022), and Kani
harnesses with stated bounds and a vacuity guard (FR-023). Withdrawn rows stay in
the matrices marked retired so no identifier dangles. Strict-unbacked rows drop
from 77 to 67; every remaining row is a planned row with no tagged test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-3551 | low | Rewritten criteria describe delivered behavior that the existing tests realize but do not yet tag, so the rows stay planned until the retag ticket. | spec/corpus-campaign-test-matrix.md, spec/verification-effectiveness-test-matrix.md |
| FND-3552 | low | FR-022 and FR-023 add requirements that have no implementation yet: threshold gate over cargo-mutants output, survivor list check, and four of five Kani harnesses. | spec/requirements/FR-022-measured-mutation-campaign.md, spec/requirements/FR-023-bounded-kani-claims.md |
| FND-3553 | low | FR-029-AC-3 is withdrawn as a claim-ordering gate; its feature-off `unsupported` half is verified under FR-028-AC-1. | spec/requirements/FR-029-infinite-trace-downstream-evidence.md |

## Checklist result

`quire validate` passes. The digest, preimage, ledger, and registry requirements
were removed rather than reworded; no hash, digest, or pin is introduced.
