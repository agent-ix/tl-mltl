---
id: FR-041
title: Refuse unsupported C2PO profiles exhaustively
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-038
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-040
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
---

# FR-041: Refuse unsupported C2PO profiles exhaustively

## Description

When rendering C2PO output, the adapter shall first classify every formula node,
interval kind, valuation kind and premise combination as mapped or a distinct
typed refusal, with no catch-all that silently accepts a future grammar form.

## Behavior

The refusal partition distinguishes unbounded liveness F/G outside the exact
outer safety guard, unbounded U/R, unbounded past outside the supported inner
fragment, fairness premises, partial valuations, unsupported signal names,
clock/profile mismatch, target-origin mismatch and resource exhaustion.
Unknown enum additions fail closed. An ordinary MappingError stays a typed
mapping refusal, not a temporal verdict. If projected into an FR-341 result,
an unsupported export is `unsupported` with execution `unsupported`, truth
`unavailable` and basis `unavailable`; resource exhaustion is `failed` with
execution `resource-incomplete`. No refusal emits a partial expression or
manifest. Every recognized node × interval kind appears in the declarative
classification table and has a positive or negative test.

| Node or context | Closed interval | Unbounded interval | Refusal if not admitted |
|---|---|---|---|
| False, True, Proposition, Not, And, Or, Implies, Equivalent | Map under validated catalog and profile | No interval form | Unsupported signal or profile |
| F, G, U, R inside `ψ` | Map only with a finite target-equivalent horizon | Refuse; only the exact outer `G[0,)` is special | Unbounded liveness or until/release |
| Exact outer `G[0,)ψ` | Not this fragment | Map as refutation-only if every child maps | Unsupported safety shape |
| O, H, S, T inside `ψ` or in past profile | Map only with target-equivalent origin | Map only when target supports the same finite-origin meaning | Target-origin mismatch |
| Y | Map only as strong previous under the origin contract | No interval form | Target-origin mismatch |
| Fairness premise, missing/conflicting valuation | Refuse | Refuse | Fairness or partial valuation |
| Foreign clock, profile, graph or target version | Refuse | Refuse | Identity mismatch |
| Exhausted work limit | Refuse | Refuse | Resource-incomplete |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-041-AC-1 | The complete node × interval × context table has one mapped/refused class per combination and no wildcard acceptance. | Test (TC-172) |
| FR-041-AC-2 | Each refusal keeps its typed cause, FR-341 projection where applicable, and emits no artifact; unknown additions fail closed. | Test (TC-173) |

## Dependencies

FR-038/040 own the two new export paths; FR-341 owns projected result labels.
