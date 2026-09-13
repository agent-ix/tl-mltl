---
id: SR-060
title: "Independent scope-boundary review of the current corpus campaign"
type: SpecReview
analysis: scope-boundary
scope: "MRS-002, FR-008 through FR-010, NFR-004, PLAN-006"
review_set: all
---

## Summary

**PASS after remediation.** Campaign schema, owner-family production,
comparison, retention, and native-source responsibilities are allocated once.
AP-001 is no longer stretched beyond its v0.1 decision boundary.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-6001 | high | No machine-readable routing proved the singular owner claims in FR-009. Fixed with PLAN-006 owner/consumer fields and one linked GitHub ticket per deliverable. | FR-009-AC-2, PLAN-006 |
| FND-6002 | medium | AP-001 could be read as accrediting a post-v0.1 campaign. Fixed by restricting it to its declared v0.1 candidate and requiring a successor assurance profile for any campaign release claim. | NFR-004, AP-001 |
| FND-6003 | medium | Native bridge work could leak source grammar, predicate, or clock/history semantics into tl-mltl. Retained owner boundary: Task-008 consumes exact #63/#64 families only and cannot define a fallback. | MRS-002, Task-008 |
| FND-6004 | low | External runtimes remain unavailable/unsupported inputs, never executable dependencies or semantic authorities. | FR-010, Task-006 |

## Responsibility allocation

| Requirement | Owning component | Class |
|---|---|---|
| FR-008 | tl-mltl campaign schema/census | core |
| FR-009 | named family repository; tl-mltl integrates manifests | infrastructure |
| FR-010 | tl-mltl C2PO/R2U2 overlay; quire-contract-ir owns FRETish/native producers | core |
| NFR-004 | tl-mltl report producer plus shared Quire/Quoin intake | cross-cutting |

## External dependencies

| Dependency | Assumed or guaranteed | Contract |
|---|---|---|
| landed TL future/W/M revisions | guaranteed by exact source and manifest digest | MRS-002 revision list, Task-002 |
| native predicate/temporal bridge | blocked, then guaranteed by exact owner digest | quire-contract-ir #63/#64, Task-008 |
| Quire/Quoin/Engineering Assurance | guaranteed only at the declared pinned identities; module drift disclosed | FR-006, assurance/pins.json |
| C2PO/R2U2/FRET runtimes | assumed unavailable/unsupported and never executed | FR-010 target catalog |
