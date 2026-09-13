---
id: MP-002
title: "MLTL corpus coverage census"
type: MeasurementPlan
status: proposed
owner: tl-mltl-corpus-owner
metric: tl-mltl.corpus-coverage
definition_version: tl-mltl.corpus-coverage/v1
stage: gate
statistical_design:
  population: every cell in the closed tl-mltl.corpus-campaign/v1 catalog, stratified as applicable, excluded, or blocked
  sampling: complete deterministic enumeration of declared cells; generated property and fuzz inputs are reported separately
  repetitions: 1
  estimator: covered applicable cells divided by all applicable cells, with raw identities and excluded/blocked populations reported separately
  error_model: omitted or duplicate cells, denominator shrinkage, stale profile or oracle identity, digest drift, self-oracling, generated-fixture substitution, and collapsed target states
  uncertainty: no sampling interval for the deterministic census; unenumerated value-space and unavailable external targets remain explicit limitations
  decision_rule: fail the corpus gate when any applicable cell lacks exactly one canonical independently-oracled fixture or any excluded/blocked cell lacks its required reason and dependency
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: measures
  - target: ix://agent-ix/tl-mltl/NFR-004
    type: measures
---

# MLTL corpus coverage census

## Decision Use

The measurement decides only whether the declared campaign population is
internally complete enough for implementation/release review. It does not rank
semantic correctness, approve a source release, qualify a monitor, or validate
the completeness of the unbounded formula/trace value space.

## Population

The population is the exact ordered set of FR-008 coverage-cell identities at
one campaign-manifest digest. Applicable, excluded, and blocked strata are
immutable inputs to the calculation. A blocked profile becomes applicable only
through a reviewed successor manifest naming the landed dependency revisions.

## Collection Procedure

The Rust corpus validator reads the versioned dimension catalog and fixture
manifests, enumerates each cell once, verifies owner/digest/provenance and stored
oracle identity, then replays applicable cases through their named consumer.
It emits the raw cell list and status before totals. Quoin retains the producer
record; Quire supplies obligation/coverage facts without running the producer.

Mutation controls delete, duplicate, reclassify, restamp, or substitute one
cell/fixture/state at a time and require the owning gate to identify it. The
collection records exact repository, dependency, corpus, tool, configuration,
and environment identities.

## Interpretation

Report raw counts and identities for every stratum before any ratio. A 100%
applicable-cell result means only that the declared finite class model is
populated and passed its named checks. Excluded, blocked, unavailable,
unsupported, non-conclusive, and generated populations remain adjacent and
cannot improve that ratio by disappearing. Human review evaluates whether the
dimension catalog itself is adequate.
