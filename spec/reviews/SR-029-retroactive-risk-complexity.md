---
id: SR-029
title: "Risk and complexity retrospective review of the tl-mltl specification corpus"
type: SpecReview
analysis: risk-complexity
scope: "spec/requirements, spec/assurance, test matrix, and current implementation at origin/main 5c4ce2a"
review_set: all
---

## Summary

The highest technical-risk obligations are the reference semantics/resource bounds,
shared-assurance producer boundary, and context-bound cross-repository records. Their
volatility is primarily external: pinned syntax, Engineering Assurance, Quire, Quoin,
and retained R2U2 artifacts can change independently. Current tests and exact pins are
appropriate mitigations, but they must be re-evaluated together on any contract advance.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | FR-006 and FR-007 combine material technical risk with external-contract volatility. Future pin or schema changes need one reviewed compatibility pass across the shared syntax contract, producer evidence, and contextual record families; updating only a Cargo revision or a fixture digest is insufficient. | FR-006-AC-1 through FR-006-AC-7; FR-007-AC-1 through FR-007-AC-7; NFR-002; NFR-003; `assurance/pins.json` |
