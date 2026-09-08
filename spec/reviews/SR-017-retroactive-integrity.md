---
id: SR-017
title: "Integrity retrospective review of the tl-mltl specification corpus"
type: SpecReview
analysis: integrity
scope: "spec/: requirements, assurance, evidence, test matrix, plans, and historical reviews at origin/main 5c4ce2a"
review_set: all
---

## Summary

The corpus has complete current FR, NFR, and stakeholder identifiers, validated
relationships, declared verification methods, and compiled requirement-tag census
coverage. One qualification-boundary criterion nevertheless combines four independent
non-execution demonstrations into a single acceptance criterion, reducing traceability
of a failed control to the large compound statement.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | NFR-003-AC-2 is not atomic: PATH-stub non-execution, Quoin-stub failure, absent-input refusal, and injected-child refusal are independently meaningful controls but share one criterion and test reference. Split them into independently traceable criteria before a future qualification claim relies on the record. | NFR-003-AC-2; TC-019; `tests/shared_assurance.rs` |
