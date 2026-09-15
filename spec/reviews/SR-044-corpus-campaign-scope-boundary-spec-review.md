---
id: SR-044
title: "Scope-boundary review of the corpus campaign"
type: SpecReview
analysis: scope-boundary
scope: "MRS-002, FR-008 through FR-010, NFR-004"
review_set: all
---

## Summary

**PASS after remediation.** tl-mltl owns campaign census and evaluator overlays;
each producer owns one canonical fixture family; Quoin owns evidence retention;
and native Quire remains the only authored clause language. Target tools are not
runtime dependencies or semantic authorities.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4401 | high | Treating native Quire as another target would invert source authority and invite a TL-authored language. Fixed by separating native correspondence from the external target catalog and blocking it on #63/#64. | MRS-002, FR-010 |
| FND-4402 | medium | A shared corpus could become multiply authored across syntax, parser, rewrite, evaluator, and bridge repositories. Fixed with one authoritative owner per family and exact-digest consumer replay. | FR-009-AC-2 |
| FND-4403 | medium | Repository-owned fixtures could grow into a local generic evidence/retention framework. Fixed: repositories retain domain bytes/manifests only; Quoin retains evidence and receipts. | FR-009-AC-5, NFR-004 |
| FND-4404 | medium | FRETish mapping responsibility could leak into tl-mltl. Fixed: quire-contract-ir owns the Rust FS06 emitter; tl-mltl owns only its canonical TL semantics and C2PO/R2U2 adapter surface. | FR-010 target catalog |

## Responsibility allocation

| Requirement | Owner | Class |
|---|---|---|
| FR-008 | tl-mltl corpus census | core |
| FR-009 | named family owner, coordinated by tl-mltl manifest contract | infrastructure |
| FR-010 | Rust target adapter owner; tl-mltl comparison overlay | core |
| NFR-004 | tl-mltl plus shared Quoin intake | cross-cutting |

## External dependencies

| Dependency | Status | Contract |
|---|---|---|
| tl-syntax profiles/corpus | guaranteed by exact revision and manifest digest | accepted MRS-002/MRS-003 and corpus identity |
| quire-contract-ir native bridge | blocked, then guaranteed by exact correspondence record | #63/#64 |
| Quire/Quoin/Engineering Assurance | guaranteed by pinned shared contracts | assurance/pins.json and FR-006 |
| C2PO/R2U2/FRET runtimes | unavailable/unsupported, never executed | FR-010 target disposition |
