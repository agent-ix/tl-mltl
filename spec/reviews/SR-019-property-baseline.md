---
id: SR-019
title: "Property and bounded-formal-methods review — MLTL baseline"
type: SpecReview
analysis: base
scope: "FR-003, NFR-001, TM-001, tests/property.rs"
review_set: base
---

# Property and bounded-formal-methods review — MLTL baseline

## Summary

TC-032 grounds the closing-prefix invariant over generated bounded Boolean and
unary temporal formulas, bounded intervals, and finite proposition traces. It
uses the production closed and prefix evaluators and claims only the declared
finite domain, not an unbounded proof.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- |
| FND-1901 | low | Loom is inapplicable: the semantic evaluator has no concurrent state, scheduler, atomics, or synchronization contract. | src/, FR-003 |
| FND-1902 | medium | Kani is the selected initial bounded-formal-methods path. A follow-up harness shall target evaluator recursion and horizon arithmetic with explicit bounds; this baseline does not claim that proof before the harness exists. | FR-001, FR-002, NFR-001 |
| FND-1903 | low | Verus remains an explicitly supported future architecture option. The property harness uses ordinary Rust/domain contracts and adds no Kani-specific production semantics, preserving a later Verus proof-layer choice. | tl-mltl#31 |
| FND-1904 | low | Fuzzing and mutation testing are outside this measured baseline and remain deferred by issue scope. | tl-mltl#31 |
