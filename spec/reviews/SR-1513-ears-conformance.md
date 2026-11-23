---
id: SR-1513
title: "EARS conformance analysis of P-TL-151 requirements"
type: SpecReview
analysis: ears-conformance
scope: "FR/NFR of P-TL-151 (left unfixed by author)"
review_set: subset
---

## Summary

`quire validate` EARS warnings introduced by the branch (baseline has none for these files). Each is a pair: ears:unclassifiable and ears:missing-subject on the line's `shall` statement. tl-parse adds none. The pattern is a Description that starts with "When ..., " followed by a noun phrase rather than `the <system> shall`, or a compound statement with two `shall`.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15131 | low | tl-syntax FR-332 line 25 (unclassifiable, missing-subject) | tl-syntax spec/requirements/FR-332-ltlf-profile-identity-and-admission.md:25 |
| FND-15132 | low | tl-syntax FR-333 line 19 (unclassifiable, missing-subject) | tl-syntax FR-333:19 |
| FND-15133 | low | tl-mltl FR-084 lines 21 and 22 (two pairs) | spec/requirements/FR-084-ltlf-reference-agreement.md:21-22 |
| FND-15134 | low | tl-mltl FR-091 line 21 | spec/requirements/FR-091-typed-loss-records.md:21 |
| FND-15135 | low | tl-mltl FR-092 line 19 | spec/requirements/FR-092-west-import-export.md:19 |
| FND-15136 | low | tl-mltl FR-093 line 15 | spec/requirements/FR-093-p-tl-151-specification-gate.md:15 |
| FND-15137 | low | tl-mltl NFR-047 lines 20 and 21 | spec/requirements/NFR-047-foreign-readers-bounded-and-fuzzed.md:20-21 |
| FND-15138 | low | tl-solve FR-305 line 22 | tl-solve spec/requirements/FR-305-smt-and-cnf-verdict-agreement.md:22 |
