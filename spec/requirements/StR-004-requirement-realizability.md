---
id: StR-004
title: Requirement authors need a reference answer for whether a requirement set can be implemented
type: StR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-043
    type: satisfied_by
  - target: ix://agent-ix/tl-solve/StR-100
    type: references
---

# StR-004: Requirement authors need a reference answer for whether a requirement set can be implemented

## Stakeholder Need

Requirement authors and assurance reviewers need a reference answer, computed
directly from the TL closed-trace semantics, to the question of whether some
implementation can satisfy a set of MLTL requirements against every possible
sequence of environment inputs, so that the faster tl-solve decision procedures
have an independent source of truth to be compared against.

## Rationale

A requirement set can be satisfiable and still unimplementable because it only
holds when the environment cooperates. The production procedures that decide
this (tl-solve) use progression, search and solver encodings. A wrong answer
from them would be an unsafe claim about a safety requirement. The reference
must therefore be small enough to inspect, share no code with those
procedures, and be defined from the same evaluation function that defines a
satisfied requirement.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-004-VC-1 | For a declared population of small requirement sets with declared input and output splits, the reference answer equals an independently computed answer, and the distinction between a set that fails for every environment input and a set that fails only because the response depends on a later input is visible in the reference's output. | Test (TC-188, TC-189, TC-192) |
| StR-004-VC-2 | A reviewer can tell from the reference's output which trace length was played, where that length came from, and that no more than the declared work was done. | Test (TC-190, TC-191) |

## Stakeholders

Requirement authors, assurance reviewers, and the maintainers of tl-solve who
compare their search against this reference.

## Context and Assumptions

The semantics are fixed by [ADR-004](../decisions/ADR-004-requirement-realizability-semantics.md).
Input and output direction is declared on the tl-syntax signal catalog
(`ix://agent-ix/tl-syntax/FR-034`). Controller synthesis is out of scope.

## Traceability

This need is realized by FR-043 and verified by TM-007.
