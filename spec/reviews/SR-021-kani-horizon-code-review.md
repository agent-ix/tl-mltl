---
id: SR-021
title: "Code review — bounded Kani horizon arithmetic"
type: SpecReview
analysis: code-review
scope: "README.md, build.rs, src/horizon.rs, tests/shared_assurance.rs, SR-020; candidate 8341b33 plus working tree"
review_set: subset
---

# Code review — bounded Kani horizon arithmetic

## Summary

Rust and code review of the Kani-only horizon arithmetic harness found no
implementation defect. The harness calls the existing private checked-addition
primitive, covers arbitrary `u64` zero-addition and the positive-bound overflow
branch, and leaves normal builds and evaluator semantics unchanged. Local Kani
0.67.0 verified the exact harness with `--unwind 4` (0 of 127 checks failed).
README documents the exact manual command and proof boundary without making it
a Make or hosted-CI gate.
The TC-024 census update classifies all four paths introduced by the stacked
property/Kani work and preserves its area and total controls.

## Assurance Context

AP-001 applies because horizon resource claims can understate a downstream
monitor's required buffer. Evaluated candidate: `8341b33` plus this working
tree. The harness is supplementary, manually invoked evidence; it is neither a
release decision nor qualification/certification evidence. No exception is
claimed. The broader public-evaluator equivalence proof remains unavailable in
this installed Kani configuration and is explicitly retained as a structural
gap in SR-020 rather than hidden by this primitive-level proof.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2101 | low | No code defect found. The verifier-only module is gated by `cfg(kani)`, whose check-cfg declaration preserves ordinary `-D warnings` builds; it adds no public API, no production branch, no suppression, and no alternate evaluator. | build.rs, src/horizon.rs, AP-001 |
| FND-2102 | medium | Evaluator-wide Kani equivalence remains unproven: a reduced public-evaluator harness did not complete in the installed verifier configuration. Do not widen this primitive proof into an MLTL semantic proof claim. | SR-020, FR-003-AC-2, TC-032 |
