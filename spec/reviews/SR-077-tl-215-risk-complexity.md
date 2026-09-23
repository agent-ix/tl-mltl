---
id: SR-077
title: "Risk and complexity review: V1 implementation hazards"
type: SpecReview
analysis: risk-complexity
scope: "TL-207, TL-208, TL-209, TL-210, TL-211, TL-212, TL-213; tl-syntax c6010f7, tl-parse 83f696c, tl-rewrite 67edaa4, tl-mltl 7db847f"
review_set: all
---

# SR-077: Risk and complexity review: V1 implementation hazards

## Summary

The highest technical risks are the partial/fair lasso fixed points, independent oracle, target-origin C2PO equivalence and exhaustive campaign cardinality. External R2U2 availability and version behavior add volatility to the live differential lane.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | FR-044 cannot complete a literal all-tree depth-three domain in the local gate under a realistic finite budget; the campaign must preserve unvisited counts and avoid an exhaustive claim until a tractable scope is selected. | FR-044 |
| FND-002 | medium | R2U2 past-origin equivalence is target-version dependent; FR-039 requires refusing export without a reviewed target contract and FR-052 retains non-conclusive live outcomes. | FR-039, FR-052 |

## Evidence and disposition

Mitigations are independent tl-oracle plus seeded faults (FR-043/053), declarative refusal partition (FR-041), both Cargo feature builds (FR-027), and measured, separately reported campaign lanes (FR-054/055).

The review is not human-accepted. Implementation remains gated by TL-215.
