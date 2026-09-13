---
id: SR-041
title: "Code review — shared past/history evaluator replay"
type: SpecReview
analysis: code-review
scope: "corpus/past-history, tests/past_history_corpus.rs, Makefile"
review_set: subset
---

# Code review — shared past/history evaluator replay

## Summary

Reviewed native history construction, evaluation, correction, serialization,
refusal execution, and corpus pinning against Task-005.

## Verdict

**PASS after remediation.** The test constructs native histories, runs native
required-history analysis and evaluation, validates immutable correction chains,
round-trips result wires, and executes every shared refusal identity.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4101 | high | All twelve refusal rows now dispatch through typed native error assertions. | `every_shared_refusal_case_exercises_its_native_boundary` |
| FND-4102 | high | Correction rows rebuild originals and validate the exact direct predecessor. | `exact_shared_corpus_replays_history_analysis_evaluation_and_corrections` |
| FND-4103 | medium | Stale history SHA-256 now fails production wire deserialization. | `corpus_digest_and_history_identity_mutations_are_detected` |
