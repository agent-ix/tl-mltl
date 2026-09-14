---
id: FR-018
title: "Publish strict temporal requests, results, and Contract-IR mapping"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-014
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-syntax/IF-004
    type: implements
  - target: ix://agent-ix/tl-syntax/VO-006
    type: implements
---
# FR-018: Publish strict temporal requests, results, and Contract-IR mapping

## Description

When a future or origin-complete past assessment is requested, tl-mltl SHALL
strict-read every exact owner artifact, execute only its selected bounded
profile, emit one immutable canonical temporal result, and publish a separate
selected Contract-IR mapping view.

## Architecture and owner contracts

The implementation SHALL be organized as `future`,
`past::{history,requirement,evaluate,result}`, `wire::{trace,request,report}`,
`clock`, and `mapping`, preserving all existing public paths by re-export.

The strict owner set includes existing `tl-mltl.trace/v1`,
`tl-mltl.command/v1`, `tl-mltl.position-history/v1`,
`tl-mltl.history-requirement/v1`, and `tl-mltl.past-evaluation/v1`, plus:

- `tl-mltl.temporal-assessment-request/v1`, which binds an owner-read formula,
  trace or history, anchor, evaluator and exact observation progress/closure/
  completeness selections; and
- `tl-mltl.temporal-assessment-result/v1`, which binds the evaluated truth or
  non-value to every request/artifact/owner identity, support, limits and
  correction relation; and
- `tl-mltl.contract-ir-result-map/v1`, a deterministic normalized view derived
  only from a validated temporal result.

Every contract SHALL publish immutable schema bytes/digest and a bounded public
`read(bytes, expected, limits)` returning a constructor-private validated view.

## Request and evaluation

A request selects exactly one lane:

- formula-v1 plus trace-v1 under `mltl.closed-trace/v1` or
  `mltl.online-prefix/v1`; or
- formula-v2 plus position-history-v1 under
  `mltl.origin-complete-history/v1` and `tl-syntax.past-operators/v1`.

The request retains formula, trace/history, proposition map, clock, anchor,
evaluator, decision-scope progress/closure, surrounding-execution progress/
closure, completeness and availability contract identities/revisions/digests.
The four progress/closure assertions each carry only independent `open` or
`closed`; completeness independently carries `complete`, `incomplete` or
`contradicted`. TL validates and echoes those owner facts but never derives one
from another.

Future semantics remain FR-001/FR-003. Past semantics remain the complete
O/H/Y/S/T, history requirement, origin/cutoff, exact clock and immutable
correction behavior in tl-syntax FR-011/012. Past requests refuse future/mixed
nodes; future requests refuse past nodes. `history-cutoff` cannot authorize
pre-origin false extension. Timestamp/dense, implicit sampling, rounding and
wall-clock advance remain unsupported.

The temporal result has exactly one assessment execution and one truth/non-value
state, preserves the four owner assertion refs/states, completeness ref/state/
facts, settlement, exact decision support and original/superseding/invalidating
direct-predecessor relation. A correction has a greater revision and new
identity; prior bytes never change. No result is emitted from an unadmitted
request or partial evaluation.

## Contract-IR mapping

`mapping::contract_ir::map(&ValidatedTemporalResult, MappingSelection, Limits)`
and the companion strict reader return a constructor-private mapped view. The
mapping view contains the source result/request/correspondence/formula/input
identities, assessment execution, `value` or `nonValue`, all four independent
progress/closure owner references/states, completeness reference/state/facts,
settlement, exact support and correction/direct predecessor.

Only a valid completed final TL Boolean maps to `value:true` or `value:false`.
Pending, unavailable, incomplete, unsupported, failed, refused and contradicted
rows map to typed `nonValue` with every source fact needed for lossless joining.
The mapping is re-derived entirely from the validated result; it accepts no
caller-supplied result fields, label parser, callback, trust flag or Contract-IR
wire type.

The temporal result and mapping identities are separate domain-separated
lowercase SHA-256 values over their exact canonical bytes with `identity`
omitted. A reader independently re-derives the applicable bytes/identity and
rejects substitution between source result, mapping, request, formula,
trace/history or schema identities.

## Strict reading and limits

Every reader rejects unknown/duplicate/missing/out-of-order fields, trailing
data, invalid UTF-8, noncanonical bytes, unknown contract/profile/operator/
state, invalid topology/order/gaps, stale identity/digest, wrong anchor/clock/
owner assertion, impossible result combination and invalid correction lineage.
Limits independently bound bytes, depth, strings, formula nodes, positions,
propositions, support, history span, evaluator steps, recursion and visited
fields. Work is charged before allocation/traversal; exact limits pass and
one-over/allocation failure yields one typed non-value/refusal and no partial
view.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-018-AC-1 | Every future and pure-past positive request strict-reads, evaluates under exactly its selected profile, emits deterministic canonical result bytes, and strict-reads to a constructor-private view. | Test (TC-084) |
| FR-018-AC-2 | Every owner contract publishes exact schema bytes/digest; unknown/duplicate/missing/reordered fields, trailing/noncanonical bytes and each identity/contract mutation refuse without partial output. | Test (TC-084) |
| FR-018-AC-3 | Four progress/closure axes and completeness remain independently representable and are echoed from exact owner assertions without derivation or vocabulary substitution. | Test (TC-084) |
| FR-018-AC-4 | The mapping derives a Boolean only from valid final Boolean results and losslessly maps every other execution/truth/completeness row to a typed non-value revalidated against the source result. | Test (TC-084) |
| FR-018-AC-5 | Origin/cutoff, future/past/mixed, clock/sample, decision-premise/settlement and correction mutations reach their exact typed outcomes with no false padding, implicit position or predecessor rewrite. | Test (TC-084) |
| FR-018-AC-6 | Exact and one-over limits cover every byte/depth/count/history/evaluation dimension; existing future and delivered past public behavior remains unchanged across the module reorganization. | Test (TC-084) |

## Dependencies

FR-001/003 supply future evaluation, tl-syntax FR-011/012 supplies past
semantics/history, tl-syntax FR-014 supplies strict formulas, and Quire
Observation FR-004 supplies the assertion views that requests/results preserve.

## Status

Implemented complete TL evaluator owner boundary for `tl-syntax#52/#64` and
`quire-contract-ir/FR-026`; promotion is tracked by `tl-mltl#66/#67`.
