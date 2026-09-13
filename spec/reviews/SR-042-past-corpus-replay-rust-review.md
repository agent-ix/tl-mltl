---
id: SR-042
title: "Rust review — shared past/history evaluator replay"
type: SpecReview
analysis: code-review
scope: "tests/past_history_corpus.rs"
review_set: subset
---

# Rust review — shared past/history evaluator replay

## Summary

Applied the Rust review checklist to native corpus construction and every
analysis/evaluation/refusal seam used by the replay.

## Verdict

**PASS.** Tests use public typed constructors and errors at history, clock,
analysis, evaluation, correction, and serialization seams. Exact-number
construction is checked, proposition IDs are explicit, and no cast truncates a
wire value. No production unsafe, blocking, lock, or panic surface changed.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4201 | medium | Native sample units are derived only from the exact owning clock binding before history validation. | `history` test helper |
| FND-4202 | medium | Resource rows now exercise reachable `TemporalSpanExceeded` and `StepLimitExceeded` paths. | `every_shared_refusal_case_exercises_its_native_boundary` |

Strict Clippy and the complete non-qualification evaluator suite pass.
