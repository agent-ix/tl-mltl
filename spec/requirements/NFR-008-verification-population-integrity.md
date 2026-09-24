---
id: NFR-008
title: Reconcile every claimed verification population
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-mltl/FR-044
    type: depends_on
---

# NFR-008: Reconcile every claimed verification population

## Statement

When a verification lane claims completeness, it shall account for every
member of its declared finite population as passed, failed, refused,
excluded, blocked or not-run and report no success with an unvisited member.

## Measurement and Evaluation

The target is exact cardinality reconciliation and 0 hidden exclusions across
exhaustive, property, mutation, proof and coverage populations.

| Metric | Target | Threshold | Method |
|---|---|---|
| Count reconciliation | Population integrity | Exact | Compare independently computed and visited counts |
| Hidden exclusions | Completeness | 0 | Seed omission, duplicate and unvisited cells |

## Verification

TC-177, TC-180, TC-183, TC-185 and TC-189 exercise the owning populations.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-008-AC-1 | Omitted, duplicated, silently excluded or unvisited members make the owning lane incomplete. | Test (TC-177, TC-180, TC-183, TC-185, TC-189) |
