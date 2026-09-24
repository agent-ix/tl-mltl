---
id: SR-076
title: "Evidence review: methods and actual artifact state"
type: SpecReview
analysis: evidence
scope: "TL-207, TL-208, TL-209, TL-210, TL-211, TL-212, TL-213; tl-syntax c6010f7, tl-parse 83f696c, tl-rewrite 67edaa4, tl-mltl 7db847f"
review_set: all
---

# SR-076: Evidence review: methods and actual artifact state

## Summary

Ran quoin advise --mismatch-only in all four repositories and inspected its V1 recommendations. New semantic obligations generally use Test/Property; FR-029 dependency-route inspections are deliberately inspections. Red stubs are not counted as discharged evidence.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | The advisor suggests tests for FR-029-AC-1/3 from their property-shape wording; inspection remains the justified method because those criteria inspect ticket predecessors and the acceptance gate, not runtime behavior. | FR-029-AC-1, FR-029-AC-3 |

## Evidence and disposition

Quire strict coverage remains red for planned unbacked V1 rows, with no contradicted status. The TL-211 past corpus has a checked digest but no fresh R2U2 execution claim; FR-052 requires a separately pinned live run.

The review is not human-accepted. Implementation remains gated by TL-215.
