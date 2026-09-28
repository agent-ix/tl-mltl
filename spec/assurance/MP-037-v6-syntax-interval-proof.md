---
id: MP-037
title: TL Stage 1 V6.syntax_interval_proof native Kani proof
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v6.syntax_interval_proof.proved
definition_version: tl.v6.syntax-interval-proof/v1
stage: gate
statistical_design:
  population: one exact direct native Kani proof for V6.syntax_interval_proof in the five-source Stage 1 campaign
  sampling: complete retained stdout stderr and execution result for the declared command
  repetitions: 1
  estimator: count
  error_model: stale source graph, incomplete checks, or proof status mismatch
  uncertainty: bounded unwind 2 proof under one pinned Kani and solver version
  decision_rule: The measured count must be at least 1.
relationships:
- target: ix://agent-ix/tl-mltl/AP-002
  type: measures
- target: ix://agent-ix/tl-mltl/FR-055
  type: references
---

# TL Stage 1 V6.syntax_interval_proof native Kani proof

## Decision Use

This direct native result is one required member of the V6 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared Kani harness in the exact five-source graph. The metric is one only when every property status succeeds and the exact harness, solver, check census, and verification summary agree; zero otherwise.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v6-syntax-interval-proof.json`. EA executes in a complete exact Git source projection. Quoin retains the request and raw stdout and stderr. The TL Rust checker independently scores Kani's property table and proof summary.

## Interpretation

A missing invocation, changed source or raw bytes, failed property, or missing checker receipt cannot pass this member. The separate seeded-false control and ordinary replay remain required V6 members before the group can pass.

## Measurement Controls

- Execution procedure: `campaign/procedures/v6-syntax-interval-proof.json`.
- Ground truth kind: `mechanical`.
- Minimum population: 1.
- Objective: higher, bound 1.
- Protected apparatus: `campaign/procedures/v6-syntax-interval-proof.json`, `src/bin/tl_campaign_check.rs`.
- Negative controls:
  - `stale-evidence`: the campaign binds this result to exact source tree revisions and retained bytes.
  - `suppressed-observation`: an absent or unverified direct invocation leaves the required member incomplete.
