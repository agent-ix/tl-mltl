---
id: MP-007
title: TL Stage 1 derived milestone completion view
type: MeasurementPlan
status: active
owner: tl-mltl-evidence-owner
metric: tl.v1.passed_milestones
definition_version: tl.v1.passed-milestones/v2
stage: observe
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
  - campaign/stage1-campaign-definition.json
  - src/bin/tl_campaign_check.rs
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

# TL Stage 1 derived milestone completion view

## Decision Use

The count is a display derived from the eleven Campaign groups. The
`all_required` CampaignDefinition over its direct members is the executable
gate. One missing or incomplete required member leaves its group and the Campaign open.
The derived count does not authorize a release or claim certification.

## Population

The eleven identities V1 through V11 and their direct members are fixed by
`campaign/stage1-campaign-definition.json`. V2's required completed depth-one partition is
distinct from its disclosed unvisited depth-three research population. V10's
required generated grid compares only admitted mappings to the pinned R2U2
target. The static past and unsafe-since results remain optional source-bound
diagnostics of unsupported target behavior. Their inconclusive verdicts remain
visible and cannot be counted as accepted mappings. The count never rounds an
unsupported, noisy, or incomplete required result into a pass.

## Collection Procedure

Derive each group status from the CampaignRun's complete member attempts and
independent domain checker receipts. Retain the source graph, exact native
process results, tool identities, raw artifacts, and eleven group statuses.
This count is a report projection over the CampaignRun, not a substitute
measurement collection or a second gate.

## Interpretation

The metric is the number of accepted groups among exactly eleven. Campaign
`all_required` is the decision rule. Milestone-specific uncertainty, refusals, survivor
reviews, uncovered branches, and performance noise remain visible in the raw
report. A later source or apparatus change is a new candidate and needs fresh
evidence; no historical pass is silently carried forward.
