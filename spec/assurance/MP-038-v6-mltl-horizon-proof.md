---
id: MP-038
title: TL Stage 1 V6.mltl_horizon_proof native Kani proof
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v6.mltl_horizon_proof.proved
definition_version: tl.v6.mltl-horizon-proof/v1
execution_procedure: campaign/procedures/v6-mltl-horizon-proof.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native Kani proof for V6.mltl_horizon_proof in the five-source Stage 1 campaign
  minimum_population: 1
  sampling: complete retained stdout stderr and execution result for the declared command
  repetitions: 1
  estimator: count
  error_model: stale source graph, incomplete checks, or proof status mismatch
  uncertainty: bounded unwind 2 proof under one pinned Kani and solver version
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v6-mltl-horizon-proof.json
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

# TL Stage 1 V6.mltl_horizon_proof native Kani proof

## Decision Use

This direct native result is one required member of the V6 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared Kani harness in the exact five-source graph. The metric is one only when every property status succeeds and the exact harness, solver, check census, and verification summary agree; zero otherwise.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v6-mltl-horizon-proof.json`. EA executes in a complete exact Git source projection. Quoin retains the request and raw stdout and stderr. The TL Rust checker independently scores Kani's property table and proof summary.

## Interpretation

A missing invocation, changed source or raw bytes, failed property, or missing checker receipt cannot pass this member. The separate seeded-false control and ordinary replay remain required V6 members before the group can pass.
