---
id: SR-063
title: "Independent failure-domain review of the M5 verification campaign"
type: SpecReview
analysis: failure-domain
scope: "FR-011 through FR-015 and NFR-005"
review_set: all
---

## Summary

**PASS after remediation.** Identity substitution, denominator drift, fuzz
window ambiguity, mutation population/restoration faults, survivor cycles,
proof widening, and hostile artifact admission now have deterministic refusal
or non-conclusive behavior.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-6301 | high | Unspecified property and proof ledger bytes allowed two conforming producers to disagree or restamp a changed population. Fixed with exact preimages, sort order, predecessor digest, and tombstones. | FR-011-AC-1, FR-014-AC-1 | missing-requirement |
| FND-6302 | high | A count-only or sparsely sampled fuzz run could claim plateau without observations covering both final windows. Fixed with exact feature sets, cadence, both anchors, and mandatory non-conclusion when either anchor is absent. | FR-012-AC-2 | wrong-requirement |
| FND-6303 | high | Mutation path normalization, byte spans, selection strata, and batch boundaries were implementation-defined. Fixed with FR-009 paths, checked half-open offsets, closed priority order, digest order, and consecutive batches of twenty. | FR-013-AC-1, FR-013-AC-2 | missing-requirement |
| FND-6304 | high | `duplicate_of` could form a cycle and an expired accepted risk could appear resolved. Fixed with a same-population acyclic root rule and RFC 3339 UTC expiry semantics. | FR-013-AC-4 | missing-requirement |
| FND-6305 | medium | Domain artifacts could bypass the campaign's already-reviewed resource envelope. Fixed by binding shared intake to `tl-mltl.corpus-limits/v1` unless an accepted shared contract is tighter. | FR-015-AC-4 | missing-requirement |
