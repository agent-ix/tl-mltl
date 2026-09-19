---
id: FR-027
title: State the infinite-trace crate boundary
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-001
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/quire-specification/AD-002
    type: references
---

# FR-027: State the infinite-trace crate boundary

## Description

tl-mltl shall evaluate closed (FR-001) and prefix (FR-003) bounded finite
traces. Lasso-word acceptance, fairness-restricted admission, and the
FR-161-equivalent inductive always/eventually/until/release and past-dual
infinite-trace semantics belong to a separate provider under the
`quire.temporal.infinite-trace/v1` facet, which tl-mltl composes with rather
than contains.

## Inputs

- A bounded MLTL graph and finite trace, exactly as FR-001 and FR-003 already
  accept: a `tl-syntax.formula/v1` document and its context-bound v2 records,
  which together are the whole of tl-mltl's formula input surface.

## Outputs

- The existing FR-001/FR-003 closed and prefix verdicts, unchanged.
- Infinite-trace dispositions are the registered provider's, under FR-028; a
  claim no registered backend can discharge settles `unsupported` with a
  warning naming the required capability (`tl-syntax.liveness/v1`, tl-syntax
  FR-290).

## Behavior

`tl-syntax.formula-unbounded/v1` is a distinct, co-existing wire edition from
the `tl-syntax.formula/v1` document tl-mltl's evaluator consumes (tl-syntax
FR-289, tl-syntax [#73](https://github.com/agent-ix/tl-syntax/issues/73));
tl-mltl's parsers and evaluator entry points admit `tl-syntax.formula/v1` and
its context-bound v2 records, and that admitted surface is the whole of what is
structurally reachable from this crate.

The infinite-trace provider owns lasso-word acceptance, fairness-restricted
admission, and the inductive always/eventually/until/release and past-dual
semantics tl-mltl [#68](https://github.com/agent-ix/tl-mltl/issues/68) tracks.
It consumes `tl-syntax.formula-unbounded/v1` graphs independently of tl-mltl,
matching the FR-006-through-FR-007 rule that each shared consumer binds only
the exact upstream identities it needs. Which crate carries that provider is
an owner decision, recorded as an open question in
[AD-002](../assurance/AD-002.md); this requirement binds the boundary and the
facet identity, not a crate name.

tl-mltl's own bounded evaluator, horizon analysis, prefix semantics, and
monitor mapping are unaffected: this requirement changes no existing FR-001
through FR-019 behavior, and closure-as-false, pending-verdict, and
work-budget semantics remain exactly as already specified.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-027-AC-1 | tl-mltl's public API and CLI admit `tl-syntax.formula/v1` documents and their context-bound v2 records as the whole of their formula input surface. | Test (TC-086) |
| FR-027-AC-2 | `spec/spec.md` allocates lasso-word, fairness, and infinite-trace inductive semantics to a provider under the `quire.temporal.infinite-trace/v1` facet and states that tl-mltl evaluates closed and prefix bounded traces. | Inspection (TC-086) |
| FR-027-AC-3 | Every existing FR-001 through FR-019 acceptance criterion continues to pass unchanged. | Test (TC-086) |

## Dependencies

Depends on FR-001's closed evaluation and FR-003's prefix semantics, the two
bounded entry points this requirement bounds. References
quire-specification AD-002's provider-independent infinite-trace model, the
architectural pattern this boundary follows. FR-028 supplies the liveness
capability registration boundary; FR-029 supplies downstream evidence and
dependency order.
