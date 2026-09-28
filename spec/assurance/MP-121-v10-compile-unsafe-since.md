---
id: MP-121
title: TL Stage 1 V10.compile.unsafe-since foreign diagnostic
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v10.compile_unsafe_since.verified
definition_version: tl.v10.compile-unsafe-since/v2
execution_procedure: campaign/procedures/v10-compile-unsafe-since.json
stage: observe
ground_truth_kind: mechanical
protected_apparatus:
- "campaign/procedures/v10-compile-unsafe-since.json"
- "src/bin/tl_campaign_check.rs"
negative_controls:
- kind: stale-evidence
  description: "source and raw bytes must agree with the exact campaign graph."
- kind: suppressed-observation
  description: "a missing native command or checker result cannot pass."
statistical_design:
  population: one exact direct native invocation for V10.compile.unsafe-since in the source-bound Stage 1 campaign
  sampling: retained process streams and all declared output artifacts
  repetitions: 1
  estimator: count
  error_model: source drift, tool substitution, input drift, missing target positions, or semantic mismatch
  uncertainty: one pinned foreign invocation with independent TL domain checking
  decision_rule:
    comparator: ge
    threshold: 1
relationships:
- target: ix://agent-ix/tl-mltl/AP-002
  type: measures
- target: ix://agent-ix/tl-mltl/FR-055
  type: references
---

# TL Stage 1 V10.compile.unsafe-since direct foreign result

## Decision Use

This optional V10 diagnostic is interpreted only from its retained native result and independently checked raw facts. The pinned C2PO 4.1.0 source compiles the sealed TL specification for the optional unsafe-since monitor. A failure or missing result remains visible but does not block the admitted-mapping parity gate.

## Population

One exact direct producer invocation, bound to the campaign source graph and declared dependencies.

## Collection Procedure

EA executes the typed procedure in the fresh source projection and retains stdout, stderr, terminal status, and every declared artifact. Quoin binds the native command and independent TL checker result to this member.

## Interpretation

The TL checker recomputes decisive facts from retained bytes. Missing, malformed, stale, or semantically mismatched evidence cannot pass.

## Measurement Controls

- Execution procedure: `campaign/procedures/v10-compile-unsafe-since.json`.
- Ground truth kind: `mechanical`.
- Minimum population: 1.
- Objective: higher, bound 1.
- Protected apparatus: `campaign/procedures/v10-compile-unsafe-since.json`, `src/bin/tl_campaign_check.rs`.
- Negative controls:
  - `stale-evidence`: source and raw bytes must agree with the exact campaign graph.
  - `suppressed-observation`: a missing native command or checker result cannot pass.
