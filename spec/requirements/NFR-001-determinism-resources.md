---
id: NFR-001
title: Preserve determinism and bounded resource claims
type: NFR
---

# NFR-001: Preserve determinism and bounded resource claims

## Statement

All library results and serialized records shall depend only on identified
inputs. Algorithms shall check arithmetic, configured evaluation recursion and
work limits, and the mapping recursion boundary before performing impractical
temporal expansion. Each declared resource limit shall admit an input exactly at
the limit and refuse an input one past it with its own typed error, and the test
suite shall produce every variant of every public error enumeration at least once.

## Scope

All public library functions, CLI records, mapping manifests, context identities
and digests, and retained differential artifacts are in scope. The crate depends on `std`
(`std::error::Error`, `std::io`, `std::time::Instant`) and makes no `no_std`
claim (FR-027, ADR-003); a `no_std` build is out of scope for this requirement.

## Rationale

Reference-oracle and resource-selection evidence must be reproducible and must
not understate bounds after arithmetic or work-budget failure.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Nondeterministic output mismatches | 0 | 0 | Test |
| Wrapped or partial resource claims | 0 | 0 | Test |

## Verification

Requirement-tagged unit and integration tests repeat semantic and serialized
operations and exercise checked arithmetic and configured work limits. Limit-edge and error-variant tests are written against
the typed values, never message text. The unit and property suites are also run
under Miri on a pinned nightly to show the absence of undefined behavior.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Repeated evaluation, analysis, mapping, context binding, digesting, comparison, and serialization are byte-for-byte stable. | integration-testing (TC-003, TC-013, TC-014, TC-025, TC-028, TC-029) |
| NFR-001-AC-2 | Overflow and work-limit exhaustion are explicit errors with no partial success claim. | unit-testing (TC-004, TC-006) |
| NFR-001-AC-3 | Each limit in the Limit Register below is tested on both sides: a value equal to the limit is accepted; a value one past the limit is refused with the typed error that names that limit. The native run is never skipped. For a fixed ceiling (the recursion ceiling, the owner-wire maxima, the past size constants) it uses the production value; for a limit the caller supplies (`EvaluationLimits`, `PastEvaluationLimits`, infinite `EvaluationLimit`, the mapping work limit) it sets the limit to the exact requirement of the input. The Miri run (NFR-001-AC-5) runs the same code path with a lowered limit only for the heavy limits named in the Limit Register's Miri column. | unit-testing (TC-175) |
| NFR-001-AC-4 | Every variant of every public error and error-code enumeration is produced by at least one test that asserts the variant itself, not its message; a variant that no reachable input can produce is removed from the enumeration in the same change instead of being left untested. The one exception is `ContextualBindingError::Refused`, which preserves a refusal of the non-exhaustive upstream `FormulaBindingError`; it is produced only when tl-syntax adds a second binding refusal. | unit-testing (TC-176) |
| NFR-001-AC-5 | The unit and property suites complete under Miri, on the pinned nightly named in the Makefile, with no undefined-behavior report, using the recorded flags and reduced property case counts. Heavy limits run under Miri with a caller-lowered limit where the API takes one, and with a named `cfg(miri)` constant otherwise, exactly as the Limit Register states. | unit-testing (TC-177) |

## Limit Register

Accept side is a value equal to the limit; reject side is one past it. Limits
marked private or `pub(crate)` have no public input, so their tests live in the
defining module's `#[cfg(test)]` block. Several limits share one error variant;
the shared variant is marked and the test also asserts the distinguishing field.
Hard maxima that only clamp a caller value have no error of their own: the
accept side is a caller value equal to the hard maximum, the reject side is a
caller value above it, which is clamped and refused by the clamped limit's error.

