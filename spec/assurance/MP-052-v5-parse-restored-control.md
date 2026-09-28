---
id: MP-052
title: TL Stage 1 V5 tl-parse restored-control native mutation result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v5.parse_restored_control.verified
definition_version: tl.v5.parse-restored-control/v1
stage: gate
statistical_design:
  population: one exact direct native restored-control invocation for tl-parse in the Stage 1 campaign
  sampling: complete retained stdout stderr and execution result
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing mutant, partial native outcome, or failed restoration
  uncertainty: one selected mutant population under pinned cargo-mutants and source revision
  decision_rule: The measured count must be at least 1.
relationships:
- target: ix://agent-ix/tl-mltl/AP-002
  type: measures
- target: ix://agent-ix/tl-mltl/FR-055
  type: references
---

# TL Stage 1 V5 tl-parse restored-control native mutation result

## Decision Use

This direct native result is one required member of the V5 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

The exact reviewed mutant selection for `tl-parse` from the V5 source population. Discovery, mutation, and restored-green control each have their own EA attempt and independent TL checker receipt.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v5-parse-restored-control.json`. EA executes in a complete exact Git source projection. Quoin retains the request, raw process streams, and mutation output tree when applicable. The TL Rust checker independently scores selected native outcomes against retained discovery and source-bound reviews.

## Interpretation

An empty discovery, missing selected mutant, timeout, below-90-percent viable kill rate, unreviewed survivor, failed restored control, changed source, or missing checker receipt cannot pass V5. The campaign combines all twelve V5 members with `all-required`.

## Measurement Controls

- Execution procedure: `campaign/procedures/v5-parse-restored-control.json`.
- Ground truth kind: `mechanical`.
- Minimum population: 1.
- Objective: higher, bound 1.
- Protected apparatus: `campaign/procedures/v5-parse-restored-control.json`, `src/bin/tl_campaign_check.rs`.
- Negative controls:
  - `stale-evidence`: the campaign binds this result to exact source tree revisions and retained bytes.
  - `suppressed-observation`: an absent or unverified direct invocation leaves the required member incomplete.
