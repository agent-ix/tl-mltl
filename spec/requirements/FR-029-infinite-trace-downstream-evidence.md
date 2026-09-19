---
id: FR-029
title: Route infinite-trace downstream evidence and dependency order
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-028
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-291
    type: depends_on
---

# FR-029: Route infinite-trace downstream evidence and dependency order

## Description

tl-mltl issue [#68](https://github.com/agent-ix/tl-mltl/issues/68)'s
lasso-witness, fairness, and infinite-trace inductive-semantics scope shall be
routed to a new `tl-live` implementation ticket, and its exit criterion shall
no longer name tl-mltl's own `spec/spec.md` scope, which FR-027 excludes it
from.

## Inputs

- The FR-027/FR-028 boundary statement this requirement routes.
- tl-syntax FR-291's dependency order, which this requirement extends by one
  further step.

## Outputs

- A recorded dependency order from tl-syntax's admitted grammar through
  `tl-live`'s registration, naming one owner and one predecessor per step.
- tl-mltl#68's scope brought into agreement with `spec/spec.md`, per this
  ticket's (tl-mltl#72) exit criterion.

## Behavior

Dependency order:

1. tl-syntax issue [#73](https://github.com/agent-ix/tl-syntax/issues/73)
   owns the `UnboundedInterval` value, the `tl-syntax.formula-unbounded/v1`
   document, and the `tl-syntax.liveness/v1` registration boundary.
2. [quire-specification#112](https://github.com/agent-ix/quire-specification/issues/112)
   mints the `quire.temporal.infinite-trace/v1` facet member tl-syntax admits
   under.
3. A new `tl-live` implementation ticket, opened against a `tl-live` repository
   this PR does not create, owns lasso-witness acceptance, fairness-restricted
   admission, the FR-161-equivalent inductive semantics, and the
   `tl-syntax.liveness/v1` registration FR-028 names. It replaces tl-mltl#68 as
   the ticket that scope routes to; tl-mltl#68 is superseded by this routing
   rather than implemented in this crate.

This requirement adds no new dependency onto tl-mltl's existing FR-001 through
FR-019 evaluator; it records where work outside this crate's boundary is
tracked so tl-mltl#68's own exit criterion can be brought into agreement with
`spec/spec.md`, as tl-mltl#72 requires.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-029-AC-1 | The stated dependency order names tl-syntax#73, quire-specification#112, and a `tl-live` implementation ticket, in that order, as the path from admitted grammar to registered liveness backend. | Inspection (TC-088) |
| FR-029-AC-2 | tl-mltl#68's scope is recorded as routed to `tl-live`, not implemented in tl-mltl, and `spec/spec.md`'s scope statement and tl-mltl#68 no longer disagree. | Inspection (TC-089) |
| FR-029-AC-3 | No dependency link in this requirement authorizes implementation ahead of tl-syntax FR-289/FR-290's own acceptance gates. | Inspection (TC-088) |

## Dependencies

Depends on FR-027's boundary statement and FR-028's registration routing.
Depends on tl-syntax FR-291's downstream-evidence and dependency-order pattern,
which this requirement extends by naming `tl-live` as the concrete step FR-291
left as tl-mltl#68/#72's routing responsibility.
