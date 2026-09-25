---
id: FR-030
title: Evaluate partial observations as possibilities
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-023
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-160
    type: references
---

# FR-030: Evaluate partial observations as possibilities

## Description

When the infinite provider receives partial valuations, it shall interpret
`true`, `false`, `missing`, and `conflicting` through QSpec FR-160's
possibility-set semantics, preserving the distinction between absent evidence
and contradictory evidence.

## Behavior

One completion assigns one Boolean value to each proposition at each lasso
position consistently across all references to that position, including loop
repetitions. The provider computes possible truth values for a formula from
those common completions; it does not choose a favorable completion per
subformula or operator. `missing` and `conflicting` each carry `{true, false}`
but keep different reasons and closure behavior; neither becomes Boolean
false. Known values carry singleton sets. Adding information cannot flip a
conclusive result. The
provider keeps exact graph, proposition-map, clock, trace and completion
identities. An empty possibility set is an inconsistent-input refusal, never
false or a vacuous proof. When closed evidence remains conflicting, a
two-valued result is indeterminate; when declared progress can resolve it,
it is pending. Both project to `inconclusive` with distinct typed details.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-030-AC-1 | Complete valuations agree with ordinary Boolean infinite semantics; missing values produce exactly the FR-160 possibility set under common completions. | Test (TC-141, TC-142) |
| FR-030-AC-2 | Conflicting and missing valuations keep distinct reasons and closure behavior despite equal possibility sets; empty sets refuse rather than becoming vacuous proof or refutation. | Test (TC-143) |
| FR-030-AC-3 | Repeated proposition references and loop visits use the same completion for one lasso position, and an identity mismatch refuses before evaluation. | Test (TC-144) |

## Dependencies

FR-027 owns the feature boundary; tl-syntax FR-023 owns the four-valued wire
input; QSpec FR-160 is the normative possibility-set authority.
