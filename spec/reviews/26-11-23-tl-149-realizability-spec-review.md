---
id: SR-614
title: Independent specification review of P-TL-149 slice in tl-mltl
type: SpecReview
analysis: base
scope: spec/decisions/ADR-004-requirement-realizability-semantics.md, spec/requirements/FR-043-realizability-reference-playout.md, StR-004, spec/realizability-test-matrix.md
review_set: all
---

# Independent specification review (TL-149, tl-mltl)

## Summary

Slice of the P-TL-149 review; the full finding set, method statement (skills
performed manually, Jev not used) and population recomputation are in tl-solve
SR-100 (`spec/reviews/26-11-23-tl-149-realizability-spec-review.md` on branch
TL-149-review). `quire validate` shows no delta against main; coverage rows
rise by planned rows only.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1401 | high | PSPACE claim wrong: MLTL satisfiability is NEXPTIME-complete, PSPACE only for MLTL0 (Li, Vardi, Rozier, CAV 2019, per index records). | ADR-004 decision 8 line 179 |
| FND-1402 | high | Contract decision 5 does not match P-TL-150 ADR-017 or FR-059 (per-position series, pending outcome, options A/B/C unnamed); FR-311-AC-2 forbids citing contracts as assumptions before a human decides. | ADR-004 decision 5 |
| FND-1403 | medium | Whole-trace A -> G makes Realizable weaker than G while A held; limit (a) and (b) acknowledged but label unchanged. | ADR-004 decision 5 |
| FND-1404 | medium | FR-043 has no interface to judge a mutated strategy (needed by tl-solve FR-106-AC-3) and no build ticket; population is declared only in tl-solve while tl-mltl FR-113 forbids that edge. | FR-043; TM-007 |
| FND-1405 | low | evaluate_closed_at returns EvaluationReport; lookahead is an upper bound, not the exact last read position. | ADR-004 decision 2; FR-043 |
