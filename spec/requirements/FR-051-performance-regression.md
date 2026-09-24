---
id: FR-051
title: Measure bounded and infinite performance regressions
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-031
    type: depends_on
---

# FR-051: Measure bounded and infinite performance regressions

## Description

When Criterion benchmarks run, the owning crate shall compare representative
bounded, past and infinite workloads against a pinned baseline on the same
hardware class and report the measured distributions.

## Behavior

Cases include parser/formatter throughput, rewrite rule application,
closed/prefix evaluation, lasso/fairness analysis and C2PO rendering at
small, median and limit-near sizes. The run records input digest, hardware,
OS, toolchain, Criterion configuration, feature set, samples, median and
variance. A regression above 20% in median on a stable host requires a
repeated run and a named finding; noisy or unmatched hosts remain
non-conclusive. Resource ceilings remain functional gates independent of
benchmark ratios.

Comparability uses the executor-observed host context. A boot-scoped identity
requires the same boot and observed CPU affinity in both receipts; an absent or
malformed context leaves the comparison non-conclusive.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-051-AC-1 | Criterion records comparable baseline/current distributions and identifies confirmed above-threshold regressions by workload and revision. | Test (TC-190) |

## Dependencies

FR-031 supplies infinite workloads; each crate owns its benchmark inputs.
