---
id: NFR-007
title: Reproduce deterministic verification results
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-mltl/FR-054
    type: depends_on
---

# NFR-007: Reproduce deterministic verification results

## Statement

When identical source, tool, feature, corpus, seed and configuration pins are
used, the campaign shall reproduce the same deterministic oracle comparisons,
population counts and semantic report payload.

## Measurement and Evaluation

The target is 100% byte identity for deterministic payloads and 0 stale or
unattributed result substitutions. Timing distributions retain each sample
rather than requiring byte equality.

| Metric | Target | Threshold | Method |
|---|---|---|
| Identical deterministic payloads | Reproducibility | 100% | Repeat under exact pins and compare bytes |
| Stale substitutions | Freshness | 0 | Mutate one pin and require refusal |

## Verification

TC-175 and TC-196 run both controls against actual report and oracle output.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-007-AC-1 | Deterministic repeats match under exact pins, and changing one pin invalidates comparison credit. | Test (TC-175, TC-196) |
