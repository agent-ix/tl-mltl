---
id: SR-073
title: "Failure-domain review: partial traces, lasso topology and registration"
type: SpecReview
analysis: failure-domain
scope: "TL-207, TL-208, TL-209, TL-210, TL-211, TL-212, TL-213; tl-syntax c6010f7, tl-parse 83f696c, tl-rewrite 67edaa4, tl-mltl 7db847f"
review_set: all
---

# SR-073: Failure-domain review: partial traces, lasso topology and registration

## Summary

Examined malformed loops, conflicting evidence, empty fair admission, duplicate backend registration, resource exhaustion and bounded-prefix proof leakage. The V1 contracts name typed, non-proving outcomes for these paths.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | The first-lap lasso scaffold is intentionally insufficient for nested past/future over repeated loops; FR-053 and TL-221 require the independent oracle extension before qualification. | FR-053, TL-221 |

## Evidence and disposition

FR-030 keeps missing and conflicting reasons distinct even though both have {true,false} possibilities. FR-032 prevents vacuous proof after fairness empties the admitted set. FR-033 keeps resource-incomplete on the FR-341 failed axis.

The review is not human-accepted. Implementation remains gated by TL-215.
