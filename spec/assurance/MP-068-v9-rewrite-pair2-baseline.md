---
id: MP-068
title: TL Stage 1 V9 rewrite pair 2 baseline Criterion result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v9.rewrite_pair2_baseline.verified
definition_version: tl.v9.rewrite-pair2-baseline/v1
execution_procedure: campaign/procedures/v9-rewrite-pair2-baseline.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct Criterion invocation for V9.rewrite_pair2_baseline in the source-bound Stage 1 campaign
  minimum_population: 1
  sampling: retained process streams, Criterion sample and estimate output tree, and observed host fingerprint
  repetitions: 1
  estimator: count
  error_model: source drift, benchmark harness mismatch, missing samples, or host noise
  uncertainty: two same-host baseline/candidate pairs of 20 Criterion samples per case with 2000 bootstrap draws
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v9-rewrite-pair2-baseline.json
  - src/bin/tl_campaign_check.rs
negative_controls:
  - kind: gain-within-noise
    description: a threshold overlap is inconclusive and cannot earn a pass
  - kind: stale-evidence
    description: the campaign binds each result to exact source revisions and retained bytes
relationships:
  - target: ix://agent-ix/tl-mltl/AP-002
    type: measures
  - target: ix://agent-ix/tl-mltl/FR-055
    type: references
---

# TL Stage 1 V9 rewrite pair 2 baseline Criterion result

## Decision Use

This direct native result is one required member of the V9 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of the declared Criterion benchmark at an exact Git source projection. Baseline and candidate use byte-identical benchmark harnesses. Two separately named pairs are required for every crate.

## Collection Procedure

The direct command and bounded response are in `campaign/procedures/v9-rewrite-pair2-baseline.json`. EA retains the Criterion output tree under `.quoin-target/criterion` with each sample and estimate file's digest. Quoin retains the request, raw process streams, observed host fingerprint, and dependency identities. The TL Rust checker independently scores the samples and the source-bound pair.

## Interpretation

A missing invocation, changed source or raw bytes, unequal benchmark harness, host drift, missing sample, above-20-percent repeated regression, or noise overlap cannot pass V9. The campaign combines all twelve V9 members with `all-required`.
