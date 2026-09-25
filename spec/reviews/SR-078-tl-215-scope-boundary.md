---
id: SR-078
title: "Scope and boundary review: four production crates and dev oracle"
type: SpecReview
analysis: scope-boundary
scope: "TL-207, TL-208, TL-209, TL-210, TL-211, TL-212, TL-213; tl-syntax c6010f7, tl-parse 83f696c, tl-rewrite 67edaa4, tl-mltl 7db847f"
review_set: all
---

# SR-078: Scope and boundary review: four production crates and dev oracle

## Summary

Checked owner allocation: tl-syntax owns grammar and capability; tl-parse owns loci; tl-rewrite owns profile-preserving rules; tl-mltl owns the opt-in provider and C2PO mapping; tl-oracle is unpublished and test-only. QSpec is a correspondence authority, not a TL production dependency.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | low | The Rust-only owner directive still covers pre-existing executable Python and shell qualification paths; this V1 spec cycle adds no new such path, but the existing inventory and disposition remain outside the new V1 requirements. | TL-88, MRS-004 |

## Evidence and disposition

The embedded no_std check applies only to tl-syntax, which declares no_std. ADR-003 makes no unsupported no_std or whole-crate certification claim for tl-mltl.

The review is not human-accepted. Implementation remains gated by TL-215.
