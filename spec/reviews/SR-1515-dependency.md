---
id: SR-1515
title: "Dependency analysis of P-TL-151"
type: SpecReview
analysis: dependency
scope: "ticket DAG TL-427..435"
review_set: subset
---

## Summary

`ix-board chain TL-431` shows the only blocker is TL-427; `ix-board project-chunk P-TL-151` puts TL-428, TL-431, TL-432, TL-433 all in layer 1 with no edge. FR-088 (TL-431) depends on FR-086 (TL-432) via Task-006 needing Task-003, so the confirmed edge TL-431 after TL-432 is missing from the tracker. Plan's assumption (add TL-432 as blocker) is the right one; the alternative moves the shared reader into TL-431 and contradicts TL-432 ownership.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15151 | high | Missing tracker edge TL-432 blocks TL-431 (FR-088 reuses FR-086 reader). Confirmed by ix-board chain and project-chunk; needs human or planner action, not a spec edit. | PLAN-050 Task-006, ix-board |
| FND-15152 | medium | Undeclared dependency: tl-solve FR-300-AC-1 and FR-302-AC-1 use the retained reference corpus, which is DEFERRED and blocks TC-231 (tl-mltl); TL-430 therefore depends on TL-428 corpus generation, not just TL-428 evaluator. | tl-solve FR-300, FR-084 |
| FND-15153 | medium | TL-430 sits in layer 4 behind TL-404 (lasso search) although the LTLf search needs only TL-428; the spec adopts the layers without questioning it. | PLAN-300 |
| FND-15154 | medium | TL-427 AC "ADRs accepted" is a human gate; FR-093-AC-2 is Inspection, and every build task is blocked on 12 proposed ADRs (tl-syntax 044/045, tl-parse 041, tl-mltl 044-047, tl-solve 300-302). | FR-093 |
