---
id: SR-075
title: "Dependency review: syntax to verification"
type: SpecReview
analysis: dependency
scope: "TL-207, TL-208, TL-209, TL-210, TL-211, TL-212, TL-213; tl-syntax c6010f7, tl-parse 83f696c, tl-rewrite 67edaa4, tl-mltl 7db847f"
review_set: all
---

# SR-075: Dependency review: syntax to verification

## Summary

The new requirement DAG follows TL-207 syntax and STD-13 authority, TL-210 provider, TL-211 export, TL-212 verification, then TL-215 acceptance before code. The separate unpublished tl-oracle is an enablement dependency for rewrite and infinite qualification.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | The tl-oracle repository is not scaffolded yet; FR-043 names TL-35/TL-221 as its implementation predecessor, so verification remains planned until that dependency lands. | FR-043, TL-35, TL-221 |

## Evidence and disposition

No new dependency cycle was found. Cargo feature unification is explicit in ADR-003; tl-rewrite does not select the infinite feature in production.

The review is not human-accepted. Implementation remains gated by TL-215.
