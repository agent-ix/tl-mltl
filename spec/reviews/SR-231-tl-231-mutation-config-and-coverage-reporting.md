---
id: SR-231
title: "spec-correctness - cargo-mutants configuration, kill-rate thresholds and coverage reporting (tl-mltl, TL-231, TL-234)"
type: SpecReview
analysis: spec-correctness
scope: "mutation configuration, thresholds, scripts and Makefile verification targets of tl-mltl"
review_set: all
---

## Summary

TL-231 and TL-234 add a reproducible cargo-mutants configuration, a kill-rate gate, a diff gate and llvm-cov
line and branch coverage reporting to tl-mltl, as separate Make targets outside `make ci`. This record is the dated
evidence (2026-11-08 and 2026-11-09); `docs/verification-tooling.md` carries no figures. The mutation targets were not met crate-wide: the crate-wide rate (85.65%) is a core plus shard 0/3 sample, not a full run, and the floors (core 95, crate 83) are the sampled rate minus a margin; the evaluator core itself met 95% (100.00%).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2311 | medium | Mutation measurements (cargo-mutants 27.0.0, `--no-shuffle`, taken on the tree before the final rebase (pre-rebase)): Evaluator core (`src/future/*.rs`): first run 49 caught, 8 missed, 25 unviable (85.96%); after `tests/mutation_survivors.rs` 56 caught, 0 missed, 25 unviable (100.00%). Crate sample (core plus `--shard 0/3`, 172 of 380 shard mutants executed before the budget cap): 179 caught, 30 missed, 44 unviable (85.65%). Timeouts count against the rate. | `mutation-thresholds.toml` |
| FND-2312 | low | The 8 first-run survivors were four `Display` impls, two work-limit comparisons in `Evaluator::consume`, `validate_trace -> Ok(())`, and one verifier-only mutant under `cfg(kani)`; the first seven are killed by `tests/mutation_survivors.rs`, the eighth is listed in `exclude_re`. The crate sample was taken before TL-223 (SR-223) merged and killed 11 of the `src/infinite/mod.rs` survivors, so those survivors are not listed here and should be re-measured. | `docs/verification-tooling.md` |
| FND-2313 | low | The enforced mutation floors are the measured sampled rates minus a margin, not the 95% and 85% targets; groups below target have open survivors, listed below, that are test gaps to kill. | `mutation-thresholds.toml` |
| FND-2314 | low | Every gate was shown able to fail: `scripts/mutation_gate.py` exits 1 with `FAIL` when a floor exceeds the measured rate, on an empty population or a failed baseline, and exits 2 on unreadable input; `make coverage COVERAGE_MIN_LINES=99` exits non-zero. `scripts/run_mutants.py` accepts cargo-mutants exit 0, 2 and 3 so the gate always prints its table. | `scripts/mutation_gate.py` |
| FND-2315 | low | Coverage re-measured from a clean `target/llvm-cov-target` on 2026-11-09, all features, `ci_guard` excluded. Stable `make coverage`: 87.83% lines / 85.76% regions / 79.35% functions. Nightly `make coverage-branch`: 83.49% lines / 55.61% regions / 79.49% functions / 76.34% branches. The two toolchains report different line totals and the nightly region figure is much lower, so `COVERAGE_MIN_LINES` is 80, below the lower line figure; both targets pass at that floor. A reviewer measured lower totals on a fresh clone than these; the difference could not be reproduced here, so the floor sits below both readings that were available. | `Makefile` |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro), heavily loaded (load average 35 to 45). Toolchain: rustc 1.98.1,
nightly rustc 1.100.0-nightly (1303417c4 2026-09-21), cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7, quire 0.38.3
(engine 0.52.1). Mutation order is fixed by `--no-shuffle`; there is no seed in this mode. The per-mutant timeout
is the cargo-mutants automatic value (three times the unmutated baseline, minimum 120 s in this crate's
`.cargo/mutants.toml`). Wall times are not equality inputs. The crate-wide rates are core plus `--shard 0/3` file
samples (cargo-mutants shards by file), not full runs.

## Open survivors and timeouts of the measured populations

Each is a test gap to kill, unless listed as equivalent above.

- `src/context.rs:26:17: replace <impl serde::Deserialize<'de> for RequiredContext>::deserialize::<impl serde::de::Visitor<'de> for ContextVisitor>::expecting -> core::fmt::Result with Ok(Default::default())`
- `src/context.rs:70:9: replace <impl core::fmt::Display for ContextualBindingError>::fmt -> core::fmt::Result with Ok(Default::default())`
- `src/differential.rs:188:9: replace <impl core::fmt::Display for ContextualComparisonError>::fmt -> core::fmt::Result with Ok(Default::default())`
- `src/differential.rs:257:53: replace && with || in compare_external`
- `src/main.rs:47:24: delete ! in run`

## TL-371 results (2026-11-12)

TL-371 ran cargo-mutants 27.0.0 (`--no-shuffle`) over the whole crate, killed or explained every survivor, and
replaced the sampled floors in `mutation-thresholds.toml` with floors taken from the measured rates.

