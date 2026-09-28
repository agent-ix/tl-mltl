---
id: FR-048
title: State and check bounded Kani proof claims
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-043
    type: depends_on
---

# FR-048: State and check bounded Kani proof claims

## Description

When a Kani claim is made, the owning crate shall run its exact harness with
stated symbolic domains, assumptions, loop and recursion bounds, unwind
checks and source/tool/solver identities.

## Behavior

Candidate invariants include checked interval arithmetic, strict graph
validation, no panic on bounded input and finite-step semantic equivalence.
A proof is only "proved within bounds" when every assertion and applicable
unwind/path check succeeds. Unknown, timeout, vacuous assumption, disabled
check or partial run is non-conclusive. A counterexample retains concrete
replay bytes. Verifier-only visibility must not alter the production subject
body; ordinary and MSRV tests detect divergence.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-048-AC-1 | Each claim records and checks its exact bounds, assumptions, source, solver and enabled checks; vacuity or incomplete unwind cannot pass. | Test (TC-185) |
| FR-048-AC-2 | A seeded false assertion yields a reproducible counterexample and verifier-only code leaves the production body unchanged. | Test (TC-186) |

## Dependencies

FR-043 provides an independent semantic comparison for semantic proof claims.
