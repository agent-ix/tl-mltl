---
id: FR-028
title: Register the opt-in infinite liveness backend once
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-290
    type: references
---

# FR-028: Register the opt-in infinite liveness backend once

## Description

When `tl_mltl::infinite` is enabled, it shall be the sole registrant of
`tl-syntax.liveness/v1` for a deployment that selects this backend and shall
settle admitted formula-unbounded requests using the FR-341 result vocabulary.

## Inputs

- Exact capability, TL profile, graph, clock, fairness, subject kind and
  subject identity. V1 subject kinds are a complete lasso, a finite prefix or
  a transition/model request.
- Provider configuration and bounded work limits.

## Outputs

- A `proved`, `refuted`, `inconclusive`, `unsupported` or `failed`
  disposition with typed detail and attributable provider identity.

## Behavior

Without a registered provider, every formula-unbounded request follows
tl-syntax FR-290's `unsupported` absence path and names
`tl-syntax.liveness/v1`. With this provider selected, each request reaches
only the opt-in module. The V1 provider can decide an admitted complete lasso
as a claim about that exact trace, and may refute a decisive finite bad
prefix. It has no model-wide procedure; a transition/model request is
`unsupported` with a typed missing-model-capability detail. Another
registrant for the same deployment is a
configuration conflict, never a precedence choice. A finite prefix cannot
yield `proved` liveness; timeout or resource exhaustion yields `failed`
with a resource-incomplete execution disposition. Syntax/profile/clock refusals remain
`unsupported`, and internal failure remains `failed`. Existing bounded
verdicts, reports and CLI schemas keep their bytes and meanings.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-028-AC-1 | The feature-off absence path yields `unsupported` with the required capability warning, and the feature-on provider registers exactly once. | Test (TC-087, TC-139) |
| FR-028-AC-2 | Two registrants, subject/identity mismatch, or a model request without model-wide capability refuse without routing to a bounded evaluator or returning a partial result. | Test (TC-139) |
| FR-028-AC-3 | Every existing bounded verdict, report and CLI schema is byte-identical with the feature enabled or disabled. | Test (TC-087, TC-138) |

## Dependencies

FR-027 owns module separation; tl-syntax FR-290 owns the capability identity
and absence settlement. FR-030 through FR-034 own provider semantics.
