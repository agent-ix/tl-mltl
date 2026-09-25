---
id: MP-040
title: TL Stage 1 V6.ordinary_counterexample_replay native Kani control
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v6.ordinary_counterexample_replay.verified
definition_version: tl.v6.ordinary-counterexample-replay/v1
execution_procedure: campaign/procedures/v6-ordinary-counterexample-replay.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native invocation for V6.ordinary_counterexample_replay in the five-source Stage 1 campaign
  minimum_population: 1
  sampling: complete retained stdout stderr and execution result for the declared command
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing Kani counterexample, or failed ordinary replay
  uncertainty: one bounded control at the pinned solver and source graph
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v6-ordinary-counterexample-replay.json
  - campaign/kani_negative/Cargo.toml
  - campaign/kani_negative/src/lib.rs
  - campaign/kani_negative/tests/replay.rs
  - src/bin/tl_campaign_check.rs
negative_controls:
  - kind: apparatus-edit
    description: the verifier-only claim must fail with a concrete counterexample that ordinary Rust replays
  - kind: stale-evidence
    description: the campaign binds the control to exact source tree revisions and retained bytes
relationships:
  - target: ix://agent-ix/tl-mltl/AP-002
    type: measures
  - target: ix://agent-ix/tl-mltl/FR-055
    type: references
---

# TL Stage 1 V6.ordinary_counterexample_replay native Kani control

## Decision Use

This direct native result is one required member of the V6 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared control at the exact five-source graph. The seeded-false claim deliberately fails; its metric is one only when Kani emits the expected concrete counterexample. The ordinary replay metric is one only when the pinned Rust test succeeds on those same bytes.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v6-ordinary-counterexample-replay.json`. EA executes in a complete exact Git source projection. Quoin retains the request and raw stdout and stderr. The TL Rust checker independently scores the Kani output and replay result.

## Interpretation

A missing invocation, changed source or raw bytes, Kani pass, mismatched counterexample, failed replay, or missing checker receipt cannot pass this member. The replay depends on the seeded-false member; the V6 group uses `all-required`.
