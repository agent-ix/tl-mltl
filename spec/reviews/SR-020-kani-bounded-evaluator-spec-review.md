---
id: SR-020
title: "Bounded Kani evaluator feasibility review"
type: SpecReview
analysis: base
scope: "FR-001, FR-002, FR-003, NFR-001, TM-001, TC-006, TC-032, src/evaluate.rs, src/horizon.rs"
review_set: base
---

# Bounded Kani evaluator feasibility review

## Summary

This review assesses Kani against the existing bounded-property baseline. It
keeps verifier code out of production semantics and makes no claim of an
unbounded MLTL proof.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2001 | medium | The existing generated TC-032 evidence is bounded but stochastic. Kani cannot presently complete a public-evaluator equivalence harness in this repository's installed toolchain, even after reducing it to one atomic observation; do not represent an incomplete run as proof. Keep TC-032's generated evidence and record evaluator-proof feasibility as follow-up work. | FR-003-AC-2, NFR-001-AC-1, TC-032 |
| FND-2002 | low | A focused Kani proof can establish the checked-arithmetic primitive used by horizon traversal: every `u32` temporal bound added to zero returns its exact `u64` value. Its bound stays in the test-only verifier module; it is not evidence for arbitrary traversal, nested temporal bounds, or traces. | FR-002-AC-2, TC-006 |
| FND-2003 | low | Loom remains inapplicable because this evaluator has no concurrent state or scheduling contract. Verus remains a future architecture choice; ordinary Rust interfaces and the property baseline must remain usable without either verifier. | SR-019, tl-mltl#31 |
| FND-2004 | low | The test matrix has no Formal method token. Retain TC-032 as Property and record the Kani command, version, exact harness, and bounded claim in review material rather than inventing a local vocabulary or changing shared Quire configuration. | TM-001, TC-032, AP-001 |
