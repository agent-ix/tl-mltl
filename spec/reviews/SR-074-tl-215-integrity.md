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
| FND-001 | high | TL-212 still phrases depth-three all-tree exhaustiveness as a local-gate obligation, while the authored FR-044 can only claim enumerated finite partitions until the full cardinality is visited. This needs an explicit owner disposition. | FR-044, TL-212 |

## Evidence and disposition

The amended FR-290/FR-033 route carries subject kind and identity. A V1 model request without a model-wide procedure is unsupported; a complete lasso may yield only a trace-scoped conclusion.

The review is not human-accepted. Implementation remains gated by TL-215.
