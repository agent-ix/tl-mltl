---
id: FR-020
title: "Ground a complete property-obligation ledger"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-008
    type: depends_on
---

# FR-020: Run evaluator property laws against the independent oracle

## Description

When tl-mltl's evaluators change, the repository shall check their laws as tagged
generated-input tests with recorded seeds and accepted-case counts, comparing
against the independent oracle wherever a law needs a reference.

## Inputs

- The exact repository revision.
- The law set: round-trip, idempotence, duality, monotonicity, lowering-equivalence
  and verdict-preservation laws over the closed-future, prefix, past-history and
  infinite-trace evaluators.
- The independent oracle (`tl-oracle`) and its public canonical input and result
  types.

## Outputs

- Rust property tests tagged by criterion id, each running under a fixed seed and
  a stated accepted-case count.
- A run record per property: seed, requested and accepted cases, discarded cases,
  and the minimal shrunk counterexample of any failure, kept as a regression
  fixture.

## Behavior

A law that has an independent reference compares the production evaluation with
the oracle verdict. A law that does not (an algebraic identity such as a duality)
compares two production evaluations of different formulas, so a defect in a bound
or a duality case breaks the equality. A finite criterion that can be exhaustively
enumerated is enumerated, and the run records the cardinality visited.

The oracle shall not call the production operation under test or a helper that
implements the same derivation. It may share public canonical input and result
types.

Every property uses a fixed seed and an accepted-case count, and its strategies
construct admissible inputs so discards stay at or below a declared ceiling. A
run with zero accepted cases, a discard count above the ceiling, an unbounded
generator, or an unreproducible failing case is a failing test. Each oracle
comparison has a seeded-fault control: a deliberately broken evaluator turns the
comparison red.

Generated inputs are evidence, never canonical conformance fixtures. Promoting a
shrunk counterexample to the corpus follows FR-009.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-020-AC-1 | Every law in the law set is run by a tagged generated-input or finite-exhaustive test, and each criterion that is a property in this repository's specification is bound to such a test by its criterion id. | property-based-testing (TC-104, TC-105) |
| FR-020-AC-2 | A law that needs a reference compares the production evaluation with the independent oracle verdict; the oracle does not call the operation under test or a helper implementing the same derivation. | property-based-testing (TC-105, TC-106) |
| FR-020-AC-3 | Each seeded evaluator fault (interval end off by one, swapped dual operator, dropped negation) turns the oracle comparison red, and a property law is configured to permit no discarded cases. | integration-testing (TC-106) |
| FR-020-AC-4 | Repeating a run with the same revision, seed and case count reproduces its accepted-case sequence and minimal counterexample, and the seed and counts are recorded for the campaign report. | property-based-testing (TC-107) |
| FR-020-AC-5 | Generated cases stay outside canonical conformance counts, and a shrunk counterexample enters the corpus only through the FR-009 promotion conditions. | integration-testing (TC-108) |

## Dependencies

Depends on FR-008/FR-009 for class and canonical-fixture boundaries. Each repository
owns its generators, oracles and tests; tl-mltl owns its evaluator, horizon and
mapping laws.
