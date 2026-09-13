---
id: SR-058
title: "Independent evidence-method review of the current corpus campaign"
type: SpecReview
analysis: evidence
scope: "FR-008 through FR-010, NFR-004, MP-002, TM-002, PLAN-006"
review_set: all
---

## Summary

**PASS with disclosed tooling limitation.** Quoin 0.23.1 evaluated the 27
campaign obligations: zero mismatch, uncatalogued, or inconclusive results.
The matrix and tasks allocate property, integration, snapshot, mutation, and
replay evidence without claiming planned tests have run.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5801 | high | The earlier test plan could not prove the cell/set identity because the byte preimage and registry boundary were absent. Fixed in TC-037/TC-038 with exact digest vectors, required-class coverage, and reorder/delete/insert mutations. | FR-008-AC-1, TC-037, TC-038 |
| FND-5802 | high | “All valid combinations” and “non-conclusive or refusal” provided no unique TC-046 oracle. Fixed with the closed state table and deterministic pre-admission versus admitted-run classification. | FR-010-AC-1, FR-010-AC-3, TC-046 |
| FND-5803 | high | Resource tests had no exact threshold values or wire-conversion boundary. Fixed by assigning TC-044/TC-045 every `tl-mltl.corpus-limits/v1` maximum plus checked `u64` overflow/conversion failures. | FR-009-AC-6, TC-044, TC-045 |
| FND-5804 | medium | TC-048 named a routing gate without a structured input. Fixed by validating PLAN-006 frontmatter and its linked GitHub issues for owner, consumer, method, predecessor, resume, language, and framework fields. | PLAN-006, TC-048 |
| FND-5805 | low | At `spec-artifacts-process` revision `737987b`, Quire reports 131 total rows, 99 backed rows, the campaign's 19 criteria and 13 test cases unbacked, and zero status lies. The unpinned installed module can instead count eight reference-only suite rows, so the exact module identity remains required evidence context. | TM-002, assurance/pins.json, quire-contract-ir#21 |
