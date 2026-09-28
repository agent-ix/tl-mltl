---
id: MP-044
title: TL Stage 1 V8.parse_example_prep native coverage result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v8.parse_example_prep.verified
definition_version: tl.v8.parse-example-prep/v1
stage: gate
statistical_design:
  population: one exact direct native invocation for V8.parse_example_prep in the five-source Stage 1 campaign
  sampling: complete retained stdout stderr and required output artifact
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing output, unmeasured critical file, or uncovered branch side
  uncertainty: one pinned native coverage export; uncovered sides require named reviews
  decision_rule: The measured count must be at least 1.
relationships:
- target: ix://agent-ix/tl-mltl/AP-002
  type: measures
- target: ix://agent-ix/tl-mltl/FR-055
  type: references
---

# TL Stage 1 V8.parse_example_prep native coverage result

## Decision Use

This direct native result is one required member of the V8 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared Cargo command at the exact five-source graph. The metric is one only when the parse example binary is built as a required artifact and consumed by the dependent parse coverage invocation; zero otherwise.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v8-parse-example-prep.json`. EA executes in a complete exact Git source projection with CARGO_TARGET_DIR `.quoin-target`. Quoin retains the request, raw stdout and stderr, and the required output artifact. The TL Rust checker independently scores the native result and export.

## Interpretation

A missing invocation, changed source or raw bytes, missing output, failed native fact, or missing checker receipt cannot pass this member. Parse coverage depends on its exact prep artifact. The campaign combines members with `all-required`.

## Measurement Controls

- Execution procedure: `campaign/procedures/v8-parse-example-prep.json`.
- Ground truth kind: `mechanical`.
- Minimum population: 1.
- Objective: higher, bound 1.
- Protected apparatus: `campaign/procedures/v8-parse-example-prep.json`, `src/bin/tl_campaign_check.rs`.
- Negative controls:
  - `stale-evidence`: the campaign binds this result to exact source tree revisions and retained bytes.
  - `suppressed-observation`: an absent or unverified direct invocation leaves the required member incomplete.
