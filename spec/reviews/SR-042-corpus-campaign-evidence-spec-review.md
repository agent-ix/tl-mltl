---
id: SR-042
title: "Evidence-method review of the corpus campaign"
type: SpecReview
analysis: evidence
scope: "FR-008 through FR-010, NFR-004, MP-002, TM-002"
review_set: all
---

## Summary

**PASS after remediation.** The pinned Quoin advisor evaluated all 27 new AC
and measurement obligations with zero mismatch, uncatalogued, or inconclusive
results. The matrix allocates property, integration, mutation, replay, and
claim-boundary evidence without counting generated inputs or external runtime
acceptance as conformance.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4201 | medium | The automated-release metric was initially authored as Inspection despite being a zero-occurrence invariant that the planned report/schema gate can test. Fixed by selecting Test, eliminating the advisor mismatch. | NFR-004-M-8, TC-049 |
| FND-4202 | high | Stored expected values could be self-oracled. Fixed with independent control-flow isolation and explicit self-oracle refusal, supplemented by mutation of oracle identities and results. | FR-008-AC-5, TC-039, TC-047 |
| FND-4203 | medium | Fuzz/property/mutation output could inflate canonical coverage. Fixed by reporting generated populations separately and requiring reviewed promotion with an independent expected result. | FR-009-AC-4, TC-045, TC-049 |
| FND-4204 | low | New external execution is unavailable under the current runtime ruling. Retained observations remain non-qualification context; Rust semantics, output snapshots, loss reports, and independent oracles own mapping evidence. | FR-010-AC-2, FR-010-AC-4 |
