---
id: NFR-009
title: Bound verification cost without overstating evidence
type: NFR
quality_attribute: performance_efficiency
relationships:
  - target: ix://agent-ix/tl-mltl/FR-055
    type: depends_on
---

# NFR-009: Bound verification cost without overstating evidence

## Statement

When a local or nightly lane reaches its declared time, memory, work or input
limit, it shall stop with a named incomplete state and retain the visited
population and result without claiming the unvisited domain passed.

## Measurement and Evaluation

The target is 0 panics or wrapped limits on caller input and 0 exhausted lanes
reported complete. Benchmark ratios are reported only for comparable hosts.

| Metric | Target | Threshold | Method |
|---|---|---|
| Panic or wrapped limit | Robustness | 0 | At-limit and one-over inputs |
| Exhausted lane reported complete | Claim integrity | 0 | Force each budget stop |

## Verification

TC-188, TC-190 and TC-199 exercise limits and incomparable benchmark hosts.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-009-AC-1 | At-limit and one-over inputs, benchmark host mismatch and exhausted campaigns retain a non-passing status and measured partial population. | Test (TC-188, TC-190, TC-199) |