- First full run (`-j 4`, 782 s timeout, 1465 mutants): 600 caught, 37 missed, 655 timeout, 173 unviable. Most
  timeouts were not hangs: a 12-minute rebuild test and the 150 s `tests/cli.rs` ran before the killing test in
  cargo's alphabetical binary order, and any surviving mutant ran the whole suite past the limit.
- Follow-up passes used `--iterate` (previous caught and unviable verdicts kept, missed and timed-out mutants rerun)
  at `-j 6` with a 600 s timeout, the last one only over mutants that new tests targeted. The combined verdicts over
  the 1582 mutants that exist at the head of this branch: 1384 caught, 6 missed, 0 timeout, 192 unviable
  (kill rate 99.57%). `evaluator-core` is 56 caught, 0 missed (100.00%). Floors are 100.0 and 99.5.
- The 6 remaining survivors need a live R2U2 checkout (the `Source::Live` arms, `run_group`, and the tool-unavailable
  guards in `src/r2u2_live.rs` and `src/r2u2_future.rs`); each is listed with its reason under `[[survivor]]`, and
  `require_survivor_reasons` is now on.
- 21 mutants are excluded in `.cargo/mutants.toml` as equivalent, each with a one-line reason: `min_*` ties in
  `src/wire/common.rs`; duplicate checks in `src/infinite/mod.rs` (703, 715, 716, 1035, 1036, 1257, 1267) and
  `src/past/mod.rs` (899, 1650, 1654, 1700:21, 1760); `src/infinite/export.rs` 465 (Since is admitted only as
  [0,0]) and 301 (needs more than 2^32 observations); `src/mapping/past.rs` 310.
- Real test gaps found and closed: the past-profile report and history validation had untested refusals
  (`src/past/mod.rs`, including the interval gcd), as did `src/wire/common.rs` identity extraction, hex and sha256
  rendering, accounting and ceilings, and `src/wire/trace.rs`, `src/wire/command.rs`, export, history and mapping
  paths. They are killed by behavior-asserting tests, each wrapped in a thread watchdog where a mutant would loop.
- Disclosures. Mutation runs skip the single test `external_consumer_feature_tree_and_complete_bounded_bytes`
  (`additional_cargo_test_args` in `.cargo/mutants.toml`); it rebuilds the crate for external consumers, checks feature
  gating that no mutant changes, and takes 12 minutes under load. `minimum_test_timeout` is 300 s. Tests written for
  TL-371 are named `a0_*` so they run first and `zz_slow_campaign_survivors.rs` runs last; no existing test was
  removed or weakened. The 24 exclusions and the skipped test are the only places a mutant is not tested.
- `make mutants-diff` is part of `make ci`; it requires every mutant in changed code to be killed.
- Re-check of four reviewed exclusions. `src/infinite/mod.rs:1134` (completion order) and `src/past/mod.rs:1700:53`
  (an identity of exactly the maximum length) were not equivalent: the first changes the work done before the search
  stops, the second refuses a valid correction. Each now has a test and both mutants are caught. `src/infinite/mod.rs:703`
  stays excluded because the clock type has one variant and both document constructors admit only the infinite-trace
  profile; `src/past/mod.rs:1654` stays excluded because each condition it carries also breaks the earlier sum check or
  the depth check; `src/past/mod.rs:1700:21` stays excluded because the identity was already required to equal the
  checked report identity.
- `make spec` was red because `quire coverage --strict` found the FR-022 rows (TC-113 to TC-118) with no backing test. `tests/mutation_gate.rs` now backs them by driving the gate on synthetic outcomes and reading the mutation configuration; it is skipped during mutation runs (`mutation_gate_` in `additional_cargo_test_args`). One divergence is recorded in the matrix: on a failing baseline the gate refuses but still prints the group rates beside `FAIL baseline`, which TC-114 forbids, so TC-114 stays planned.

## Resolution

FND-3712 and FND-3713 of SR-371 are closed. The unmutated suite must pass `baseline_runs` times (default 2) and an
unstable baseline is refused with no kill rate (FR-022-AC-2, TC-114); the real `mutation-thresholds.toml` is checked
to keep `evaluator-core` at or above 95 and `crate` at or above 85 (FR-022-AC-4, TC-116). FR-022 and TC-114 are
implemented in the matrix, so the divergence recorded above no longer applies. The floors in `mutation-thresholds.toml`
(100.0 and 99.5) rest on the TL-371 measured run above (a full run followed by `--iterate` passes), not on a fresh
full run, and FND-2313 describes the earlier sampled floors it replaced.

`kani_gate` runs each Kani harness under a wall-clock budget (`TL_MLTL_KANI_BUDGET_SECS`, default 1800 s), kills the run
at the budget and fails the gate with the harness name and `TIMED OUT` (FR-023-AC-2, TC-120); the timeout is recorded
as its own proof outcome. The budget replaces the earlier caller-set `TL_MLTL_KANI_TIMED_OUT`.
