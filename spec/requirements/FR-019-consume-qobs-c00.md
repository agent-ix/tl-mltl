---
id: FR-019
title: Consume the pinned QObs C00 temporal boundary without semantic substitution
type: FR
status: superseded
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-018
    type: depends_on
  - target: ix://agent-ix/quire-observation/FR-009
    type: references
  - target: ix://agent-ix/quire-observation/FR-010
    type: references
  - target: ix://agent-ix/quire-observation/FR-011
    type: references
  - target: ix://agent-ix/quire-mltl/FR-002
    type: superseded_by
---

# FR-019: Consume the pinned QObs C00 temporal boundary without semantic substitution

## Description

When a caller selects a QObs C00 handoff, tl-mltl shall bind the exact compiled
QObs revision and shall dispatch only the already-owned bounded temporal request
adapter. If the caller selects the QObs repair-plan or closed-population-query
contract, then tl-mltl shall return an explicit typed unsupported outcome
without accepting, inspecting, projecting, evaluating, or relabeling a foreign
artifact.

## Inputs

- The exact compiled `quire-observation` revision
  `2bdeb833a330bfa777c19eb4c28c423f856f3ba6`.
- For the temporal path, the existing [FR-018](./FR-018-publish-temporal-owner-wire.md)
  request input containing constructor-private clock, progress, closure,
  completeness, and availability views plus caller-lowered owner limits.
- For unsupported paths, the closed QObs FR-010 repair-plan or FR-011
  closed-population-query contract selector.

## Outputs

- The byte-identical canonical FR-018 temporal request document produced by the
  existing bounded adapter.
- A closed, machine-matchable unsupported outcome naming the exact foreign
  contract and compiled QObs revision for repair and query inputs.
- The existing typed owner-read error when temporal input or bounds are invalid;
  no partial request or fallback outcome is emitted.

## Behavior

- The consumer dispatch shall delegate temporal request production to
  `wire::request::derive` under the supplied `OwnerLimits` and shall not duplicate
  temporal evaluation or observation-authority logic.
- The temporal dispatch output shall be byte-identical to a direct invocation of
  the existing adapter for the same inputs and limits.
- The repair branch shall identify `quire.observation.repair-plan/v1` as
  unsupported and shall not accept, inspect, or mutate a plan artifact.
- The query branch shall identify
  `quire.observation.closed-population-query/v1` as unsupported and shall not
  accept, inspect, aggregate, or mutate an evaluation artifact.
- Every supported or unsupported result shall identify the same exact compiled
  QObs revision exported by tl-mltl; no branch may substitute a compatible range,
  ambient checkout, ingestion order, or TL-owned reconstruction.

## Constraints

| ID | Constraint | Type | Validation |
|---|---|---|---|
| FR-019-CON-1 | The QObs dependency and exported revision shall name the same exact 40-hex commit | Provenance | Test |
| FR-019-CON-2 | Unsupported repair and query dispatch shall not expose a Boolean, aggregate, repaired result, or partial temporal document | Integrity | Test |

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-019-AC-1 | Given valid temporal owner views, dispatch produces bytes and resource usage identical to direct FR-018 request derivation under exact and one-over limits. | Test (TC-085) |
| FR-019-AC-2 | Given the QObs repair-plan or closed-population-query contract selector, compatibility dispatch returns the corresponding typed unsupported contract and exact compiled revision with no value-bearing or artifact input/output. | Test (TC-085) |
| FR-019-AC-3 | Cargo resolution, the public revision constant, request provenance, and compatibility outcomes all name `2bdeb833a330bfa777c19eb4c28c423f856f3ba6`; the existing temporal owner suite remains unchanged in behavior. | Test (TC-085) |

## Dependencies

This requirement advances the accepted FR-018 consumer boundary from its prior
QObs pin to the accepted C00 owner revision merged by QObs PR #27. That
enablement prerequisite is now satisfied. Any later owner revision requires a
repin and renewed compatibility review. QObs retains ownership of revisioned I07 bundles,
repair planning/coordinator behavior, and aggregate-query semantics. tl-mltl
owns only temporal request/evaluation semantics and the explicit compatibility
disposition at its boundary.

## Supersession

This requirement is superseded, not deleted or rewritten. The architect ruled,
against Linear epic [TL-175](https://linear.app/agent-ix/issue/TL-175), that
every TL-* crate — `tl-syntax`, `tl-parse`, `tl-mltl`, `tl-rewrite` — stays
independent of the agent-ix/Quire ecosystem, and that only a dedicated
integration crate may bridge one of them to Quire types. `quire-mltl` is that
bridge (TL-176/TL-177); TL-178 ported this requirement's QObs C00
compatibility-dispatch boundary there unchanged as
[quire-mltl FR-002](ix://agent-ix/quire-mltl/FR-002). TL-180 retires this
requirement in `tl-mltl`, removes MRS-001's `quire-contract-ir/PGM-01`
`depends_on` edge, and removes NFR-002's PGM-01 reference, completing
`tl-mltl`'s independence from Quire governance.

The Description, Inputs, Outputs, Behavior, Constraints, and Acceptance
Criteria above are retained unmodified as the historical record of what this
requirement specified while it governed `tl-mltl`'s own QObs boundary; they no
longer describe `tl-mltl`'s current or target behavior. The implementation
they describe is not removed by this ticket — dropping the `quire-observation`
dependency and the four wire/mapping source files is separate, tracked
follow-up work (TL-179's PR B) — so `src/wire/observation.rs` and TC-085 still
exist and still pass at the time this requirement is marked superseded.
SR-044 through SR-051, which reviewed the prior FR-018/FR-019 implementation
and PR [#67](https://github.com/agent-ix/tl-mltl/pull/67), are historical
record and are not edited by this supersession; see
[ADR-001](../decisions/ADR-001-retire-pgm01-citation-and-fr-019.md) and
[SR-052](../reviews/SR-052-retire-pgm01-citation-and-fr-019.md) for the
reversal itself.
