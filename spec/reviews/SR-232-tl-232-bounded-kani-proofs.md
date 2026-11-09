---
id: SR-232
title: "spec-correctness - bounded Kani proofs for interval arithmetic, evaluator combinators, wire scanners and the atomic closing prefix (TL-232)"
type: SpecReview
analysis: spec-correctness
scope: "src/future/kani_evaluate.rs, src/future/horizon.rs, src/wire/kani_wire.rs, the make kani-check target"
review_set: all
---

## Summary

TL-232 extends the single horizon `checked_add` proof to seven verified Kani harnesses and runs them in
`make kani-check` (part of `make ci`). Kani 0.68.0 with CBMC 6.11.0 verified all seven with no failed
check and every cover property satisfied. Three of the four requested proof families are delivered in a
narrower form than requested, because CBMC ran out of memory on the direct formulations; each narrowing is
stated in the harness doc comment and below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2321 | low | Interval arithmetic is proved exactly. `node_lookahead` (extracted from `lookahead` with unchanged behavior) returns `interval.end + max(operands)` when that sum fits `u64` and the typed `ArithmeticOverflow` otherwise, for every temporal operator, every `u32` interval and every `u64` operand value. `evaluator_temporal_endpoints_full_range` proves the evaluator's span computation never overflows for any `start <= end`. | `horizon_temporal_lookahead_is_exact_or_typed_overflow`, `evaluator_temporal_endpoints_full_range` |
| FND-2322 | medium | Evaluator no-panic is NOT proved for whole formulas of depth <= 2 over traces of length <= 4. A direct proof through the recursive `Evaluator::at` exhausted CBMC memory (over 12 GB) even for one operator over a constant leaf. It is replaced by a compositional proof: with `at` stubbed, every combinator (and, or, implies, equivalent, future, globally, until, release) returns `Ok` or a typed error for every `u64` verdict time and every interval over the full `u32` range (the proof sets the span limit to 3, so wider intervals take the typed span-limit error). The stub returns an arbitrary verdict, so this proves no-panic and typed errors only, not that any verdict is correct; `cover!` shows reachability, not correctness. Recursion depth and work are bounded at run time by `max_recursion_depth` and `max_node_evaluations`, which unit tests cover but no proof does. | `evaluator_combinators_with_stubbed_at_never_panic` |
| FND-2323 | medium | Wire decode no-panic is proved for the hand-written lexical stage only, and below the requested 64-byte bound: `preflight` for ASCII input of 0..=4 bytes (the 8-byte version did not finish in 15 minutes; the non-ASCII variant needed over 29 GB because of `str::from_utf8`), and `scan_string` / `scan_value` for 0..=16 bytes (64 bytes exhausted 14 GB). `serde_json` decoding is third-party code and is not modelled. | `preflight_never_panics_on_short_input`, `identity_scanners_never_panic_on_short_input` |
| FND-2324 | medium | Closing-prefix is proved at the atomic level only: an open-prefix proposition lookup is decided exactly when `time < len` and a decided answer is unchanged by appending an instant. The compound claim (a formula whose lookahead is below the prefix length is decided and stable) is not proved; the unfinished SR-020 harness stays open for compound formulas. | `closing_prefix_atomic_verdict_is_stable` |
| FND-2325 | low | No proof is vacuous: every harness carries cover checks (2 to 3 each) and all are satisfied; unwinding assertions are on (Kani default), so an incomplete unwind fails the harness, as the first `preflight` attempt at unwind 6 without the ASCII assumption showed (`unwinding assertion loop 0`). | Kani output |
| FND-2326 | low | Each proof was shown able to fail. Replacing `checked_add` by `wrapping_add` in `add_bound` fails `horizon_temporal_lookahead_is_exact_or_typed_overflow` (`assertion failed: u128::from(value) == exact`). Replacing `bytes.get(index)` by `bytes[index]` in `scan_string` fails the scanner proof (`index out of bounds`). Both edits were reverted. | harness runs |
| FND-2327 | low | The proof sources are compiled only under `cfg(kani)`, so ordinary and MSRV builds are unchanged; they are excluded from mutation and coverage. The only non-verifier edit is the extraction of `node_lookahead`. The stub proof needs Kani's unstable `-Z stubbing`. FR-023 and TC-119 to TC-124 describe a digest-bound proof ledger that this work does not implement, so no FR-023 matrix row is claimed as backed. | `Makefile`, `spec/requirements/FR-023-bounded-kani-claims.md` |
| FND-2328 | medium | TL-372, 2026-11-08. The harness list is now checked by exact qualified name: `scripts/kani-harnesses.txt` is compared with the `#[kani::proof]` functions by `tests/kani_harnesses.rs` (a renamed, missing or unlisted harness fails `cargo test` and the first step of `make kani-check`), and `scripts/kani-gate.sh` runs each name with `--harness NAME --exact`; Kani 0.68.0 exits non-zero for a name that matches nothing. Kani exits 0 with `VERIFICATION:- SUCCESSFUL` when a `cover!` is unsatisfied (measured: `cover!(... && false)` gave `2 of 3 cover properties satisfied`, exit 0), so TL-232's gate did not fail a vacuous proof; the gate script now requires every cover satisfied. The horizon harness gained two cover checks. The combinator harness was renamed `evaluator_combinators_with_stubbed_at_never_panic` and its doc says no-panic and typed-error only; its span-limit wording is corrected (intervals are unconstrained, wider ones take the typed error). Backs FR-023-AC-1, AC-2 and AC-4 (TC-120, TC-123); FR-023-AC-3 and TC-121 (campaign report records) remain unbacked. FR-023 as a whole is not claimed. | `tests/kani_harnesses.rs`, `scripts/kani-gate.sh` |
| FND-2329 | medium | TL-372, 2026-11-08. Not proved, with the measurements (RSS of the kani process tree sampled every 2 s, run aborted above 8 GB; load average 12 to 16). (1) Whole-formula evaluator no-panic through the real `Evaluator::at`: depth 1 with symbolic operator, leaves, interval, trace of at most 3 instants: aborted at 8037 MB after 390 s. Per operator `not(leaf)`: aborted at 8015 MB after 425 s. A concrete one-node formula `Proposition` with a symbolic trace and unwind 2, no recursion: aborted at 8063 MB after 236 s. The cost is therefore the statically inlined body of `at` (all operator arms, with recursion and nested windows unrolled), not the formula or trace size. (2) Compound closing-prefix (decided verdict stable under append; decided once the prefix passes `time + lookahead`) over the same depth-1 formulas: aborted at 8014 MB after 337 s. (3) Wire lexer for arbitrary non-ASCII bytes, `lex_usage` split out of `preflight`: 4 bytes aborted at 8392 MB after 48 s, 3 bytes at 8945 MB after 39 s, 2 bytes at 8388 MB after 41 s; at 3 bytes with unwind 5 the unwinding assertion failed. Preflight beyond ASCII 0..4 bytes therefore stays unproved and the refactor was reverted. For scale, the existing ASCII `preflight` proof itself peaked at 10789 MB in 89 s. A compositional closing-prefix lemma (each combinator preserves stability given stable children) needs two coupled stub runs and was not attempted. All abandoned harnesses were removed; no source of them remains in the tree. | measured runs |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro), load average 35 to 45. Toolchain: rustc 1.98.1, Kani Rust
Verifier 0.68.0 (cargo plugin), CBMC 6.11.0. Command: `cargo kani --lib -Z stubbing --output-format terse`.

| Harness | Result | Cover | Time |
| --- | --- | --- | --- |
| `horizon_bound_addition_matches_checked_add` | verified | none | 0.1 s |
| `horizon_temporal_lookahead_is_exact_or_typed_overflow` | verified | 2 of 2 | 2 s |
| `evaluator_temporal_endpoints_full_range` | verified | 2 of 2 | 2 s |
| `evaluator_combinators_with_stubbed_at_never_panic` (no-panic and typed error only) | verified | 3 of 3 | 96 s alone, 308 s under load |
| `closing_prefix_atomic_verdict_is_stable` | verified | 3 of 3 | 3 to 4 s |
| `identity_scanners_never_panic_on_short_input` | verified | 2 of 2 | 60 to 116 s |
| `preflight_never_panics_on_short_input` | verified | 2 of 2 | 94 to 156 s |
