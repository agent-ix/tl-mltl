---
id: MP-011
title: TL Stage 1 V1 production finite faults native result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v1.v1_production_finite_faults.passed
definition_version: tl.v1.v1-production-finite-faults/v1
execution_procedure: campaign/procedures/v1-production-finite-faults.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native invocation for V1.production_finite_faults in the five-source Stage 1 campaign
  minimum_population: 1
  sampling: complete retained stdout stderr and execution result for the declared command
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing raw capture, wrong native marker, or command failure
  uncertainty: one bounded native result; later source revisions require new evidence
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v1-production-finite-faults.json
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

# TL Stage 1 V1 production finite faults native result

## Decision Use

This direct native result is one required member of the V1 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared native command at the exact five-source graph. The metric is one when the independent TL checker accepts the retained result and zero otherwise.

## Collection Procedure

The exact command and bound response are in `campaign/procedures/v1-production-finite-faults.json`. EA runs the command without a shell. Quoin retains its request, raw stdout and stderr, output artifacts, collection, and checker result. The TL Rust checker independently scores the decisive native facts.

## Interpretation

A missing invocation, changed source or raw bytes, failed native fact, or missing checker receipt cannot pass this member. The campaign combines member verdicts with `all-required`; this plan never substitutes a count for the other required members.
