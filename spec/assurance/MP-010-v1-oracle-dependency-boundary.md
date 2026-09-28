---
id: MP-010
title: TL Stage 1 V1 oracle dependency boundary native result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v1.v1_oracle_dependency_boundary.passed
definition_version: tl.v1.v1-oracle-dependency-boundary/v3
stage: gate
ground_truth_kind: mechanical
protected_apparatus:
- "campaign/procedures/v1-oracle-dependency-boundary.json"
- "src/bin/tl_campaign_check.rs"
negative_controls:
- kind: stale-evidence
  description: "the campaign binds this result to exact source tree revisions and retained bytes."
- kind: suppressed-observation
  description: "an absent or unverified direct invocation leaves the required member incomplete."
statistical_design:
  population: one exact direct Cargo normal-dependency tree invocation for V1.oracle_dependency_boundary in the five-source Stage 1 campaign
  sampling: complete retained stdout stderr and execution result for the declared command
  repetitions: 1
  estimator: count
  error_model: stale source graph, missing raw capture, missing tl-syntax, forbidden production dependency, or command failure
  uncertainty: one bounded native result; later source revisions require new evidence
  decision_rule:
    comparator: ge
    threshold: 1
relationships:
- target: ix://agent-ix/tl-mltl/AP-002
  type: measures
- target: ix://agent-ix/tl-mltl/FR-055
  type: references
---

# TL Stage 1 V1 oracle dependency boundary native result

## Decision Use

This direct native result is one required member of the V1 Campaign group. It informs the bounded Stage 1 evidence decision and does not authorize a release.

## Population

One direct EA invocation of `cargo tree --locked --offline --edges normal --prefix none` in the pinned `tl-oracle` checkout. The metric is one only when the independent TL checker finds `tl-oracle` and `tl-syntax` in the retained normal dependency graph, finds no `tl-mltl` or `tl-rewrite`, and the native command exits cleanly; zero otherwise.

## Collection Procedure

The exact command and bound response are in `campaign/procedures/v1-oracle-dependency-boundary.json`. EA executes Cargo directly without a shell or nested Cargo child. Quoin retains its request, raw stdout and stderr, collection, and checker result. The TL Rust checker independently parses the normal dependency tree. This avoids Cargo's `CARGO` environment variable naming EA's sealed, deleted executable during a nested test.

## Interpretation

A missing invocation, changed source or raw bytes, failed native fact, or missing checker receipt cannot pass this member. The campaign combines member verdicts with `all-required`; this plan never substitutes a count for the other required members.

## Measurement Controls

- Execution procedure: `campaign/procedures/v1-oracle-dependency-boundary.json`.
- Ground truth kind: `mechanical`.
- Minimum population: 1.
- Objective: higher, bound 1.
- Protected apparatus: `campaign/procedures/v1-oracle-dependency-boundary.json`, `src/bin/tl_campaign_check.rs`.
- Negative controls:
  - `stale-evidence`: the campaign binds this result to exact source tree revisions and retained bytes.
  - `suppressed-observation`: an absent or unverified direct invocation leaves the required member incomplete.
