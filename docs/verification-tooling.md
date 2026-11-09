# Verification tooling

How this crate's mutation, coverage and Kani evidence is produced and gated (TL-231, TL-232, TL-234). The measured numbers below come from a
single machine (Apple M5 Pro, Darwin 25.4.0, rustc 1.98.1, cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7,
nightly rustc 1.100.0-nightly 2026-09-21 for branch coverage, Kani 0.68.0 with CBMC 6.11.0) that was heavily loaded by other jobs.

## Targets

| Target | What it does | Budget |
| --- | --- | --- |
| `make mutants` | Full cargo-mutants run (`--no-shuffle`, so the population and order are fixed), then `scripts/mutation_gate.py` against `mutation-thresholds.toml` | hours; weekly, not part of `make ci` |
| `make mutants-diff` | cargo-mutants on `git diff $(MUTANTS_BASE)...HEAD` only (default `origin/main`); every mutant in the diff must be killed | minutes to an hour, per PR |
| `make coverage` | `cargo llvm-cov` line, function and region coverage on the pinned toolchain with a line floor (`COVERAGE_MIN_LINES`) | about 1 to 5 minutes |
| `make coverage-branch` | the same plus branch coverage (unstable, needs nightly with llvm-tools) | about 1 to 5 minutes |
| `make kani-check` | every Kani harness; part of `make ci` | about 5 minutes |

`CI-plumbing` modules are excluded from every threshold (`.cargo/mutants.toml` `exclude_globs` and
`exclude` in `mutation-thresholds.toml`). Equivalent mutants are listed in `exclude_re` with a one-line
reason. `cargo mutants` tests every mutant against the whole test suite, so a full run costs
(mutants x suite time) divided by `MUTANTS_JOBS`; use `MUTANTS_JOBS` to trade memory for time.

## Mutation thresholds

Kill rate is `caught / (caught + missed + timeout)`; unviable mutants count on neither side. The campaign
targets are 95% for the core and 85% for the whole crate. The values enforced by the gate are the measured
floors (they only ratchet up).

| Group | Files | Enforced floor |
| --- | --- | --- |
| `evaluator-core` | `["src/future/*.rs"]` | 95.0% |
| `crate` | `["src/*.rs", "src/*/*.rs"]` | 83.0% |

Measured at ccd968e (pre-rebase) with the working tree of this PR on this machine: core run: 81 mutants (`-f src/future/evaluate.rs -f src/future/horizon.rs`, 56 caught, 0 missed, 25 unviable, 100.00%) after `tests/mutation_survivors.rs` killed the 8 first-run survivors (first run 49 caught, 8 missed = 85.96%). Crate sample: the same core plus `--shard 0/3` (380 mutants generated, 172 executed before the budget cap: `src/context.rs`, `src/differential.rs`, `src/infinite/mod.rs`, `src/main.rs`) = 179 caught, 30 missed, 44 unviable, 85.65%.

The crate floor is the sampled rate minus a margin, because the sample covers only the files named
above; the first full `make mutants` run replaces the sample and the floor should then be raised to the
measured value.

The gate is shown to fail: with a floor above the measured rate it prints `FAIL` and exits 1.
The survivors in `src/infinite/mod.rs` below were measured before SR-223 (TL-223) merged, which killed 11 of them; re-measure before triaging them.

Unresolved survivors and timeouts of the measured populations (each is a `test_gap` to kill with a test,
or an equivalent mutant to list in `exclude_re` with a reason):

- `src/context.rs:26:17: replace <impl serde::Deserialize<'de> for RequiredContext>::deserialize::<impl serde::de::Visitor<'de> for ContextVisitor>::expecting -> core::fmt::Result with Ok(Default::default())`
- `src/context.rs:70:9: replace <impl core::fmt::Display for ContextualBindingError>::fmt -> core::fmt::Result with Ok(Default::default())`
- `src/differential.rs:188:9: replace <impl core::fmt::Display for ContextualComparisonError>::fmt -> core::fmt::Result with Ok(Default::default())`
- `src/differential.rs:257:53: replace && with || in compare_external`
- `src/infinite/mod.rs:1008:42: replace > with == in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1008:42: replace > with >= in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1008:9: replace || with && in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1009:39: replace > with == in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1009:39: replace > with >= in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1009:9: replace || with && in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1010:9: replace || with && in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1014:39: replace > with == in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1014:39: replace > with >= in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1035:13: replace || with && in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:1036:13: replace || with && in evaluate_prefix_safety_at`
- `src/infinite/mod.rs:367:13: replace || with && in RawTraceRequest<'_>::validate`
- `src/infinite/mod.rs:368:13: replace || with && in RawTraceRequest<'_>::validate`
- `src/infinite/mod.rs:375:41: replace || with && in RawTraceRequest<'_>::validate`
- `src/infinite/mod.rs:703:9: replace || with && in evaluate_lasso`
- `src/infinite/mod.rs:715:9: replace || with && in evaluate_lasso`
- `src/infinite/mod.rs:716:9: replace || with && in evaluate_lasso`
- `src/infinite/mod.rs:814:32: replace || with && in evaluate_model`
- `src/infinite/mod.rs:814:55: replace || with && in evaluate_model`
- `src/infinite/mod.rs:864:38: replace || with && in PrefixRequest<'_>::validate_shape`
- `src/infinite/mod.rs:883:17: replace || with && in PrefixRequest<'_>::validate_shape`
- `src/infinite/mod.rs:884:17: replace || with && in PrefixRequest<'_>::validate_shape`
- `src/infinite/mod.rs:88:9: replace <impl core::fmt::Display for InfiniteError>::fmt -> core::fmt::Result with Ok(Default::default())`
- `src/infinite/mod.rs:899:9: replace PrefixRequest<'_>::content_identity -> Result<String, InfiniteError> with Ok("xyzzy".into())`
- `src/infinite/mod.rs:899:9: replace PrefixRequest<'_>::content_identity -> Result<String, InfiniteError> with Ok(String::new())`
- `src/main.rs:47:24: delete ! in run`

