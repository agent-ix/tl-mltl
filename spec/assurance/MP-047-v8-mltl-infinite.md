---
id: MP-047
title: TL Stage 1 V8.mltl_infinite native coverage result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v8.mltl_infinite.verified
definition_version: tl.v8.mltl-infinite/v1
execution_procedure: campaign/procedures/v8-mltl-infinite.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native invocation for V8.mltl_infinite in the five-source Stage 1 campaign
  minimum_population: 1
  sampling: complete retained stdout stderr and required output artifact
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing output, unmeasured critical file, or uncovered branch side
  uncertainty: one pinned native coverage export; uncovered sides require named reviews
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v8-mltl-infinite.json
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

# TL Stage 1 V8.mltl_infinite native coverage result

## Decision Use

This direct native result is one required member of the V8 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared Cargo command at the exact five-source graph. The metric is one only when the raw llvm-cov export is a required artifact, and the checker scores each critical file and branch side; zero otherwise.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v8-mltl-infinite.json`. EA executes in a complete exact Git source projection with CARGO_TARGET_DIR `.quoin-target`. Quoin retains the request, raw stdout and stderr, and the required output artifact. The TL Rust checker independently scores the native result and export.

## Interpretation

A missing invocation, changed source or raw bytes, missing output, failed native fact, or missing checker receipt cannot pass this member. Parse coverage depends on its exact prep artifact. The campaign combines members with `all-required`.
