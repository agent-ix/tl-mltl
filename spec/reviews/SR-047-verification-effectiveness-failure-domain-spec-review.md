---
id: SR-047
title: "Failure-domain review of the verification-effectiveness campaign"
type: SpecReview
analysis: failure-domain
scope: "FR-011 through FR-015, NFR-005, MP-003 through MP-006"
review_set: all
---

## Summary

**PASS after remediation.** Identity drift, vacuous or self-referential
properties, stochastic fuzz ambiguity, mutation instability, proof vacuity,
unsafe artifact paths, partial records, and shared-state erasure now fail
closed or remain explicitly non-conclusive.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-4701 | high | A property could agree with the production implementation because both derived the same rule. Fixed with independent-oracle provenance and a seeded fault that the claimed oracle must discriminate. | FR-011-AC-2, FR-011-AC-3 | missing-requirement |
| FND-4702 | high | Fuzz feature counts could conceal one disappearing and one new feature. Fixed: plateau requires exact feature sets or stable bitmap identity under one unchanged instrumentation identity. | FR-012-AC-2, MP-004 | wrong-requirement |
| FND-4703 | high | Mutation instability or timeout suppression could inflate the killed fraction. Fixed with three initial controls, controls around batches of at most twenty, suspect batches, complete states, and the conservative caught/(caught+missed+timeout) estimator. | FR-013-AC-2, FR-013-AC-3, MP-005 | missing-requirement |
| FND-4704 | high | A loop-free proof could pass an inert unwind flag and appear checked. Fixed: unwind is not-applicable only after an exact retained loop/recursion census; disabled, incomplete, or inert checks are non-conclusive. | FR-014-AC-1, FR-014-AC-2 | wrong-requirement |
| FND-4705 | high | Missing, stale, hostile, partial, or oversized evidence could be inferred from process status or logs. Fixed with typed state preservation, normalized tracked-file confinement, digest-before-decode, and finite caps. | FR-015-AC-2 through FR-015-AC-4 | missing-requirement |
