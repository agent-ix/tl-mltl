---
id: FR-011
title: "Ground a complete property-obligation ledger"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-006
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-008
    type: depends_on
---

# FR-011: Ground a complete property-obligation ledger

## Description

When an exact TL repository revision enters the effectiveness campaign, the
campaign shall classify every criterion in its Quire property export and bind
every applicable criterion to a finite generated or exhaustive domain, an
independent oracle, a reproducible execution budget, and one owning Rust test.

## Inputs

- Exact repository revision and Quire/Quoin/toolchain identities.
- The complete `quire properties --json` criterion population, including row
  id, statement hash, property shape, source path, and authored method.
- Reviewed priority from every owning TestMatrix row that traces to the
  criterion, plus the exact matrix digest and applicable semantic, operator,
  interval, trace/history, clock, capture, resource, and refusal profiles.

## Outputs

- An owner-native `<repository>.property-ledger/v1` record with every criterion
  classified exactly once as `applicable`, `excluded`, or `blocked`; tl-mltl's
  identity is `tl-mltl.property-ledger/v1`.
- For each applicable criterion, an owner-native property-run record and exact
  owning Rust test symbol.
- A typed refusal for duplicate, omitted, stale, self-oracled, unbounded,
  vacuous, or identity-incomplete entries.

## Behavior

The ledger population is the complete Quire export, not only criteria that the
property-shape classifier labels specific. `property_shaped`, `specific_shaped`,
`example`, `universal`, and `unclassified` are observations, not verdicts.

An `applicable` row declares the generated or finite-exhaustive input domain,
precondition, valid/invalid partitions, generator and shrinker identities,
independent oracle or metamorphic relation, operation under test, accepted-case
budget, discard ceiling, expected result/refusal classes, and limitation. An
`excluded` row names a closed reason such as inspection-only, analysis-only,
example-specific, or non-generative. A finite criterion that can be exhaustively
enumerated remains applicable and binds its enumerator evidence. A
`blocked` row names the unresolved requirement/profile/implementation revision
and produces no coverage credit.

The stable row identity is the domain-separated SHA-256 of repository, exact
source revision, criterion id and statement hash, operation, semantic-profile
identity, and domain-partition identity. Classification and execution result are
excluded from that identity, so reclassifying a blocked or failing row cannot
hide the same concern as a new row.

The ledger digest covers the ordered row identities, classification, reason or
dependency, domain, profile, oracle, generator, owning matrix priorities, and
ownership fields. The ledger is append-only within one identity. Changing any
covered value requires a reviewed successor ledger with predecessor digest and
an explicit added/changed/removed census; a removed row remains a tombstone.

Campaign priority is the highest urgency among the exact TestMatrix rows that
trace to a criterion; ties retain every row identity. A criterion with no
reviewed matrix priority is blocked. Neither Quire's currently nullable
`obligation.criticality` field nor a producer default may invent a priority.

The independent oracle may share public canonical input and result types. The
independent oracle shall not call the production operation under test, its evaluator/horizon/
history/parser/formatter/rewrite/mapping helpers, or a helper implementing the
same derivation. A metamorphic relation identifies both executions and explains
why the relation is independent of the suspected fault. A mutation control
must demonstrate that the oracle or relation detects at least one relevant
seeded fault; a surviving control makes the row `suspect`, not passed.

Each generated repetition records the exact seed, requested and accepted cases,
discarded cases by reason, class histogram, first failure, minimal shrunk case,
shrink termination, tool/configuration/environment identities, and start/end
corpus digests when a corpus participates. Zero accepted cases, a discard count
above the declared ceiling, an unbounded generator, or an unreproducible failing
case is non-conclusive and cannot satisfy the row. Exhaustive runs record the
declared finite cardinality and prove every member was visited exactly once.

Property inputs are generated evidence, never canonical conformance fixtures.
Promotion follows FR-009 and requires a minimal reproducer, independent expected
result, complete provenance, review, and a successor manifest.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-011-AC-1 | Every criterion in each exact Quire export appears exactly once as applicable, excluded, or blocked under a lifecycle-independent row identity and classification-covering ledger digest; omissions, duplicates, stale statement hashes, unknown classifications, missing matrix priority, and unreviewed successor changes refuse while naming the row. | Test (TC-050, TC-051) |
| FR-011-AC-2 | Every applicable row binds a finite domain, partitions, precondition, generator/exhaustive enumerator, shrink behavior, independent oracle or justified metamorphic relation, operation, Rust symbol, budgets, expected classes, and limitation. | Test (TC-051, TC-052) |
| FR-011-AC-3 | Self-oracles, shared derivation helpers, vacuous preconditions, zero accepted cases, excessive discards, incomplete exhaustive visits, and seeded faults that the oracle misses remain suspect or non-conclusive and earn no property-coverage credit. | Test (TC-052, TC-053) |
| FR-011-AC-4 | Repeating a generated run with the same revision, domain, configuration, seed, and environment reproduces its accepted/discarded class sequence and minimal counterexample, while different seeds remain separately identified. | Test (TC-053) |
| FR-011-AC-5 | Generated cases remain outside canonical conformance counts, and promotion is refused unless every FR-009 identity, oracle, provenance, review, and successor-manifest condition is present. | Test (TC-054) |

## Dependencies

Depends on FR-006 for shared static facts/intake and FR-008/FR-009 for class
and canonical-fixture boundaries. Each repository owns its domain generators,
oracles, and tests; tl-mltl owns only its own evaluator/horizon/mapping rows and
the campaign-level contract.
