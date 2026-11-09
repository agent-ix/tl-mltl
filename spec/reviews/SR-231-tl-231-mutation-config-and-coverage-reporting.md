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
