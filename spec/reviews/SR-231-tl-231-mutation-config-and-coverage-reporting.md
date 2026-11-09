---
id: SR-231
title: "spec-correctness - cargo-mutants configuration, kill-rate thresholds and coverage reporting (tl-mltl, TL-231, TL-234)"
type: SpecReview
analysis: spec-correctness
scope: ".cargo/mutants.toml, mutation-thresholds.toml, scripts/, tests/mutation_survivors.rs, Makefile verification targets, campaign/members.json"
review_set: all
---

## Summary

TL-231 and TL-234 add a reproducible cargo-mutants configuration, a kill-rate gate, a diff gate, and
llvm-cov line and branch coverage reporting to tl-mltl. All gates are separate Make targets (`make mutants`,
`make mutants-diff`, `make coverage`, `make coverage-branch`) outside `make ci`, because a full mutation
run costs hours. The thresholds are measured baselines, not the campaign targets of 95% (core) and 85%
(crate): the core kill rate before any change was 49 caught, 8 missed, 25 unviable = 85.96%; after: 56 caught, 0 missed, 25 unviable = 100.00%.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2311 | medium | Measured kill rates (cargo-mutants 27.0.0, `--no-shuffle`, base ccd968e rebased onto 80191d2): core 56 caught, 0 missed, 25 unviable = 100.00%; crate sample 85.65% (179 caught, 30 missed, 44 unviable over the core plus `--shard 0/3`; 172 of the shard's 380 mutants executed before the budget cap). Timeouts count against the rate. The crate rate is a file sample, not a full run: the machine was loaded (load average 35 to 45), so a full run did not fit the budget and sharded sampling was used. The first full `make mutants` run should replace the sample and the floors should then be raised. | `mutation-thresholds.toml` |
| FND-2312 | low | The 8 first-run survivors were four `Display` impls (`EvaluationError`, `ContextualEvaluationError`, `HorizonError`, `ContextualHorizonError`), two work-limit comparisons in `Evaluator::consume`, `validate_trace -> Ok(())`, and one verifier-only mutant under `cfg(kani)` that no test compiles. The first seven are killed by `tests/mutation_survivors.rs`; the eighth is listed in `exclude_re` as equivalent (verifier-only code). The survivors in `src/infinite/mod.rs` seen in the crate sample predate SR-223, which killed 11 of them. | `docs/verification-tooling.md` |
| FND-2313 | low | The enforced floors are evaluator-core 95.0, crate 83.0. A floor below its target means survivors are recorded, not accepted: each is a test gap listed in `docs/verification-tooling.md`. No survivor was classified equivalent by a machine; the only `exclude_re` entries carry a one-line reason. | `.cargo/mutants.toml` |
| FND-2314 | low | Every gate was shown able to fail. `scripts/mutation_gate.py` exits 1 with `FAIL` when a floor exceeds the measured rate (raised floor of 101 against a 100% core; and the pre-fix core run against the 95% floor), exits 1 on an empty population or a failed unmutated baseline, and exits 2 on unreadable input. `make coverage COVERAGE_MIN_LINES=99` fails with a non-zero exit. | `scripts/mutation_gate.py` |
| FND-2315 | low | Coverage (`make coverage-branch`, nightly, all features, `ci_guard` excluded): lines 82.13%, regions 76.47%, functions 73.91%, branches 70.30%. `COVERAGE_MIN_LINES` is set to 80 on the stable `make coverage` target, a floor below the measured line coverage. Branch coverage is unstable in cargo-llvm-cov 0.8.7 and needs nightly. | `Makefile` |
| FND-2316 | low | The Makefile guard (NFR-006) rejected the first version of the recipes (bare `;` in a recipe and in a help string), so the cargo-mutants exit-status handling lives in `scripts/run_mutants.py` and the Makefile only invokes it. | `scripts/run_mutants.py` |

## Execution record

Host: Darwin 25.4.0, arm64 (Apple M5 Pro). Toolchain: rustc 1.98.1, nightly rustc 1.100.0-nightly
(1303417c4 2026-09-21), cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7, quire 0.38.3 (engine 0.52.1). Mutation
order is fixed by `--no-shuffle`; cargo-mutants takes no seed in this mode. Per-mutant timeout is the
cargo-mutants automatic value (three times the unmutated baseline, minimum 60 s). Files: `.cargo/mutants.toml`, `mutation-thresholds.toml`, `scripts/mutation_gate.py`, `scripts/run_mutants.py`, `tests/mutation_survivors.rs`, `campaign/members.json`.
Wall times are not stable on a shared machine and are not equality inputs.
