---
id: SR-1516
title: "Risk and complexity analysis of P-TL-151"
type: SpecReview
analysis: risk-complexity
scope: "ltlf semantics, closure policy, external formats"
review_set: subset
---

## Summary

Highest risk is semantic: the finite/infinite/padded triple. Mission-bound closure policy (ADR-045: refuse by default, acknowledgement bound to report digest) is directionally sound but incompletely specified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15161 | high | ADR-045 acknowledgement is the SHA-256 of the FR-090 report, but FR-090 is defined only for already-bounded imported formulas (horizon H, class from bound equals M); an unbounded formula has no horizon (FR-083 returns unbounded-by-trace-length), so the report to be digested is undefined. The digest is also computable by the caller from the same library, so it binds identity but is not evidence that a human read it. Decide what the acknowledgement attests. | ADR-045, FR-090, FR-094-AC-3 |
| FND-15162 | high | FR-094 positive path (matching acknowledgement emits a bounded formula) needs the closure conversion, yet Task-007 says Peter decides whether it is built; AC-1/AC-3 are untestable until decided. | FR-094, Task-007 |
| FND-15163 | medium | Closure of `Xs` and `Xw` is unspecified: closed-trace pads a state (propositions false, src/future/evaluate.rs:517-528), not an absent position, so `Xw` has no bounded closed-trace equivalent and `Xs` maps to `F[1,1]` only approximately. FR-094 known-case table covers only F,G,U,R. | ADR-045, FR-094 |
| FND-15164 | medium | Exact-semantics risk in FR-308-AC-1: suffix after the marker has unspecified proposition values; independence from the suffix is the property to test, so test several suffix valuations. | tl-solve FR-308 |
| FND-15165 | medium | tl-parse lexing rule "maximal munch so Gp0 is G p0" is underspecified for `Ftrue`, `Gtrue`, `Xx`, `Uq`; hostile corpus cannot settle it until a lexical grammar is stated. | tl-parse FR-058 |
