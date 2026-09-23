---
id: MP-003
title: "TL property-domain grounding coverage"
type: MeasurementPlan
status: proposed
owner: tl-verification-campaign-owner
metric: tl.property-domain-grounding
definition_version: tl-mltl.property-domain-grounding/v2
stage: gate
objective:
  direction: higher
statistical_design:
  population: every criterion in the exact tl-mltl Quire property export, classified as applicable, excluded, or blocked
  sampling: complete criterion census; each applicable finite-exhaustive domain is fully enumerated and each generated domain uses three declared seeds and its predeclared accepted-case budget
  repetitions: 1
  estimator: proportion
  error_model: omitted or stale criteria, classifier overreach, vacuous preconditions, self-oracling, correlated derivation faults, generator bias, excessive discards, incomplete enumeration, and irreproducible shrinking
  uncertainty: finite exhaustive domains have no sampling interval; generated runs expose seed and class variation and make no probability-of-correctness estimate
  decision_rule:
    comparator: ge
    threshold: 1
relationships:
  - target: ix://agent-ix/tl-mltl/FR-020
    type: measures
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: measures
---

# TL property-domain grounding coverage

## Decision Use

The measure decides whether every declared applicable criterion has a reviewable
property grounding. It does not estimate semantic defect probability or approve
a source release.

## Population

The population is the full criterion list from the exact tl-mltl Quire export.
Applicable, excluded, and blocked rows, statement hashes, repository revisions,
and classifier observations are fixed before runs. Excluded and blocked rows
remain visible and outside the numerator and denominator for grounded coverage.
The collection is repeated once; within it, each generated row carries three
RNG-seeded executions while each finite-exhaustive row visits its domain once.

## Collection Procedure

Rust domain producers execute each finite enumeration or the three declared
generated seeds, retain exact domains, partitions, budgets, accepted/discarded
sequences, class histograms, oracle/control identities, failures and shrinking,
and emit structured records. Quoin retains those bytes; Quire supplies the
criterion census and runs no producer.

Sibling TL repositories require their own accepted local MeasurementPlan and
owner-native producer before collecting the corresponding measure. This plan
may be reused as a reviewed template but does not govern a sibling repository.

## Interpretation

The `proportion` estimate is completely grounded applicable criteria divided by
all applicable criteria. A row is completely grounded only when its grounding,
discriminating oracle control, finite budget, execution record, and limitation
are all complete. An applicable row that is not completely grounded is blocked
from advancing regardless of the aggregate, and
`statistical_design.decision_rule` holds only when no applicable row is blocked.
Per-domain accepted, discarded, class, failure, and shrink populations are
reported separately and never enter the estimate.

Report every population and per-seed observation before the grounding fraction.
A fully grounded row means the declared finite/generated domain and independent
oracle were exercised as specified. It says nothing about values outside that
domain or about criteria excluded or blocked by the campaign.
