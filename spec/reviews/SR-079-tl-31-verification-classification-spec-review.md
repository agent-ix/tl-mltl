---
id: SR-079
title: "Base spec review: verification classification of tl-mltl acceptance criteria (TL-31)"
type: SpecReview
analysis: base
scope: "Verification cells of spec/requirements, the Verification Classification section of spec/test-matrix.md, and TC Type corrections in the test matrices"
review_set: base
---

## Summary

The 82 tagged or inspected acceptance criteria now name a catalog method in
the Verification cell: 13 `property-based-testing`, 21 `unit-testing`,
45 `integration-testing`, 1 `fuzzing`, 2 `Inspection`. Property means a
tagged proptest, or a tagged test enumerating a bounded domain against an
independent oracle, asserts the criterion. 36 planned rows with no tagged
test stay `Test`, unclassified, and FR-020-AC-1 through AC-5 are rewritten to the
rescoped property-law intent and name their intended method, still planned; 2 are `Withdrawn`. Eleven test cases typed
Property without such a test are retyped Unit. `make spec` reports 87 unbacked
rows before and after; this change adds no tests and fakes no coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-0791 | medium | FR-020 is rewritten from the ledger and digest contract to evaluator property laws run as tagged generated-input tests with recorded seeds and case counts, compared with the independent oracle. The ledger and digest ceremony is dropped. The five rows stay planned; PR 137 laws and PR 138 oracle agreement will back AC-1, AC-2 and AC-4 once merged and tagged. FR-021 to FR-023 still refer to an applicable FR-020 row and need a follow-up edit. | spec/requirements/FR-020-property-obligation-ledger.md |
| FND-0792 | low | Existing tests are not tagged to any criterion: `tests/finite_oracle.rs` (open PR 138) and the once/historically proptests in `tests/past_history.rs`. They do not count toward property until tagged. | spec/test-matrix.md |
| FND-0793 | low | FR-030-AC-1, FR-031-AC-1 and FR-032-AC-2 are classified property through enumerated-lasso tests, while sibling cross-comparisons over fixed hand-picked words are example-only. | tests/infinite_matrix_rows.rs, tests/infinite_oracle.rs |
| FND-0794 | low | FR-018-AC-1 through AC-3 are reclassified from Inspection to integration-testing (example-only); `tests/tc_090_owner_wire_contracts.rs` already carries the tags. | spec/requirements/FR-018-publish-temporal-owner-wire.md |

## Checklist result

`quire validate` passes. No hash, digest or pin is introduced.
