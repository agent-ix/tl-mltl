---
id: FR-040
title: Export only the monitorable infinite safety fragment
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-031
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-038
    type: depends_on
---

# FR-040: Export only the monitorable infinite safety fragment

## Description

When an infinite-trace formula has the exact shape `G[0,)ψ` and ψ contains
only admitted past or bounded-future operations in any nesting, the opt-in
provider may export a C2PO monitor expression as refutation-only evidence.

## Behavior

The export resides behind `tl_mltl::infinite` and consumes only validated
formula-unbounded/v1 under `mltl.infinite-trace/v1`. A per-node, per-interval
classification admits the outer `G[0,)` and only inner operators whose
finite-horizon verdict is sound under the selected target's clock and origin
contract. Any mixed nesting must have a finite decision horizon at each
position; otherwise it refuses before output. A violating observation may
establish a replayable finite bad
prefix; absence of violation, a target pass, or an incomplete observation
never yields `proved`. The adapter does not register as a second liveness
backend. Its manifest names its refutation-only scope, source and target
identities and exact input and output digests.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-040-AC-1 | Every admitted `G[0,)ψ` export preserves the profile and yields only refutation or non-conclusive comparison, never proof. | Test (TC-168, TC-169) |
| FR-040-AC-2 | Finite bad-prefix replay establishes each reported violation under the provider's infinite semantics. | Test (TC-170) |
| FR-040-AC-3 | Feature-off builds expose no infinite export and retain bounded mapping bytes. | Test (TC-171) |

## Dependencies

FR-031 owns infinite semantics; FR-038 owns target mapping conventions.
