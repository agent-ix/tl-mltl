---
id: NFR-017
title: Keep numeric evaluation and evidence reproducible and float-free
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-mltl/NFR-001
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-054
    type: constrains
---

# NFR-017: Keep numeric evaluation and evidence reproducible and float-free

## Statement

The evaluator shall return byte-identical numeric evaluation and contract
reports for identical inputs on every supported platform, using no floating-point
type, with all work bounded by limits checked before evaluation.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Floating-point types in numeric modules | 0 | 0 | Inspection |
| Repeat-run report byte difference | 0 | 0 | Test |
| Retained numeric exchange digests verified at replay | 100% | 100% | Test |

## Verification

Static inspection of the numeric modules' types, byte comparison of repeated runs
and of the corpus replay report.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-017-AC-1 | Numeric and contract reports are byte-identical across repeated runs and no `f32` or `f64` appears in the numeric modules. | unit-testing (TC-1599) |
