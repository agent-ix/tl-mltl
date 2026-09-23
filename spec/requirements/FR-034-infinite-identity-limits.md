---
id: FR-034
title: Preserve infinite result identity and work limits
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-033
    type: depends_on
---

# FR-034: Preserve infinite result identity and work limits

## Description

When the infinite provider accepts a request, it shall preserve the exact
source, formula, subject kind/identity, trace, fairness, proposition-map, profile and event-position
clock identities and enforce configured resource bounds before returning a
complete result.

## Behavior

Identity validation precedes semantic work. Every accepted result names the
TL profile, feature-enabled provider revision, graph, subject and trace identities,
selected event position, clock and ordered fairness premises. Checked
arithmetic and explicit node, lasso, valuation, state, iteration and work
limits prevent wrapping, uncontrolled allocation and partial success. Work
exhaustion returns FR-033's `failed` resource-incomplete result with no
proof or counterexample claim. The default bounded build adds no
provider-only dependency; the feature build retains bounded golden bytes.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-034-AC-1 | Each identity axis is preserved or yields a typed pre-evaluation refusal when mismatched; no mixed-owner result is emitted. | Test (TC-159) |
| FR-034-AC-2 | At-limit work completes, one-over each limit is resource-incomplete, and checked arithmetic never wraps or panics on caller input. | Test (TC-159) |
| FR-034-AC-3 | Repeated runs with identical pins and inputs produce byte-identical results; bounded golden bytes agree between feature-off and feature-on builds. | Test (TC-138, TC-159) |

## Dependencies

FR-027 owns the feature boundary; FR-033 owns result settlement.
