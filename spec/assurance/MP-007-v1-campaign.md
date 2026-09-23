---
id: MP-007
title: TL Stage 1 V1 campaign milestone completion
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v1.passed_milestones
definition_version: tl.v1.passed-milestones/v1
stage: gate
ground_truth_kind: mechanical
objective:
  direction: higher
  bound: 11
statistical_design:
  population: the fixed V1 through V11 milestone set for one exact five-repository source graph
  minimum_population: 11
  sampling: complete enumeration of the eleven registered milestones, with no omitted or substituted lane
  repetitions: 1
  estimator: count
  error_model: stale source or lock pin, missing native output, incomplete domain visits, unconfirmed noisy timing, or selective lane reporting
  uncertainty: the count is deterministic for the recorded report; native fuzz and performance limitations remain explicit in their milestone results
  decision_rule:
    comparator: ge
    threshold: 11
protected_apparatus:
  - campaign/make_manifest.py
  - campaign/v1_campaign.py
  - campaign/v4_fuzz.py
  - campaign/v5_gate.py
  - campaign/v5_mutation.py
  - campaign/v5_run.py
  - campaign/v6_kani.py
  - campaign/v7_gate.py
  - campaign/v7_native.py
  - campaign/v8_coverage.py
  - campaign/v8_gate.py
  - campaign/v9_criterion.py
  - campaign/v9_gate.py
  - tests/v1_finite_partition.rs
  - tests/v11_lasso_partition.rs
negative_controls:
  - kind: suppressed-observation
    description: the fixed V1 through V11 registry and each native population reconciliation expose a missing lane or skipped member
  - kind: stale-evidence
    description: source revisions, lockfiles, input hashes and raw output digests must match the selected graph
relationships:
  - target: ix://agent-ix/tl-mltl/AP-002
    type: measures
  - target: ix://agent-ix/tl-mltl/FR-054
    type: references
---

# TL Stage 1 V1 campaign milestone completion

## Decision Use

The count informs the release owner's bounded Stage 1 decision. Eleven passes
are the campaign gate; one missing or incomplete milestone leaves it open.
The measurement does not authorize a release or claim certification.

## Population

The eleven identities V1 through V11 and their registered lanes are fixed by
`campaign/v1_campaign.py`. V2's required completed depth-one partition is
distinct from its disclosed unvisited depth-three research population. V10
reports only admitted mappings to the pinned R2U2 target. The count never
rounds an unsupported, noisy, or incomplete result into a pass.

## Collection Procedure

Build one manifest after code and tool pins are frozen, execute every selected
lane, and retain the campaign report and raw artifacts with their SHA-256
digests. Record the exact manifest and report bytes, five source revisions,
tool identities, eleven statuses, and the passed count as one Quoin measurement
collection. Check the collection against this plan and the protected apparatus
before promotion.

## Interpretation

The metric is the number of `passed` milestones among exactly eleven. The
checker rule requires 11. Milestone-specific uncertainty, refusals, survivor
reviews, uncovered branches, and performance noise remain visible in the raw
report. A later source or apparatus change is a new candidate and needs fresh
evidence; no historical pass is silently carried forward.
