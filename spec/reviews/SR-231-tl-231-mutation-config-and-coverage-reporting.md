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

## TL-371 status (unfinished, 2026-11-12)

This section records an interrupted measurement; no floor has been changed from it. Floors in
`mutation-thresholds.toml` are still the pre-TL-371 sampled values.

- First full run (cargo-mutants 27.0.0, `--no-shuffle`, `-j 4`, 782 s timeout, 1465 mutants): 600 caught, 37 missed,
  655 timeout, 173 unviable. Most timeouts were not hangs: the 12-minute `tests/tc138_feature_boundary.rs` rebuild and
  the 150 s `tests/cli.rs` run before the killing test in cargo's alphabetical binary order, and a surviving mutant runs the
  whole suite past the limit.
- Disclosure: mutation runs skip the single test `external_consumer_feature_tree_and_complete_bounded_bytes`
  (`additional_cargo_test_args` in `.cargo/mutants.toml`; it checks feature gating, which no mutant changes), and
  `minimum_test_timeout` is 300 s. Test binaries written for TL-371 are named `a0_*` so they run first, and the
  slow campaign tests are `zz_slow_campaign_survivors.rs` so they run last; no test was removed or weakened.
- Retests with `--iterate` (kept caught and unviable verdicts; reran missed and timed-out mutants):
  the last pass (`-j 6`, 600 s timeout, 219 mutants) was stopped unfinished with       60 caught,      153 missed,        6 timeout and
         0 unviable recorded in `target/mutants/mutants.out`. It ran against a tree that predates the export, history,
  mapping and replay tests, so its misses in those files are expected to be killed.
- Equivalent by analysis, to be excluded with reasons once confirmed: `src/infinite/mod.rs` 703, 715, 716, 1035,
  1036, 1134; `src/past/mod.rs` 899:41, 1650, 1654, 1700, 1760; `src/infinite/export.rs` 301 and 465:40;
  `src/mapping/past.rs` 310. Reachable only with a live R2U2: `Source::Live` arms in `src/r2u2_live.rs` 742 and
  `src/r2u2_future.rs` 275, `run_group` (`src/r2u2_live.rs` 843), and the `run_future` guards at 274 and 298.
