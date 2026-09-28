---
id: MP-022
title: TL Stage 1 V4.parse_unbounded_roundtrip native libFuzzer result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v4.parse_unbounded_roundtrip.bounded_no_crash
definition_version: tl.v4.parse-unbounded-roundtrip/v1
execution_procedure: campaign/procedures/v4-parse-unbounded-roundtrip.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native libFuzzer invocation for V4.parse_unbounded_roundtrip in the five-source Stage 1 campaign
  minimum_population: 1
  sampling: complete retained stdout stderr and optional fixed crash artifact
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing raw capture, wrong native budget marker, or crash artifact
  uncertainty: one bounded native result; later source revisions require new evidence
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v4-parse-unbounded-roundtrip.json
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

# TL Stage 1 V4.parse_unbounded_roundtrip native libFuzzer result

## Decision Use

This direct native result is one required member of the V4 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared libFuzzer target at the exact five-source graph. The metric is one only when 1,000 executions with seed 181, address sanitizer, a clean exit, and no crash artifact are independently verified; zero otherwise.

## Collection Procedure

The direct command and response are in `campaign/procedures/v4-parse-unbounded-roundtrip.json`. EA runs the command without a shell in a complete exact Git source projection. Quoin retains the request, raw stdout and stderr, and optional `crash.bin` output artifact. The TL Rust checker independently scores the engine budget and crash evidence.

## Interpretation

A missing invocation, changed source or raw bytes, incomplete budget, failed native process, or crash artifact cannot pass this member. The campaign combines member verdicts with `all-required`.
