---
id: SR-038
title: "Code review — origin-complete history evaluation"
type: SpecReview
analysis: code-review
scope: "tl-mltl#63; src/past.rs; future evaluator compatibility; dependency provenance; tests/past_history.rs"
review_set: subset
---

# Code review — origin-complete history evaluation

## Summary

Reviewed the complete `tl-mltl#63` implementation against accepted upstream
FR-011/FR-012, Task-003, and TC-048 through TC-053/TC-056. The public API now
has one typed path for strict position histories, checked history requirements,
anchored Boolean evaluation, exact clocks, bounded work, and immutable result
corrections. Existing future evaluation, horizon, mapping, formula-v1, and
retained-corpus behavior remain separate.

## Verdict

**ACCEPTED** — every finding below was fixed and backed by a compiled test or
an exact repository gate. No finding is deferred.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-3801 | high | Advancing the compiled tl-syntax pin initially made the retained W/M corpus look as though it came from the new dependency revision. **FIXED** with an independent future-corpus basis and guard without changing corpus bytes. | `TL_SYNTAX_FUTURE_CORPUS_BASIS`; `assurance/pins.json`; checksum gates |
| FND-3802 | high | New enum variants stopped compilation, and Boolean-only past-profile input could enter future horizon analysis. **FIXED** with exhaustive past-node refusals and an explicit horizon profile guard. | `src/evaluate.rs`; `src/horizon.rs`; `src/mapping.rs`; `tests/past_history.rs` |
| FND-3803 | medium | Public documentation omitted the new strict-history/non-Pending boundary. **FIXED** with the complete `evaluate_past` contract summary. | `README.md` |
| FND-3804 | medium | Initial mutation coverage did not validate the history-requirement wire independently. **FIXED** with strict deserialization/validation and positive plus mutation controls. | `src/past.rs`; `tests/past_history.rs` |

## Verification observed

- All implementation/regression targets excluding the explicitly paused
  shared-assurance workflow pass: 63 tests total, including 12 Task-003 tests,
  plus all example targets.
- `cargo clippy --all-targets --all-features -- -D warnings` passes.
- Rust 1.75 all-target/all-feature check passes.
- Release build, rustdoc, unsafe audit, Cargo Deny licenses/sources, both
  retained-corpus checksums, and the compiled traced-test census pass.
- The shared-assurance target remains outside this implementation lane: it
  requires producer-created `target/assurance` inputs and the repo-pinned
  ix-flow 0.0.4 while the host exposes 0.2.3. No qualification input was
  generated and no gate was weakened.