## Coverage

`make coverage-branch` on this tree (all features, `ci_guard` excluded):

| Metric | Value | Metric | Value | Metric | Value | Metric | Value |
| --- | --- | --- | --- | --- | --- | --- | --- |
| lines | 82.13% | regions | 76.47% | functions | 73.91% | branches | 70.30% |

The stable-toolchain `make coverage` reports slightly different totals than the nightly branch run (it counts test sources too), so `COVERAGE_MIN_LINES` is set a point or two below the lower of the two. It is a floor, not a target; it fails the target when line coverage drops below it.

## Kani proofs (TL-232)

`make kani-check` runs every harness (`cargo kani --lib -Z stubbing`; stubbing is an
unstable Kani feature used only by the combinator proof). Kani 0.68.0, CBMC 6.11.0,
Apple M5 Pro, Darwin 25.4.0, measured under heavy machine load (load average 35 to 45), so
the times are an upper bound.

| Harness | Bounds | Result | Time |
| --- | --- | --- | --- |
| `horizon_bound_addition_matches_checked_add` (existing) | all `u32` bound, all `u64` child | verified | 0.1 s |
| `horizon_temporal_lookahead_is_exact_or_typed_overflow` | every temporal operator, full `u32` interval, full `u64` operand values: result equals the mathematical sum or the typed overflow, never a wrap | verified, 2 of 2 covers | 2 s |
| `evaluator_temporal_endpoints_full_range` | every `start <= end` over `u32`, every `u64` span limit | verified, 2 of 2 covers | 2 s |
| `evaluator_combinators_never_panic` | `Evaluator::at` stubbed; and/or/implies/equivalent/future/globally/until/release for every `u64` time, interval span <= 3 | verified, 3 of 3 covers | 96 s unloaded run, 146 to 308 s loaded |
| `closing_prefix_atomic_verdict_is_stable` | atomic proposition, trace <= 2 instants: decided exactly when `time < len`, stable under appending an instant | verified, 3 of 3 covers | 4 s |
| `preflight_never_panics_on_short_input` | wire lexer `preflight`, ASCII input of 0..=4 bytes, both byte ceilings, depth ceiling 1 | verified, 2 of 2 covers | 94 to 156 s |
| `identity_scanners_never_panic_on_short_input` | `scan_string` and `scan_value`, any input of 0..=16 bytes and any start offset | verified, 2 of 2 covers | 60 to 116 s |

Each doc comment states what the proof does not claim. What could not be proved, and why:

- A direct proof of the recursive `Evaluator::at` (evaluator no-panic for formulas of depth <= 2 over
  traces of length <= 4) exhausted CBMC memory (more than 12 GB) even for a depth-1 formula over a
  2-instant trace, with constant leaves. It is replaced by the compositional proofs above. The
  recursion bound itself rests on `max_recursion_depth` and `max_node_evaluations`, which the unit tests exercise.
- `preflight` over 8 bytes did not finish in 15 minutes (14 GB); the unconstrained-byte variant at 4
  bytes needed more than 29 GB because `str::from_utf8` loops. The shipped proof assumes ASCII and
  4 bytes. The requested 64-byte bound is not reached: the scanners are proved to 16 bytes (64 bytes
  exhausted 14 GB), and `serde_json` decoding is third-party code that is not modelled.
- The compound closing-prefix claim (a formula whose lookahead is below the prefix length is decided and
  stable) is proved only at the atomic level (SR-020 remains open for compound formulas).

Each proof was shown able to fail: replacing `checked_add` by `wrapping_add` in `add_bound` fails the
horizon proof (`assertion failed: u128::from(value) == exact`), and replacing `bytes.get(index)` by
`bytes[index]` in `scan_string` fails the scanner proof (`index out of bounds`).

`node_lookahead` was extracted from `lookahead` (behavior unchanged) so the arithmetic can be proved
without the `Vec` the traversal allocates, which CBMC could not handle within 12 GB.
