---
id: SR-612
title: "Base spec review: past-profile evaluation requirement FR-042 (TL-378)"
type: SpecReview
analysis: base
scope: "FR-042, TM-006, spec.md FR ownership paragraph"
review_set: base
---

## Summary

FR-042 gives the tl-mltl past-profile evaluator an owning requirement. It adopts
the evaluator text tl-syntax withdrew (FR-011-AC-1, AC-2, AC-3, AC-5 and
FR-012-AC-1, AC-2, AC-5) and states the system behavior and rationale: reverse
offset semantics, pre-origin false extension, origin-complete history admission,
required-history equations, the two admitted clocks, owner non-values, and the
resource limits. Result attribution and correction relations stay with
tl-syntax FR-012-AC-3 and AC-4. No applicable AssuranceProfile exists in this
repository, so no profile governs this review. The base checklist passes: ids
are unique and follow the repository scheme (FR-042, TC-178 through TC-186,
TM-006), criteria are atomic and concrete, and no hash, digest, or pin is
introduced. Every criterion is backed by a tagged test.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-6121 | low | FR-042-AC-7 cannot be driven through `analyze_required_history`: overflow needs about 2^32 nested nodes at `u32::MAX`, past the node-table bound. The test asserts the shared checked addition at its boundary in a unit test beside it, and mutation to a wrapping add fails it. | src/past/mod.rs, spec/past-profile-test-matrix.md |
| FND-6122 | low | FR-042-AC-9 is verified at the refusal level; no test observes that an over-limit interval is refused before iteration, only that the typed error is returned. The criterion is worded to the observable refusal. | tests/past_history.rs |
| FND-6123 | low | tl-syntax still holds the withdrawn rows and its owner-evidence table says TL-378 adds the tags; it should repoint to FR-042 after this lands. | spec/requirements/FR-042-past-profile-evaluation.md |

## Checklist result

`quire validate` passes. Strict coverage: 15 unbacked rows before and 15 after; every FR-042 row is backed.
