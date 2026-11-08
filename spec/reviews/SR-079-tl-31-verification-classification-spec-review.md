---
id: SR-079
title: "Base spec review: verification classification of tl-mltl acceptance criteria (TL-31)"
type: SpecReview
analysis: base
scope: "Verification cells of spec/requirements, the Verification Classification section of spec/test-matrix.md, and TC Type corrections in the test matrices"
review_set: base
---

## Summary

Acceptance criteria now name a catalog method in the Verification cell:
18 `property-based-testing`, 19 `unit-testing`, 46 `integration-testing`,
1 `fuzzing`, 2 `Inspection`. Property means a tagged proptest, or a tagged test
enumerating a bounded domain against an independent oracle, asserts the
criterion. 36 planned rows with no tagged test stay `Test`, unclassified, and
2 are `Withdrawn`. Eleven test cases typed Property without such a test are
retyped Unit. `make spec` reports 85 unbacked rows on main and 77 after the
FR-020 rows were tagged to the merged tests; no coverage is faked.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-0791 | medium | FR-020 is rewritten from the ledger and digest contract to evaluator property laws with recorded seeds and case counts, compared with the independent oracle. After PRs 137 and 138 merged, AC-1, AC-2 and AC-4 are tagged to `tests/temporal_laws.rs`, `tests/finite_oracle.rs` and `tests/boundary_corpus.rs` and classified property; AC-3 is example-only (seeded-fault controls) and its text was narrowed to what those controls assert; AC-5 stays planned. FR-002-AC-1 and FR-003-AC-1 are now property through the merged laws. Leftover FR-020 ledger wording in FR-021, FR-022, FR-023 and NFR-005 is removed; FR-023's own proof-candidate ledger is a Kani artifact and is unchanged. | spec/requirements/FR-020-property-obligation-ledger.md |
| FND-0792 | low | Existing tests are not tagged to any criterion: `tests/finite_oracle.rs` (open PR 138) and the once/historically proptests in `tests/past_history.rs`. They do not count toward property until tagged. | spec/test-matrix.md |
| FND-0793 | low | FR-030-AC-1, FR-031-AC-1 and FR-032-AC-2 are classified property through enumerated-lasso tests, while sibling cross-comparisons over fixed hand-picked words are example-only. | tests/infinite_matrix_rows.rs, tests/infinite_oracle.rs |
| FND-0794 | low | FR-018-AC-1 through AC-3 are reclassified from Inspection to integration-testing (example-only); `tests/tc_090_owner_wire_contracts.rs` already carries the tags. | spec/requirements/FR-018-publish-temporal-owner-wire.md |

## Checklist result

`quire validate` passes. No hash, digest or pin is introduced.
