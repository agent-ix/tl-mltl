---
id: SR-019
title: "Evidence retrospective review of the tl-mltl specification corpus"
type: SpecReview
analysis: evidence
scope: "spec/requirements, spec/evidence, spec/assurance, test matrix, and Quoin/Quire advisor integration at origin/main 5c4ce2a"
review_set: all
---

## Summary

The requirements consistently declare Test or Inspection methods and name their matrix
rows and producer-owned suites. The mandatory deterministic advisor step could not be
completed: `quoin advise --json` rejected the installed `quire 0.31.0` while claiming it
could not determine a version meeting its documented `>= 0.21.0` minimum. This is a
tool-contract defect, not evidence that the obligations are unverified.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | medium | Quoin's evidence advisor cannot consume the installed Quire CLI, so this review cannot mechanically compare authored verification methods with the catalog recommendations. Repair the Quoin-to-Quire version probe, then rerun `quoin advise` and record any actual method mismatches separately. | FR-001 through FR-007; NFR-001 through NFR-003; `quoin advise --json`; `quire --version` |
