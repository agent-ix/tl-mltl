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
42 `integration-testing`, 1 `fuzzing`, 5 `Inspection`. Property means a
tagged proptest, or a tagged test enumerating a bounded domain against an
independent oracle, asserts the criterion. 41 planned rows with no tagged
test stay `Test`, unclassified; 2 are `Withdrawn`. Eleven test cases typed
Property without such a test are retyped Unit. `make spec` reports 87 unbacked
rows before and after; this change adds no tests and fakes no coverage.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-0791 | medium | FR-020 (property-obligation ledger) describes the ledger and digest contract that the TL-31 rescope dropped. Withdrawing or rewriting FR-020-AC-1 through AC-5 is a scope decision for the owner, so they are left as planned unbacked rows. | spec/requirements/FR-020-property-obligation-ledger.md |
| FND-0792 | low | Existing tests are not tagged to any criterion: `tests/finite_oracle.rs` (open PR 138) and the once/historically proptests in `tests/past_history.rs`. They do not count toward property until tagged. | spec/test-matrix.md |
| FND-0793 | low | FR-030-AC-1, FR-031-AC-1 and FR-032-AC-2 are classified property through enumerated-lasso tests, while sibling cross-comparisons over fixed hand-picked words are example-only. | tests/infinite_matrix_rows.rs, tests/infinite_oracle.rs |
| FND-0794 | low | FR-018-AC-1 through AC-3 are Inspection though `tests/tc_090_owner_wire_contracts.rs` backs them; candidates for reclassification as example-only. | spec/requirements/FR-018-publish-temporal-owner-wire.md |

## Checklist result

`quire validate` passes. No hash, digest or pin is introduced.
