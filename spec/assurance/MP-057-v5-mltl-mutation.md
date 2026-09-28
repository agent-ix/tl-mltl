---
id: MP-057
title: TL Stage 1 V5 tl-mltl mutation native mutation result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v5.mltl_mutation.verified
definition_version: tl.v5.mltl-mutation/v1
execution_procedure: campaign/procedures/v5-mltl-mutation.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native mutation invocation for tl-mltl in the Stage 1 campaign
  minimum_population: 1
  sampling: complete retained stdout stderr and bounded native output tree
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing mutant, partial native outcome, or failed restoration
  uncertainty: one selected mutant population under pinned cargo-mutants and source revision
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v5-mltl-mutation.json
  - src/bin/tl_campaign_check.rs
negative_controls:
  - kind: stale-evidence
    description: the campaign binds this result to exact source tree revisions and retained bytes
  - kind: suppressed-observation
    description: an absent or unverified direct invocation leaves the required member incomplete
relationships:
  - target: ix://agent-ix/tl-mltl/AP-002
    type: measures
  - target: ix://agent-ix/tl-mltl/FR-055
    type: references
---

# TL Stage 1 V5 tl-mltl mutation native mutation result

## Decision Use

This direct native result is one required member of the V5 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

The exact reviewed mutant selection for `tl-mltl` from the V5 source population. Discovery, mutation, and restored-green control each have their own EA attempt and independent TL checker receipt.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v5-mltl-mutation.json`. EA executes in a complete exact Git source projection. Quoin retains the request, raw process streams, and mutation output tree when applicable. The TL Rust checker independently scores selected native outcomes against retained discovery and source-bound reviews.

## Interpretation

An empty discovery, missing selected mutant, timeout, below-90-percent viable kill rate, unreviewed survivor, failed restored control, changed source, or missing checker receipt cannot pass V5. The campaign combines all twelve V5 members with `all-required`.
