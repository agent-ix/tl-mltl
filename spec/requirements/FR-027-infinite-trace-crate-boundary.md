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

tl-mltl shall evaluate only closed (FR-001) and prefix (FR-003) bounded finite
traces. Lasso-word acceptance, fairness-restricted admission, and the
FR-161-equivalent inductive always/eventually/until/release and past-dual
infinite-trace semantics are owned by a named provider crate, `tl-live`, and
tl-mltl shall neither implement nor partially approximate them.

## Inputs

- A bounded MLTL graph and finite trace, exactly as FR-001 and FR-003 already
  accept.
- A `tl-syntax.formula-unbounded/v1` document (tl-syntax
  [FR-289](https://github.com/agent-ix/tl-syntax/blob/main/spec/requirements/FR-289-infinite-trace-interval-grammar.md)):
  not an input this crate accepts at all.

## Outputs

- The existing FR-001/FR-003 closed and prefix verdicts, unchanged.
- No lasso witness, fairness admission, or infinite-trace disposition of any
  kind; tl-mltl exposes no entry point that accepts an `UnboundedInterval` or a
  `tl-syntax.formula-unbounded/v1` document.

## Behavior

A `tl-syntax.formula-unbounded/v1` document is a distinct, co-existing wire
edition from the `tl-syntax.formula/v1` document tl-mltl's evaluator consumes
(tl-syntax FR-289); tl-mltl's parsers and evaluator entry points admit only
`tl-syntax.formula/v1` and its context-bound v2 records, so an unbounded
document is never structurally reachable, not merely refused at evaluation
time.

`tl-live` is the named provider crate that owns lasso-word acceptance,
fairness-restricted infinite-trace admission, and the inductive
always/eventually/until/release and past-dual semantics tl-mltl#68 originally
requested. `tl-live` consumes `tl-syntax.formula-unbounded/v1` graphs
independently of tl-mltl; it does not extend, wrap, or depend on the tl-mltl
crate to do so, matching the FR-006-through-FR-007 rule that each shared
consumer binds only the exact upstream identities it needs.

tl-mltl's own bounded evaluator, horizon analysis, prefix semantics, and
monitor mapping are unaffected: this requirement changes no existing FR-001
through FR-019 behavior, and closure-as-false, pending-verdict, and
work-budget semantics remain exactly as already specified.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-027-AC-1 | tl-mltl's public API and CLI expose no entry point that accepts `UnboundedInterval` or a `tl-syntax.formula-unbounded/v1` document; only `tl-syntax.formula/v1` and its context-bound v2 records are reachable. | Test (TC-086) |
| FR-027-AC-2 | `spec/spec.md` names `tl-live` as the owner of lasso-word, fairness, and infinite-trace inductive semantics, and states that tl-mltl evaluates only closed and prefix bounded traces. | Inspection (TC-086) |
| FR-027-AC-3 | Every existing FR-001 through FR-019 acceptance criterion continues to pass unchanged. | Test (TC-086) |

## Dependencies

Depends on FR-001's closed evaluation and FR-003's prefix semantics, the two
bounded entry points this requirement bounds. References
quire-specification AD-002's provider-independent infinite-trace model, the
architectural pattern this boundary follows. FR-028 supplies the liveness
capability registration boundary; FR-029 supplies downstream evidence and
dependency order.
