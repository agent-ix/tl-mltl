---
id: MRS-002
title: "MLTL corpus coverage and interoperability campaign"
type: MasterRequirements
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: depends_on
  - target: ix://agent-ix/tl-syntax/MRS-002
    type: depends_on
  - target: ix://agent-ix/tl-syntax/MRS-003
    type: depends_on
---

# MLTL corpus coverage and interoperability campaign

## Purpose

This specification defines how the TL ecosystem shows that its conformance
corpus covers the evaluator's operator and interval space, and how it records
what an external monitor did with the same cases. Coverage is a plain list of
cells, a gap check over that list, and agreement between every stored expected
verdict and an independent oracle. Generated test populations and retained
external observations stay separate from the canonical fixtures.

The canonical `tl-syntax` formula graph is this campaign's single source
authority. R2U2/C2PO is a mapping target and a source of retained observations,
never a source authority or a qualification dependency.

This campaign covers the TL-owned families only. Under the TL-175 architect
ruling every TL-* crate stays independent of the agent-ix/Quire ecosystem, so
the native-predicate and native-temporal correspondence dimension, whose
producer is `quire-contract-ir`, is not owned here. It is specified and routed
in `quire-mltl`. See
[ADR-002](./decisions/ADR-002-native-correspondence-lives-in-quire-mltl.md).

## Scope

### In scope

- A cell list of operator, interval-shape, profile, observation, and result
  classes, each with positive, negative, and boundary cases, and a gap check
  that fails when a cell has none.
- Stored expected verdicts derived by hand and confirmed by an independent
  oracle, with seeded faults showing the corpus can fail.
- Fixture families with one owner each, provenance, and license for
  retained external observations.
- Agree, disagree, and not-comparable outcomes for comparisons against retained
  R2U2/C2PO observations, with the reason, and typed refusals for mappings
  that cannot be emitted.
- A reproducible report that lists every population separately and states what
  it does not establish.

### Out of scope

- New temporal semantics, parsers, rewrite rules, or a production monitor.
- The native-predicate and native-temporal correspondence coverage dimension,
  which `quire-mltl` owns.
- Exhaustive enumeration of every formula graph, trace, or u32 interval value.
  Exhaustive small-scope sweeps against the oracle are evidence under MRS-003.
- Live execution of Java, Node, Electron, C2PO, R2U2, or another foreign runtime
  by a build, test, CI, or qualification path. Observations are retained once
  and replayed.
- Treating parser acceptance, differential agreement, local gates, or coverage
  ratios as source release, qualification, certification, or human approval.

## Requirements architecture

[FR-008](./requirements/FR-008-corpus-coverage-model.md) defines the cell list,
the gap check, and oracle agreement.
[FR-009](./requirements/FR-009-versioned-fixture-families.md) defines fixture
families, ownership, and how generated inputs relate to canonical fixtures.
[FR-010](./requirements/FR-010-explicit-interoperability-dispositions.md)
defines the agree, disagree, and not-comparable outcomes.
[NFR-004](./requirements/NFR-004-reproducible-corpus-retention.md) constrains
reproducibility and claim boundaries. [TM-002](./corpus-campaign-test-matrix.md)
assigns evidence.

## Profiles covered

The cell list covers the future profiles (`mltl.closed-trace/v1` and
`mltl.online-prefix/v1`) and the past-history profile
(`mltl.origin-complete-history/v1`). The derived W/M operators have no cells of
their own: they lower to primitive graphs that FR-016 verifies. A graph that
mixes future and past operators is refused by profile validation and has no
cell.
