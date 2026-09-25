---
id: SR-074
title: "Integrity review: semantics and claim scope"
type: SpecReview
analysis: integrity
scope: "TL-207, TL-208, TL-209, TL-210, TL-211, TL-212, TL-213; tl-syntax c6010f7, tl-parse 83f696c, tl-rewrite 67edaa4, tl-mltl 7db847f"
review_set: all
---

# SR-074: Integrity review: semantics and claim scope

## Summary

Checked profile, clock, wire edition, partial-valuation and result vocabulary consistency against QSpec revision 2449ceb. The review corrected the resource-incomplete mapping in tl-parse and tl-rewrite and separated one-lasso satisfaction from model-wide proof.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | Initial TL-212 wording overstated the local gate. The ticket now matches FR-044: full depth-three exhaustiveness remains incomplete until its full population is visited. Resolved on re-review. | FR-044, TL-212 |

## Evidence and disposition

The amended FR-290/FR-033 route carries subject kind and identity. A V1 model request without a model-wide procedure is unsupported; a complete lasso may yield only a trace-scoped conclusion.

Re-review of the amended TL-212 description found no remaining ticket/spec
claim mismatch for FR-044.

The review is not human-accepted. Implementation remains gated by TL-215.
