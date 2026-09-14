---
id: SR-046
title: "Gap analysis — temporal evaluator owner boundary"
type: SpecReview
analysis: gap-analysis
scope: "FR-018-AC-1 through FR-018-AC-6; TC-084; tl-mltl#66; tl-mltl#67"
review_set: subset
---

# Gap analysis — temporal evaluator owner boundary

## Summary

Traced each FR-018 obligation through owner schema, public admission API,
validated view, evaluator/mapping implementation, exact wire identity, negative
mutation, resource boundary, and compiled Rust test.

## Verdict

**VALIDATED after remediation.** FR-018-AC-1 through FR-018-AC-6 all have
production implementations and real `Trace: TC-084` tests. The suite covers
both temporal lanes, every supported profile and past operator, every owner
contract, all assertion-state products, corrections, substitutions, and exact
and one-over resource limits. There is no unowned vocabulary or deferred
implementation gap.

## Requirement trace

| Requirement | Implementation evidence | Test evidence | Result |
|---|---|---|---|
| FR-018-AC-1 | strict future/past request and result owners; constructor-private validated views | `tc_084_future_and_past_requests_evaluate_and_strict_read`; `tc_084_all_profiles_past_operators_and_supported_clocks_use_one_owner_path` | COVERED |
| FR-018-AC-2 | eight immutable schema byte constants/digests and eight bounded readers | `tc_084_schemas_are_pinned_and_all_readers_fail_closed`; `tc_084_request_result_map_and_lineage_reject_substitution` | COVERED |
| FR-018-AC-3 | four independent axis references plus completeness and availability references | `tc_084_owner_axes_remain_independent_and_non_values_are_total`; `tc_084_owner_evidence_contexts_cannot_be_cross_wired` | COVERED |
| FR-018-AC-4 | source-result-derived selected map with final-Boolean-only value admission | `tc_084_owner_axes_remain_independent_and_non_values_are_total`; `tc_084_profile_clock_and_resource_refusals_never_coerce_boolean` | COVERED |
| FR-018-AC-5 | lane/profile/clock/anchor/settlement/correction and owner-context validators | `tc_084_all_profiles_past_operators_and_supported_clocks_use_one_owner_path`; `tc_084_request_result_map_and_lineage_reject_substitution`; `tc_084_owner_evidence_contexts_cannot_be_cross_wired` | COVERED |
| FR-018-AC-6 | clamped byte/depth/string/node/position/proposition/support/span/step/recursion/visit accounting and legacy re-exports | `tc_084_exact_and_one_over_limits_cover_each_owner_dimension`; complete non-qualification regression suite | COVERED |

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4601 | high | **FIXED:** Admission parity between raw past history and validated future trace was closed by requiring the validated past owner view. | `wire::request`; TC-084 future/past admission |
| FND-4602 | high | **FIXED:** Nested schema closure and negative-reader coverage now apply uniformly to all eight contracts. | `schemas`; TC-084 reader mutations |
| FND-4603 | high | **FIXED:** The complete 192-row axis/completeness/availability product proves independent representation and total non-value mapping. | TC-084 owner-axis product |
| FND-4604 | medium | **FIXED:** Exact and one-over tests cover every FR-018 resource dimension, including output allocation and refused-attempt usage. | TC-084 exact/one-over limits |
| FND-4605 | medium | **FIXED:** Public accessors make required provenance observable without allowing consumers to construct or mutate validated state. | `wire::request`; `wire::report`; `mapping::contract_ir` |
