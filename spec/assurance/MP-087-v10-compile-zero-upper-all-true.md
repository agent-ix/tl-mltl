---
id: MP-087
title: TL Stage 1 V10.compile.zero-upper-all-true direct foreign result
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v10.compile_zero_upper_all_true.verified
definition_version: tl.v10.compile-zero-upper-all-true/v1
execution_procedure: campaign/procedures/v10-compile-zero-upper-all-true.json
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 1
statistical_design:
  population: one exact direct native invocation for V10.compile.zero-upper-all-true in the source-bound Stage 1 campaign
  minimum_population: 1
  sampling: retained process streams and all declared output artifacts
  repetitions: 1
  estimator: count
  error_model: source drift, tool substitution, input drift, missing target positions, or semantic mismatch
  uncertainty: one pinned foreign invocation with independent TL domain checking
  decision_rule:
    comparator: ge
    threshold: 1
protected_apparatus:
  - campaign/procedures/v10-compile-zero-upper-all-true.json
  - src/bin/tl_campaign_check.rs
negative_controls:
  - kind: stale-evidence
    description: source and raw bytes must agree with the exact campaign graph
  - kind: suppressed-observation
    description: a missing native command or checker result cannot pass
relationships:
  - target: ix://agent-ix/tl-mltl/AP-002
    type: measures
  - target: ix://agent-ix/tl-mltl/FR-055
    type: references
---

# TL Stage 1 V10.compile.zero-upper-all-true direct foreign result

## Decision Use

This required V10 member is interpreted only from its retained native result and independently checked raw facts. The pinned C2PO 4.1.0 source compiles the sealed TL specification. The campaign combines every required member with `all-required`.

## Population

One exact direct producer invocation, bound to the campaign source graph and declared dependencies.

## Collection Procedure

EA executes the typed procedure in the fresh source projection and retains stdout, stderr, terminal status, and every declared artifact. Quoin binds the native command and independent TL checker result to this member.

## Interpretation

The TL checker recomputes decisive facts from retained bytes. Missing, malformed, stale, or semantically mismatched evidence cannot pass.
