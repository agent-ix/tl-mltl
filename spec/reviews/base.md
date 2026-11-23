---
id: SR-1500
title: "base review of TL-150 numeric specs (TL-150-spec-tl-mltl)"
type: SpecReview
analysis: base
scope: "tl-mltl ADR-014, FR-053..060, NFR-017, StR-014, TM-016, PLAN-020; corpus/r2u2-v4.2"
review_set: all
---

## Summary

Base checklist: quoin validate no findings; 27 new ACs all mapped; matrix delta +27 untagged rows, +7 requirement-without-design.

Method note: independent fresh-context review, no build tools run. Skill lenses were applied manually by the reviewer in one pass; the criterion-strength lens normally uses the external Jev service, which was not used. All specs are unbuilt and ADRs are proposed.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Ungrounded C2PO/R2U2 facts: retained corpus has only bool INPUT, FTSPEC and PTSPEC; numeric types, float tolerance, abs, contract construct and per-step contract output are not in corpus/r2u2-v4.2, past-c2po-v1 or r2u2-live-v1. FR-060 labels the decomposition export "grounded" without retained evidence of how R2U2 reports contract status. | FR-060:26; FR-057; corpus/r2u2-v4.2/formulas.c2po |
| FND-002 | medium | FR-059-AC-1 asks for all nine (a,g) combinations on closed and prefix, but the closed profile yields no Pending; closed padding also makes tail positions report padded invalid/inactive. | FR-059; FR-055 |
| FND-003 | medium | Differential records have no disposition for R2U2 emitting no verdict at tail positions (6,912 and 192 counts). | FR-058; FR-060 |
| FND-004 | medium | ADR-014 here collides in name with tl-syntax ADR-014; FR-057 cites unqualified ADR-015. | ADR-014; FR-057 |
| FND-005 | medium | ADR-015 error bound omits decimal-to-float conversion of inputs and literal thresholds and the application of tolerance to inequalities. | tl-syntax ADR-015; FR-057 |
| FND-006 | low | FR-054 reference is a throwaway script plus tl-oracle on Boolean traces; verified independence claim holds for temporal part only. | FR-054 |
| FND-007 | low | Hash/digest use: retained exchange digests and one-byte-alteration test, schema digests, graph identity, pinned revision and profile (checklist antipattern; follows NFR-004 precedent). | FR-053; FR-058; FR-059 |
