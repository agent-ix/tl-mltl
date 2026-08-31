---
id: StR-001
title: Provide reviewable reference semantics
type: StR
---

# StR-001: Provide reviewable reference semantics

## Stakeholder Need

Temporal-tool developers need one deterministic, side-effect-free evaluator
whose finite-trace assumptions are explicit and whose results can serve as an
oracle without claiming production-monitor qualification.

## Rationale

Independent temporal tools need a common oracle to detect disagreement without
embedding production-monitor assumptions into syntax or rewrite crates.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-001-VC-1 | The library evaluates every tl-syntax v1 operator under a named profile and returns traceable identities. | Test |
| StR-001-VC-2 | Documentation states that reference results do not accredit a consuming monitor. | Inspection |

## Stakeholders

Temporal-crate developers, monitor integrators, assurance reviewers, and the
human v0.1 release owner.

## Context and Assumptions

Inputs have passed the exact tl-syntax structural validation contract.

## Traceability

This need is realized by FR-001 and FR-005 and verified by TM-001.
