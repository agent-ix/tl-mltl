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

Rust and code review of the Kani-only horizon arithmetic harness found and
closed a traceability claim that no census reads, a vacuous arithmetic oracle,
and an absent Make gate. The harness calls the existing private checked-addition
primitive and compares arbitrary `u32` bounds and `u64` children to the standard
library checked-addition oracle, leaving normal builds and evaluator semantics
unchanged. Local Kani 0.67.0 verified the exact harness with `--unwind 4`.
README documents the exact manual command and proof boundary; `make kani-check`
is part of the locally invoked aggregate, while hosted CI remains manual-only.
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
| FND-2101 | low | `cargo:rustc-check-cfg` is emitted only when the invoking Cargo supports it (1.80+); the Rust-1.75 MSRV receives no unsupported directive, while modern ordinary `-D warnings` builds still validate the verifier-only `cfg(kani)`. The version comparison handles a future Cargo major version. The module adds no public API, production branch, or alternate evaluator. | build.rs, src/horizon.rs, AP-001 |
| FND-2102 | medium | Evaluator-wide Kani equivalence remains unproven: a reduced public-evaluator harness did not complete in the installed verifier configuration. Do not widen this primitive proof into an MLTL semantic proof claim. | SR-020, FR-003-AC-2, TC-032 |
| FND-2103 | high | A `// Trace:` comment above `#[kani::proof]` was neither read by Quire nor included in the Rust `#[test]` census, creating a non-binding traceability claim. Removed; SR-020 already records that the supplementary proof owns no matrix row. | src/horizon.rs, SR-020 FND-2004 |
| FND-2104 | medium | The former zero-bound/eager-overflow cases did not falsify a bound-scaling mutation. Fixed by making both operands symbolic and asserting equality with `u64::checked_add`, including the exact domain overflow error. `make kani-check` now runs this exact harness as part of the manually invoked local aggregate. | src/horizon.rs, Makefile, AP-001 |
| FND-2105 | low | The Kani module consistently imports both private arithmetic symbols through `super`, avoiding mixed qualification. | src/horizon.rs |
