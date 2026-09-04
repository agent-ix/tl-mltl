---
id: StR-003
title: Preserve typed temporal context through reference results
type: StR
---

# StR-003: Preserve typed temporal context through reference results

## Stakeholder Need

Temporal frontend developers and assurance reviewers need evaluation, resource,
monitor-mapping, and differential results that remain attributable to the exact
shared signal catalog and, when supplied, the exact requirement revision and
clause source associated with the formula.

## Rationale

Synthetic proposition aliases and formula/trace ids do not identify the named
bounded inputs or requirement clause whose behavior is being evaluated. A
semantically correct verdict attached to substituted context is still an
incorrect and potentially unauditable result.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-003-VC-1 | Context-aware evaluation, horizon, mapping, external-verdict, and differential records preserve the exact shared catalog identity and optional requirement context together with every participating tl-* revision. | Test (TC-025, TC-028, TC-031) |
| StR-003-VC-2 | C2PO mapping uses supported shared Boolean signal names without invention, and changed, absent, unsupported, or mismatched context cannot produce agreement or an executable mapping artifact. | Test (TC-026, TC-027) |

## Stakeholders

Temporal frontend developers, monitor integrators, assurance reviewers, and the
human source-release owner.

## Context and Assumptions

The catalog and optional caller context are the shared validated types from the
exact tl-syntax revision. tl-mltl validates their use but neither derives them
from contract IR nor establishes that the caller's provenance statement is
true. The future FRETish/IR adapter remains outside this crate.

## Traceability

This need is realized by FR-007 and verified by TM-001.
