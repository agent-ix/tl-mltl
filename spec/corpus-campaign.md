---
id: MRS-002
title: "MLTL corpus coverage and interoperability campaign"
type: MasterRequirements
status: superseded
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/issues/38
    type: references
  - target: ix://agent-ix/tl-syntax/MRS-002
    type: depends_on
  - target: ix://agent-ix/tl-syntax/MRS-003
    type: depends_on
---

# MLTL corpus coverage and interoperability campaign

> Historical campaign artifact, superseded for V1 by MRS-004 and TM-006.
> Existing implemented tests and retained evidence remain available.

## Purpose

This specification defines a versioned, reviewable coverage model for the TL
ecosystem corpus. It separates canonical conformance fixtures, generated test
populations, retained external observations, and qualification claims while
making every applicable or excluded coverage cell explicit.

The canonical `tl-syntax` formula graph is this campaign's single source
authority. The `tl-parse` text dialect is an internal representation of that
same graph, and R2U2/C2PO is a mapping target rather than a source authority or
a qualification dependency.

This campaign is scoped to the TL-owned families only. Under the TL-175
architect ruling every TL-* crate stays independent of the agent-ix/Quire
ecosystem, so the native-predicate and native-temporal correspondence
dimension — whose producer is `quire-contract-ir` — is not owned here. It is
specified and routed in `quire-mltl`, the ruling's one deliberate bridge crate.
See [ADR-002](./decisions/ADR-002-native-correspondence-lives-in-quire-mltl.md).

## Scope

### In scope

- Operator, interval-class, trace/history-shape, semantic-profile, outcome, and
  resource-boundary coverage cells.
- Strict fixture identities, manifests, digests, expected results, ownership,
  license/provenance, retention, and promotion rules.
- Explicit mapping loss and external-observation states for R2U2/C2PO,
  including unavailable and unsupported cases.
- Landed W/M canonical-lowering rows plus conditional rows for the
  not-yet-accepted past/history profile.
- A reproducible measurement that reports the declared population and every
  exclusion rather than inferring strength from fixture counts.

### Out of scope

- New temporal semantics, parsers, rewrite rules, or a production monitor.
- The native-predicate and native-temporal correspondence coverage dimension,
  which `quire-mltl` owns.
- Exhaustive enumeration of every formula graph, trace, or u32 interval value.
- Live execution of Java, Node, Electron, C2PO, R2U2, or another foreign runtime
  by build, test, CI, or qualification paths.
- Treating parser acceptance, differential agreement, local gates, or coverage
  ratios as source release, qualification, certification, or human approval.

## Requirements architecture

[FR-008](./requirements/FR-008-corpus-coverage-model.md) defines the closed
coverage-cell model. [FR-009](./requirements/FR-009-versioned-fixture-families.md)
defines fixture families, identities, ownership, and lifecycle.
[FR-010](./requirements/FR-010-explicit-interoperability-dispositions.md)
defines mapping and external-observation states.
[NFR-004](./requirements/NFR-004-reproducible-corpus-retention.md) constrains
reproducibility and retention. [MP-002](./assurance/MP-002-corpus-coverage.md)
defines the measurement, and [TM-002](./corpus-campaign-test-matrix.md) assigns
planned evidence.

## Admission and implementation gate

M0 is closed at `tl-mltl` v0.1.0 (`4bff387`). The W/M specification and routed
implementations have also landed: tl-syntax #37/#42 (`8d3ff98`/`8dc18ee`),
tl-parse #32 (`9ca856b`), tl-rewrite #36 (`033a687`), and tl-mltl #49/#50
(`36604b8`/`1770705`). W/M cells are therefore part of the current eligible
population and cannot remain `blocked` merely because this campaign predates
those merges.

Past/history rows remain `blocked` until their exact specification and
implementation revisions are independently accepted and landed. A blocked row
is visible but excluded from the applicable-coverage denominator and cannot own
a conformance fixture.

No campaign implementation begins before this MRS, its requirements,
measurement plan, matrix, PLAN-007 routing bundle, and the review of the exact
revision that carries them are accepted.
Past/history fixtures additionally require tl-syntax #38 and its routed
evaluator implementation.

After those gates, implementation order is:

1. `tl-mltl` implements the campaign/dimension/cell schema and fail-closed
   census for already landed future-v1 and W/M contracts.
2. Each authoritative owner publishes its family manifest and canonical cases;
   consumers add exact-digest replay only after the owner revision exists.
3. Rust adapter owners add loss reports; `tl-mltl` adds observation/comparison
   overlays without executing a target runtime.
4. The corpus census and replay feed MP-002 through the existing shared Quoin
   intake. Conditional families enter only through successor manifests after
   their own dependencies land.

The machine-readable owner, consumer, evidence, predecessor, and external
resume-condition allocation is PLAN-007 under
`plan/PLAN-007-mltl-corpus-campaign/`. GitHub implementation tickets mirror
that bundle and remain blocked on this specification ticket until human
acceptance is recorded.
