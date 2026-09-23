---
id: FR-027
title: Keep bounded entry points separate from the opt-in infinite provider
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-001
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-mltl/decisions/ADR-003
    type: references
---

# FR-027: Keep bounded entry points separate from the opt-in infinite provider

## Description

When the `infinite-trace` Cargo feature is disabled, tl-mltl shall expose only
its existing bounded evaluator entry points. When it is explicitly enabled,
`tl_mltl::infinite` shall expose a distinct provider for
`mltl.infinite-trace/v1` without changing those bounded entry points.

## Inputs

- Existing formula v1/v2 and finite closed, prefix or origin-complete history
  inputs for the bounded API.
- Formula-unbounded/v1, lasso, fairness and partial-valuation documents for
  the feature-gated provider.

## Outputs

- Existing bounded verdicts unchanged, and a separate infinite result type
  only when the feature is enabled.

## Behavior

The bounded core does not import `infinite::*` or infer an infinite profile from
unbounded text. The provider does not route an infinite request through
closure-as-false, bounded horizon or finite-prefix pending logic. Both paths
use their exact owner graph and clock identities. The default build excludes
provider-only dependencies and public APIs. The feature-enabled build retains
all existing bounded behavior and wire bytes. A consumer's Cargo feature
selection is explicit and inspected in its resolved dependency graph. The
baseline crate is std-only; this feature does not assert `no_std` support.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-027-AC-1 | Default builds expose existing bounded APIs but no infinite provider; opt-in builds expose `tl_mltl::infinite` and retain bounded API and wire bytes. | Test (TC-086, TC-138) |
| FR-027-AC-2 | Module-import and dependency-tree checks show no bounded import of `infinite`, no provider-only dependency when disabled, and no implicit feature selection from tl-rewrite. | Test (TC-138) |
| FR-027-AC-3 | Every existing FR-001 through FR-019 criterion and golden byte remains valid in both feature sets. | Test (TC-086, TC-138) |

## Dependencies

FR-001 and FR-003 own bounded semantics. ADR-003 records the feature decision;
FR-028 owns the provider's liveness registration.
