---
id: SR-039
title: "Rust review — origin-complete history evaluator"
type: SpecReview
analysis: code-review
scope: "src/past.rs; src/evaluate.rs; src/horizon.rs; src/mapping.rs; src/main.rs; tests/past_history.rs"
review_set: subset
---

# Rust review — origin-complete history evaluator

## Summary

Reviewed idiomatic Rust, repository conventions, typed-error boundaries,
untrusted wire decoding, integer conversions, exact arithmetic, recursion and
work limits, panic/unsafe surface, correction identity, and semantic test
independence. Production additions contain no `unsafe`, panic, unwrap, expect,
unchecked integer cast, blocking async bridge, or lock.

## Verdict

**ACCEPTED** — every discovered defect was fixed before this review closed.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-3901 | high | Missing fixed-sample coordinates were generic clock refusals although FR-012 classifies them as incomplete. **FIXED** with a distinct incomplete-history error and exact preservation of all six owner states. | `HistoryError::IncompleteSample`; TC-053 |
| FND-3902 | high | Strong Previous initially bypassed a zero temporal-span limit despite being O[1,1]. **FIXED** by applying the cardinality-one boundary. | `PastEvaluator::at`; TC-052 |
| FND-3903 | high | `corrected` initially permitted a reused/regressed history revision until relation construction. **FIXED** at correction construction and independently rechecked for result relations. | `PositionHistoryDocument::corrected`; TC-051 |
| FND-3904 | medium | Persisted analysis/result strings were post-validated but not bounded at serde fields, and analysis records had no semantic validator. **FIXED** with bounded visitors and strict report validation. | `src/past.rs`; TC-051, TC-052 |
| FND-3905 | medium | The initial correction validator carried nine scalar arguments and failed strict Clippy. **FIXED** by grouping immutable dimensions in `CorrectionContext`. | `correction_relation`; strict Clippy |
| FND-3906 | low | Direct sample arithmetic labeled a non-positive period `NotNormalized`. **FIXED** with the exact `NonPositivePeriod` cause. | `fixed_sample_instant`; TC-053 |
| FND-3907 | medium | The serde path bounded propositions per row, but the programmatic history constructor initially admitted an oversized sorted proposition vector. **FIXED** with the same explicit limit in structural validation and a constructor-level boundary test. | `PositionHistoryDocument::validate_without_digest`; TC-051 |
| FND-3908 | high | A persisted non-original report could validate its internal reference shape, but there was initially no public operation to compare that reference with the complete predecessor bytes loaded by a caller. **FIXED** with `validate_with_predecessor`, which validates both reports, exact reference equality, and every stable correction-chain dimension. | `PastEvaluationReport::validate_with_predecessor`; TC-051, TC-056 |

## Semantic and safety result

- Independent test-local recursion checks O/H/S/T over generated histories;
  it calls no production evaluator, history admission, or history analysis.
- S uses witness `[a,b]` and left offsets `[a,j)`; an explicit mutation control
  makes offsets below nonzero `a` false while both witness endpoints succeed.
- T equals the structural Boolean dual of S, and primitive Y equals O[1,1] for
  propositions, constants, and Boolean nests at origin and later anchors.
- Every reverse subtraction is signed and checked; only proposition lookup
  supplies pre-origin false extension, so constants and nested operators retain
  their recursive truth.
- History rows and proposition IDs are bounded at decoding, ordered and
  gap-free; exact rational arithmetic uses checked i128/u128 intermediates and
  checked narrowing.
- Result digests bind every field except their own digest; non-original results
  require a valid direct predecessor, a matching corrected-history reference,
  stable chain dimensions, distinct digests, and advancing revisions.
