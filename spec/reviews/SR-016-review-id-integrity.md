---
id: SR-016
title: "Base review — tracked SpecReview identity integrity"
type: SpecReview
analysis: base
scope: "spec/reviews, spec/requirements/NFR-002-governance-boundary.md, tests/shared_assurance.rs"
review_set: base
---

## Summary

The active review set contained two documents declaring `SR-012`. The residual
composite review is renumbered to the next unused identity, while the
context-bound review retains `SR-012`; the only live requirement citation to
the residual finding now follows that identity. A tracked-tree test makes a
second active duplicate fail rather than silently changing review attribution.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1601 | medium | Two active review artifacts declared `SR-012`, so a citation could name two different findings. Fixed by assigning the residual composite review `SR-015` and preserving the context-bound review's existing identifier. | SR-012, SR-015 |
| FND-1602 | medium | `NFR-002` cited the residual review's FND-1204 but would have become ambiguous after the duplicate was discovered. Fixed by updating only that live citation to `SR-015 FND-1204`; PLAN-003 continues to cite the context-bound SR-012. | NFR-002, PLAN-003 |
| FND-1603 | low | No executable control rejected future duplicate review IDs. Fixed by a tracked `spec/reviews` census that reads each frontmatter identity and fails with every colliding path. | tests/shared_assurance.rs |

## Verification

`cargo test --all-features --test shared_assurance
every_tracked_spec_review_id_is_unique -- --nocapture` passed after staging the
rename, so the test enumerated the same tracked tree the candidate commit will
contain. `quire validate --scope . 'spec/**/*.md'` also passed.
