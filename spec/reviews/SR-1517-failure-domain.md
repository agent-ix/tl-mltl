---
id: SR-1517
title: "Failure-domain analysis of P-TL-151"
type: SpecReview
analysis: failure-domain
scope: "padding, end-of-trace, exports"
review_set: subset
---

## Summary

LTLf checks of ADR-044 item 3 against my recollection of De Giacomo and Vardi: correct. Evidence that closed-trace padding is per-proposition (src/future/evaluate.rs:517-528: a position beyond the trace reads every proposition as false when closed; constants and negations still evaluate on that all-false state) matters for exports.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15171 | high | tl-solve ADR-301 item 2 / FR-304 encode closed-trace padding as "zero bits shifted in" of width n bit-vectors. The evaluator pads the proposition state, not the truth of subformulas: `F[1,1] (not p0)` at the last position is true under closed-trace (pad state has p0 false) but a shifted-in zero bit gives false; likewise `G[0,2] true`. SMT verdicts would disagree with the CNF path, caught only at the DEFERRED TC-322. Encoding must extend atoms (or width) by the interval horizon and evaluate operators over the pad. | tl-solve ADR-301, FR-304 |
| FND-15172 | medium | tl-solve FR-305: verdict agreement is defined per problem but trace length n is not fixed in the manifest; unsat at length n is not unsatisfiable, and the `phi and not phi` negatives are the only negatives. | FR-305 |
| FND-15173 | medium | FR-309 flags external unsat contradicting a bounded witness, but not the reverse (external sat with an exhausted bounded search) and nothing checks external unsat for longer-than-4 traces; reduction correctness beyond 340 traces per formula is untested. | FR-308, FR-309 |
| FND-15174 | low | Empty-trace refusal appears in FR-083 and tl-syntax ADR-044; tl-parse and tl-solve have no empty-trace statement for K=0 beyond FR-300-AC-2. | FR-083 |
