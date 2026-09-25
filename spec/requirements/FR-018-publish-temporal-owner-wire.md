---
id: FR-018
title: "Publish TL-owned temporal artifact contracts"
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
  - target: ix://agent-ix/tl-syntax/IF-004
    type: implements
  - target: ix://agent-ix/tl-syntax/VO-006
    type: implements
---
# FR-018: Publish TL-owned temporal artifact contracts

## Description

tl-mltl SHALL publish its existing future- and past-evaluation artifact
contracts — `tl-mltl.trace/v1`, `tl-mltl.command/v1`,
`tl-mltl.position-history/v1`, `tl-mltl.history-requirement/v1`, and
`tl-mltl.past-evaluation/v1` — as immutable, schema-pinned, strictly-read
documents. tl-mltl does not itself define, bind, or publish any
owner-assertion request/result wire contract; a caller that wants to compose
this crate's evaluators with an external owner's assertion views (for example
Quire Observation's `authority::{clock,progress,closure,completeness,availability}`
views) does so entirely outside this crate, through a dedicated integration
crate. That boundary is an architect ruling (Linear epic TL-175): every
`tl-*` crate stays independent of the Quire ecosystem, and only a purpose-built
integration crate may bridge them. `quire-mltl` is that crate for the
Quire Observation pairing; see its FR-001/FR-002.

TL-179 removed this crate's `quire-observation` and
`agent-ix-baseline-producer` dependencies along with the
`tl-mltl.temporal-assessment-request/v1`, `tl-mltl.temporal-assessment-result/v1`,
and `tl-mltl.contract-ir-result-map/v1` contracts, the `wire::request`,
`wire::report`, and `wire::observation` modules, and the `mapping::contract_ir`
module that published them. Those contracts and the QObs C00 compatibility
dispatch built on them (formerly FR-019) now live in `quire-mltl`, ported
unchanged in evaluation behavior. This requirement's prior text described
that now-removed layer; this revision instead documents the artifact
contracts tl-mltl actually still owns and publishes, which no other
requirement in this spec independently names.

## Architecture and owner contracts

The implementation SHALL be organized as `future`,
`past::{history,requirement,evaluate,result}`, `wire::{trace,command}`,
`clock`, and `mapping::legacy`, preserving all existing public paths by
re-export.

The owner set is exactly the five contracts above. Every contract SHALL
publish immutable schema bytes/digest and a bounded public
`read(bytes, expected, limits)` returning a constructor-private validated
view:

- `tl-mltl.trace/v1` (`wire::trace::ValidatedTrace`) — a finite ordered
  observation trace for the future evaluator.
- `tl-mltl.command/v1` (`wire::command::ValidatedCommand`) — the CLI's single
  request/response document.
- `tl-mltl.position-history/v1` (`past::history::ValidatedPositionHistory`) —
  an origin-complete position history for the past evaluator.
- `tl-mltl.history-requirement/v1`
  (`past::requirement::ValidatedHistoryRequirement`) — the checked required
  history/work report `analyze_required_history` emits.
- `tl-mltl.past-evaluation/v1` (`past::result::ValidatedPastResult`) — the
  immutable original/superseding/invalidating past result.

## Strict reading and limits

Every reader rejects unknown/duplicate/missing/out-of-order fields, trailing
data, invalid UTF-8, noncanonical bytes, unknown contract/schema identity, and
stale identity/digest. `OwnerLimits` independently bounds bytes, depth,
strings, formula nodes, positions, propositions, and history span across all
five readers; work is charged before allocation/traversal, and one-over
refuses with a typed error and no partial view.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-018-AC-1 | Every one of the five retained owner contracts publishes exact schema bytes/digest, and its bounded reader accepts only canonical bytes matching that digest, returning a constructor-private validated view. | Inspection (TC-090) |
| FR-018-AC-2 | Every reader rejects unknown/duplicate/missing/reordered fields, trailing/noncanonical bytes, and identity/digest mutation with a typed error and no partial output; every owner-declared resource ceiling is enforced before allocation or traversal. | Inspection (TC-090) |
| FR-018-AC-3 | Removing the `quire-observation`-coupled request/result/mapping layer (TL-179) changes no existing future evaluator, past evaluator, CLI schema, or `mapping::legacy` C2PO mapping behavior. | Inspection (TC-090) |

## Dependencies

FR-001/003 supply future evaluation and the `tl-mltl.trace/v1` /
`tl-mltl.command/v1` contracts it publishes; tl-syntax FR-011/012 supply past
semantics and the `tl-mltl.position-history/v1` / `tl-mltl.history-requirement/v1`
/ `tl-mltl.past-evaluation/v1` contracts built on them. This requirement adds
no evaluator semantics beyond those; it only documents the immutable
schema/digest/strict-reading discipline shared across all five.

## Status

The five retained contracts remain implemented. `tests/inspection_evidence.rs`
pins their published schema digests, exercises all five canonical readers,
malformed-input refusals, common and owner-specific `OwnerLimits` ceilings,
and one-over output limits under TC-090. The same test compares the exact
pre/post TL-179 source revisions and confirms that the retained evaluator,
CLI, owner-reader, schema, and legacy-mapping source paths did not change in
that removal. Current behavior is exercised by `tests/reference.rs`,
`tests/past_history.rs`, `tests/cli.rs`, and `tests/interop.rs`. TL-179 removed
the QObs-coupled request/result/mapping layer and its dedicated test
(`tests/tc_084_temporal_owner_wire.rs`, formerly TC-084) rather than porting it,
since that behavior now lives in `quire-mltl`.
