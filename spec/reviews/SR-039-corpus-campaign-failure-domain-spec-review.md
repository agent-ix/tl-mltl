---
id: SR-039
title: "Failure-domain review of the corpus campaign"
type: SpecReview
analysis: failure-domain
scope: "FR-008 through FR-010, NFR-004, MP-002"
review_set: all
---

## Summary

**PASS after remediation.** Cell identity, independent-oracle isolation,
fixture filesystem/resource safety, external observation ambiguity, and blocked
native/profile states now fail closed before a conformance claim can escape.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-3901 | high | A production evaluator could write the expected value and then agree with itself. Fixed: the independent oracle may share public input types only and cannot call or read any owning production or expected-result path. | FR-008-AC-5, TC-039 | missing-requirement |
| FND-3902 | high | Manifest paths and sizes were unconstrained, allowing traversal, symlink substitution, non-regular files, or resource exhaustion before digest validation. Fixed with normalized tracked regular-file confinement, hard caps, and pre-decode digest checks. | FR-009-AC-6, TC-044, TC-045 | missing-requirement |
| FND-3903 | medium | Cell identity had no uniqueness key, so reclassification could disguise the same concern as a new cell. Fixed with a domain-separated digest over the canonical campaign/dimension tuple excluding lifecycle state. | FR-008-AC-1 | missing-requirement |
| FND-3904 | medium | Missing, stale, partial, failed, or aggregation-ambiguous external output could appear conclusive. Fixed: those states remain non-conclusive/refused and cannot become agreement or mapping correctness. | FR-010-AC-1, FR-010-AC-3 | wrong-requirement |
