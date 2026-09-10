---
id: MRS-002
title: "MLTL corpus coverage and interoperability campaign"
type: MasterRequirements
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

## Purpose

This specification defines a versioned, reviewable coverage model for the TL
ecosystem corpus. It separates canonical conformance fixtures, generated test
populations, retained external observations, and qualification claims while
making every applicable or excluded coverage cell explicit.

Native Quire remains the sole editable formal-clause language. TL formulas and
the tl-parse text dialect are internal representations; R2U2/C2PO and FRETish
remain mapping targets rather than source authorities or qualification
dependencies.

## Scope

### In scope

- Operator, interval-class, trace/history-shape, semantic-profile, outcome, and
  resource-boundary coverage cells.
- Strict fixture identities, manifests, digests, expected results, ownership,
  license/provenance, retention, and promotion rules.
- Explicit mapping loss and external-observation states for R2U2/C2PO and
  FRETish, including unavailable and unsupported cases.
- Conditional rows for reviewed-but-not-yet-accepted future-derived,
  past/history, native-predicate, and native-temporal bridge profiles.
- A reproducible measurement that reports the declared population and every
  exclusion rather than inferring strength from fixture counts.

### Out of scope

- New temporal semantics, parsers, rewrite rules, native Quire grammar, scalar
  predicate evaluation, or a production monitor.
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

The current v1 future-profile rows may be specified against landed contracts.
Rows for W/M, past/history, and native predicate/temporal correspondence remain
`blocked` until their exact specification revisions are independently accepted
and landed. A blocked row is visible but excluded from the applicable-coverage
denominator and cannot own a conformance fixture.

No implementation begins before M0 closes and this MRS, its requirements,
measurement plan, matrix, and review set are accepted. Predicate fixtures also
require quire-contract-ir #63; native temporal correspondence fixtures require
#63 and #64; W/M and past/history fixtures require tl-syntax #37 and #38.

After those gates, implementation order is:

1. `tl-mltl` implements the campaign/dimension/cell schema and fail-closed
   census for already landed future-v1 contracts.
2. Each authoritative owner publishes its family manifest and canonical cases;
   consumers add exact-digest replay only after the owner revision exists.
3. Rust adapter owners add loss reports; `tl-mltl` adds observation/comparison
   overlays without executing a target runtime.
4. The corpus census and replay feed MP-002 through the existing shared Quoin
   intake. Conditional families enter only through successor manifests after
   their own dependencies land.
