---
id: SR-049
title: "Dependency review of the verification-effectiveness campaign"
type: SpecReview
analysis: dependency
scope: "MRS-003, FR-011 through FR-015, NFR-005 and routed ecosystem work"
review_set: all
---

## Summary

**PASS after remediation.** Specification can be reviewed now, while every
implementation and evidence path is gated by the exact upstream semantic,
corpus, plan, binding, attachment, and build-profile capabilities it consumes.
The requirement graph is acyclic.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4901 | high | Early campaign language could admit W/M, past/history, or native-predicate rows before their semantics existed. Fixed with explicit blocked rows and exact landed-profile/evaluator/bridge gates. | MRS-003 Admission and dependency order |
| FND-4902 | high | Mutation implementation lacked a complete authoritative requirement-to-production-symbol dependency. Fixed by consuming Quire 0.31 `implements`, measuring the six current bindings, and requiring complete reviewed local bindings before selection without private inference. | FR-013, FR-015 |
| FND-4903 | high | Shared intake was described before attachment, truthful non-release build profile, and active local plan capabilities were demonstrated. Fixed by making each a separate admission gate and tracking the reusable gaps in quoin#363/#364. | FR-015-AC-4, FR-015-AC-5 |
| FND-4904 | medium | Fuzz and mutation appeared to be mandatory serial stages. Fixed: fuzz is applicable only at reviewed Fuzz-kind boundaries and mutation may proceed from the grounded property baseline; result-driven selections carry conditional edges only. | FR-012, FR-013, FR-014 |

## Classification and order

| Requirement | Class | Predecessor |
|---|---|---|
| FR-011 | enablement | landed M0/M4 and accepted MRS-003 |
| FR-012 | conditional feature | FR-011 plus applicable Fuzz-kind boundary |
| FR-013 | feature | FR-011 plus complete exact Quire `implements` bindings |
| FR-014 | conditional feature | FR-011 plus one exact admitted trigger |
| FR-015 | shared integration | relevant domain record plus shared plan/schema/retention capability |
| NFR-005 | cross-cutting constraint | constrains FR-011 through FR-015 |

The local DAG is `M0 + M4 + accepted MRS-003 -> FR-011 -> {FR-012,
FR-013, FR-014} -> applicable FR-015 intake`. W/M additionally waits for
tl-syntax #37, past/history for #38 plus evaluator support, and native rows for
implemented quire-contract-ir #63/#64. No reverse edge exists.
