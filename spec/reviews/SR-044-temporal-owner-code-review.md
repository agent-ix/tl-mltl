---
id: SR-044
title: "Code review — temporal evaluator owner boundary"
type: SpecReview
analysis: code-review
scope: "FR-018; TC-084; tl-mltl#66; tl-mltl#67"
review_set: subset
---

# Code review — temporal evaluator owner boundary

## Summary

Reviewed all eight owner contracts, their schemas and strict readers, future
and past request/evaluation paths, immutable corrections, the selected
Contract-IR mapping view, public provenance accessors, and compatibility paths
against FR-018.

## Verdict

**PASS after remediation.** Every finding was corrected in the implementation
and is exercised through TC-084 or the complete non-qualification regression
suite. No finding is deferred.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4401 | high | **FIXED:** Owner identities now hash the contract domain separator and the exact canonical struct-order bytes with only the top-level identity member omitted; readers independently reproduce that byte operation. | `wire::common`; `tc_084_schemas_are_pinned_and_all_readers_fail_closed` |
| FND-4402 | high | **FIXED:** Past requests accept only `ValidatedPositionHistory`, closing the raw-history bypass and matching the future lane's `ValidatedTrace` seam. | `wire::request`; `tc_084_future_and_past_requests_evaluate_and_strict_read` |
| FND-4403 | high | **FIXED:** All nested schema objects are closed and every one of the eight owner readers rejects unknown, duplicate, missing, reordered, trailing, and noncanonical input. | `schemas`; `tc_084_schemas_are_pinned_and_all_readers_fail_closed` |
| FND-4404 | high | **FIXED:** Decision and surrounding evidence retain independent axes while authority, subject, population, and pair context cross-wiring is rejected. | `wire::request`; `tc_084_owner_evidence_contexts_cannot_be_cross_wired` |
| FND-4405 | medium | **FIXED:** Constructor-private validated views expose the read-only profile, artifact, evaluator, clock, axis, lineage, source-result, support, and completeness provenance consumers need. | `wire::request`; `wire::report`; `mapping::contract_ir` |
| FND-4406 | low | **FIXED:** Compatibility assertions now name the exact tl-syntax revision and the reviewed post-reorganization production module census. | `tests/cli.rs`; `tests/future_parity.rs` |
| FND-4407 | low | **FIXED:** A dead availability-state marker and its lint suppression were removed; owner-state preservation is demonstrated through executable result behavior. | `wire::report`; TC-084 non-value rows |

The mapping remains a TL-owned projection: it introduces no Contract-IR parser,
evaluator, vocabulary, callback, trust flag, or Boolean coercion.
