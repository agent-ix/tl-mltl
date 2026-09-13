---
id: SR-055
title: "Independent failure-domain review of the current corpus campaign"
type: SpecReview
analysis: failure-domain
scope: "FR-008 through FR-010, NFR-004, MP-002, TM-002"
review_set: all
---

## Summary

**PASS after remediation.** The independent pass closed denominator, digest,
filesystem/resource, state-classification, and authority escape paths. Blocked
producer rows cannot yield fixtures or verdicts, and no callback/plugin or
user-supplied evaluator surface is introduced.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-5501 | high | A closed enum catalog plus an author-supplied cell list did not prove the selected obligation population was retained. Fixed with one reviewed ordered registry, required-class presence, pinned set digest, and successor/tombstone handling. | FR-008-AC-1 | missing-requirement |
| FND-5502 | high | Platform path interpretation, unchecked count/byte conversion, and unspecified caps could admit traversal or resource exhaustion before decode. Fixed with `/`-only exact UTF-8 components, backslash refusal, checked `u64` preflight, and closed numeric maxima. | FR-009-AC-6 | missing-requirement |
| FND-5503 | high | A manifest carrying its own digest has no finite byte preimage, and the aggregate corpus hash had no ordering. Fixed by carrying the exact-byte manifest hash externally and hashing sorted family/hash pairs under a separate domain. | FR-009-AC-1 | wrong-requirement |
| FND-5504 | high | Missing identity, tool failure, partial output, and unavailable execution could be classified either as refusal or non-conclusive. Fixed by refusing malformed identity before comparison and reserving non-conclusive for an admitted exact run with unusable output. | FR-010-AC-3 | missing-requirement |
| FND-5505 | medium | A blocked native/past row could accidentally acquire denominator credit or a fixture. Retained rule: blocked rows require exact unresolved dependencies and carry neither verdict nor canonical fixture. | FR-008-AC-2, MRS-002 | correct-requirement-no-evidence |
