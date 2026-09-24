---
id: SR-072
title: "Base checklist: all four V1 TL specifications"
type: SpecReview
analysis: base
scope: "TL-207, TL-208, TL-209, TL-210, TL-211, TL-212, TL-213; tl-syntax c6010f7, tl-parse 83f696c, tl-rewrite 67edaa4, tl-mltl 7db847f"
review_set: all
---

# SR-072: Base checklist: all four V1 TL specifications

## Summary

Reviewed the four V1 spec branches against ID, link, acceptance-criterion and six-rule Test Matrix checks. Quire validation is grammar-clean in all four repositories; the new TC rows are planned with red stubs, not implementation evidence.

The main cross-crate review index is `SR-110` at
`tl-syntax/spec/reviews/SR-110-tl-215-combined-v1-review.md`, as TL-215 directs.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | Initial TL-212 local-gate wording was infeasible. TL-212 now adopts FR-044's finite-partition scope and incomplete result for unvisited cells. Resolved on re-review. | FR-044, TL-212 |

## Evidence and disposition

TC-085 remains retired for unrelated QObs work. The new TL-210 rows start at TC-138; TL-211 starts at TC-160 and TL-212 at TC-175. No new acceptance criterion lacks a verification reference.

Re-review confirmed that the revised TL-212 description preserves the full
depth-three target without claiming that a local partition completes it.

The review is not human-accepted. Implementation remains gated by TL-215.
