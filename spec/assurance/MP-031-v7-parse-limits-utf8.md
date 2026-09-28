---
id: MP-031
title: TL Stage 1 V7.parse_limits_utf8 native embedded or Miri result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v7.parse_limits_utf8.passed
definition_version: tl.v7.parse-limits-utf8/v1
execution_procedure: campaign/procedures/v7-parse-limits-utf8.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native invocation for V7.parse_limits_utf8 in the five-source Stage 1 campaign
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
  - campaign/procedures/v7-parse-limits-utf8.json
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

# TL Stage 1 V7.parse_limits_utf8 native embedded or Miri result

## Decision Use

This direct native result is one required member of the V7 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared native command at the exact five-source graph. The metric is one only when the embedded target build or named Miri test succeeds under the declared toolchain and flags; zero otherwise.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v7-parse-limits-utf8.json`. EA executes in a complete exact Git source projection. Quoin retains the request and raw stdout and stderr. The TL Rust checker independently scores the native result.

## Interpretation

A missing invocation, changed source or raw bytes, failed native fact, or missing checker receipt cannot pass this member. The campaign combines members with `all-required`.