| Limit (value) | Location | Typed error | Accept / reject side | Miri run |
|---|---|---|---|---|
| `MAX_RECURSION_DEPTH` (512), `pub(crate)` | `src/lib.rs` | `MappingError::RecursionDepthExceeded`, `EvaluationError::RecursionDepthExceeded`, `PastEvaluationError::RecursionDepthExceeded` | formula nesting 512 / 513 | production value |
| `EvaluationLimits.max_node_evaluations` | `src/future/evaluate.rs` | `EvaluationError::WorkLimitExceeded` | limit equals required work / one less | production value |
| `EvaluationLimits.max_temporal_span` | `src/future/evaluate.rs` | `EvaluationError::TemporalSpanExceeded` (shared with past) | window of `limit` offsets / `limit`+1 | production value |
| `EvaluationLimits.max_recursion_depth` | `src/future/evaluate.rs` | `EvaluationError::RecursionDepthExceeded` | depth = limit / limit-1 | production value |
| mapping render `work_limit` | `src/mapping/legacy.rs`, private `Renderer.limit` | `MappingError::WorkLimitExceeded` | visits = limit / limit+1 | production value |
| `PastEvaluationLimits.max_steps` (hard max `HARD_MAX_PAST_STEPS` 4,000,000) | `src/past/mod.rs` | `PastEvaluationError::StepLimitExceeded` | steps = limit / limit-1; clamp at hard max | caller-set; the clamp is asserted on the clamped limits without evaluation work |
| `PastEvaluationLimits.max_temporal_span` (hard max `HARD_MAX_TEMPORAL_SPAN` 100,000) | `src/past/mod.rs` | `PastEvaluationError::TemporalSpanExceeded` | span = limit / limit+1 | caller-set; hard-max clamp as above |
| `PastEvaluationLimits.max_recursion_depth` (clamped to 512) | `src/past/mod.rs` | `PastEvaluationError::RecursionDepthExceeded` | depth = limit / limit-1 | production value |
| `PastEvaluationLimits.max_input_positions` (clamped to `MAX_HISTORY_POSITIONS`) | `src/past/mod.rs` | `PastEvaluationError::InputPositionLimitExceeded` | positions = limit / limit+1 | caller-set |
| `MAX_IDENTITY_BYTES` (256), private | `src/past/mod.rs` | `HistoryError::HistoryIdentityTooLong` | 256-byte identity / 257 | production value |
| `MAX_UNIT_BYTES` (128), private | `src/past/mod.rs` | `ClockError::UnitTooLong` | 128-byte unit / 129 | production value |
| `MAX_HISTORY_POSITIONS` (1,000,000), private | `src/past/mod.rs` | `HistoryError::PositionLimitExceeded` | 1,000,000 positions / 1,000,001 | named `cfg(miri)` constant replaces the value in the module |
| `MAX_PROPOSITIONS_PER_POSITION` (100,000), private | `src/past/mod.rs` | `HistoryError::PropositionLimitExceeded` | 100,000 propositions / 100,001 | named `cfg(miri)` constant replaces the value in the module |
| `OwnerLimits.max_input_bytes` (8 MiB) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared by every owner limit; field name distinguishes) | document of `limit` bytes / `limit`+1 | caller-lowered |
| `OwnerLimits.max_output_bytes` (8 MiB) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | output of `limit` bytes / `limit`+1 | caller-lowered |
| `OwnerLimits.max_depth` (64) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | JSON depth 64 / 65 | production value |
| `OwnerLimits.max_string_bytes` (1 MiB) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | string of `limit` bytes / `limit`+1 | caller-lowered |
| `OwnerLimits.max_formula_nodes` (100,000) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | nodes = limit / limit+1 | production value (usage is a number, no allocation) |
| `OwnerLimits.max_formula_depth` (4,096) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | depth = limit / limit+1 | production value (usage is a number, no allocation) |
| `OwnerLimits.max_positions` (1,000,000) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | positions = limit / limit+1 | production value (usage is a number, no allocation) |
| `OwnerLimits.max_propositions` (100,000) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | propositions = limit / limit+1 | production value (usage is a number, no allocation) |
| `OwnerLimits.max_support` (100,000) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | support = limit / limit+1 | production value (usage is a number, no allocation) |
| `OwnerLimits.max_history_span` (100,000) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | span = limit / limit+1 | production value (usage is a number, no allocation) |
| `OwnerLimits.max_evaluation_steps` (4,000,000) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | steps = limit / limit+1 | production value (usage is a number, no allocation) |
| `OwnerLimits.max_recursion_depth` (512) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | depth = limit / limit+1 | production value |
| `OwnerLimits.max_visited_fields` (1,000,000) | `src/wire/common.rs` | `OwnerReadErrorCode::ResourceIncomplete` (shared) | fields = limit / limit+1 | caller-lowered |
| `OWNER_MAX` clamp of every field above | `src/wire/common.rs` | same as the field | caller value above `OWNER_MAX` is lowered to it, so usage at the maximum is accepted and one past is refused | production value for the semantic fields |
| `EvaluationLimit.max_nodes` | `src/infinite/mod.rs` | `InfiniteError::ResourceIncomplete` (shared by all six infinite limits) | limit equals required / one less | production value |
| `EvaluationLimit.max_positions`, `max_valuation_cells`, `max_states`, `max_completions`, `max_steps` | `src/infinite/mod.rs` | `InfiniteError::ResourceIncomplete` (shared) | each at its measured requirement / one less | production value |
| `EvaluationLimit.deadline` | `src/infinite/mod.rs` | `ResultReason::ResourceIncomplete` | not a numeric boundary; an already-expired deadline refuses, a far deadline accepts | production value |

The owner-wire rows marked caller-lowered run natively at the production
`OWNER_MAX` value, and under Miri at a lower value passed through `OwnerLimits`
for the same code path. The infinite-trace limits are asserted by the existing
inclusive-boundary tests, which carry the TC-175 trace tag.

## Dependencies

Constrains FR-001 through FR-005 and FR-007.
