---
id: SR-057
title: TL-216 specification base review
type: SpecReview
analysis: base
scope: "agent-ix/tl-mltl@18c1d0bebcc980d114bd3e3a967c01f39409add8; TL-216; spec/requirements/{FR-038-past-c2po-export,FR-039-past-origin-contract}.md; spec/r2u2-v1-test-matrix.md"
review_set: subset
---

# TL-216 specification base review

## Summary

Reviewed both new requirements and TM-005 for IDs, criteria, verification links, and Quire structure.

## Verdict

**FAIL.** TM-005 is structurally invalid, so its coverage rows are not reconciled by Quire.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | TM-005 uses an unrecognized three-column table and omits the required Functional Requirement Coverage and Test Case Summary sections. `quire validate --scope .` reports both fields missing; the claimed TC-160–167/174 rows cannot serve as a validated test matrix. | spec/r2u2-v1-test-matrix.md:14; TM-005 |
