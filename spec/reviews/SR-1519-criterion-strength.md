---
id: SR-1519
title: "Criterion-strength analysis of P-TL-151 (manual)"
type: SpecReview
analysis: criterion-strength
scope: "weak acceptance criteria (manual; Jev unavailable)"
review_set: subset
---

## Summary

Jev (typesafe.ai) is an external service not used here; this is a manual judgement of whether each criterion can fail. Most counted-population criteria can fail. Weak ones below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15191 | medium | FR-083-AC-3 duality laws (Xw, G, R) become tautologies if the evaluator implements the derived node through its dual; need independent implementations (FR-084-AC-4 partly covers). | FR-083-AC-3 |
| FND-15192 | medium | FR-093-AC-1/AC-2 and TC-253 Inspection cannot fail mechanically. | FR-093 |
| FND-15193 | medium | FR-305 negative controls are only `phi and not phi`; a solver or encoder returning unsat always passes half the corpus. | FR-305-AC-2 |
| FND-15194 | low | FR-088-AC-1, FR-087-AC-1 "every fixture in the manifest" is vacuous if the manifest is empty beyond the census check; require minimum count after retrieval. | FR-087, FR-088 |
| FND-15195 | low | NFR-300 targets and "accepted by two solvers" depend on solver identity unspecified (FR-304 "for example Z3 and cvc5; not verified"). | FR-304-AC-2 |
