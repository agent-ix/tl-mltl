---
id: SR-223
title: "spec-correctness and spec-fuzz - infinite-trace laws, fuzz execution and mutation pilot (TL-222, TL-223)"
type: SpecReview
analysis: spec-correctness
scope: "tests/infinite_laws.rs, tests/infinite_reference_corpus.rs, tests/infinite_prefix_limits.rs, tests/infinite_fuzz_seeds.rs, fuzz/fuzz_targets/lasso_differential.rs, fuzz/fuzz_targets/partial_valuation_monotone.rs, src/infinite"
review_set: all
---

## Summary

TL-223 adds two cargo-fuzz targets for the infinite-trace provider, records a
180-second execution of each, and records a deterministic cargo-mutants pilot
over the provider's operator semantics and prefix route. Both fuzz targets
ran more than two million executions with no crash, leak, timeout or
divergence. The mutation pilot generated 142 mutants: 10 unviable, 129 caught
and 3 missed. The 3 survivors are dispositioned below as equivalent or
unreachable; no survivor is an untested semantic difference.

The pilot first missed 14 mutants, all in the finite-prefix safety route
(`evaluate_prefix_safety_at`) and one period guard in `periodic::evaluate`.
Eleven were killed by new tests in `tests/infinite_prefix_limits.rs` and a unit
test in `src/infinite/mod.rs`; one of those kills exposed that the prefix route
had no test for sound completion enumeration (a mutant that dropped the
all-false completion made the route refute a satisfiable safety claim).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-2231 | low | Both fuzz targets exceeded the 1,000,000-execution exit with no divergence, crash or leak artifact. Execution used eight libFuzzer fork workers inside the 180-second cap and no sanitizer, so a panic or assertion is the only failure signal; memory-safety instrumentation was not part of this run. | FR-021, FR-030-AC-1, FR-030-AC-2 |
| FND-2232 | low | Every property law in `tests/infinite_laws.rs` and both fuzz targets were shown able to fail: 14 temporary provider mutants (table below) each fail at least one law, and three of them crash a fuzz target within 60 seconds. | FR-031-AC-1, FR-032-AC-2, FR-033-AC-2 |
| FND-2233 | medium | Mutation pilot: 142 mutants (periodic.rs in full; `evaluate_trace`, `evaluate_prefix_safety_at`, `finite_horizons` in mod.rs), 129 caught, 3 missed, 10 unviable, 0 timeouts. Score 129 / 132 = 97.7% of viable mutants. This is a pilot of one module under one test selection, not a campaign result. | FR-022, `src/infinite/periodic.rs`, `src/infinite/mod.rs` |
| FND-2234 | low | Survivor dispositions: `mod.rs:1035:13` and `mod.rs:1036:13` (`||` to `&&`) sit in an observation-shape check that duplicates `validate_shape()`, which runs first and returns the same `IdentityMismatch`; the guard is unreachable. `mod.rs:1134:65` (`!=` to `==`) complements every completion bit, so the same 2^n completions are enumerated in a different order; only `evaluation_steps` can differ. | FR-033-AC-2, FR-034-AC-1 |
| FND-2235 | medium | Spec question, no code change made. FR-001 puts the left operand of `U[a,b]` on `[a,i)`; FR-031 puts it on every offset from zero. For a lower bound above zero the finite closed evaluator and the infinite provider disagree on the same word. The bounded-embedding law is restricted to lower bound zero for `U` and `R`, and `until_guard_range_differs_between_the_finite_and_infinite_profiles` pins the divergence. | FR-001-AC-1, FR-031-AC-1, FR-031-AC-3 |
| FND-2236 | low | No shared cargo-mutants configuration existed on `main` when the pilot ran (TL-231 had no open PR), so the pilot uses explicit command-line scoping, recorded below, and adds no configuration file. It runs the release profile with thin LTO off, so integer overflow wraps rather than panics; mutants that only differ by overflow are not distinguished. It measures the whole infinite-trace test selection; the share of kills owed only to the TL-222/TL-223 tests was not measured. | FR-022 |
| FND-2237 | low | tl-mltl previously pinned `tl-oracle` at `9bf3994`, a commit reachable only from an unmerged branch (syntax 0.4.0 with serde, plus an unused `v11.rs` census). `tl-oracle` `main` now carries the syntax 0.4.0 dependency (agent-ix/tl-oracle#11) and `tl_oracle::fixpoint` (agent-ix/tl-oracle#10, no `v11`), and tl-mltl pins `main` commit `3e6743a`. | NFR-005 |

## Law and fuzz sensitivity

Each row is a temporary edit to `src/infinite`, reverted after the run, with
the laws run under their recorded seeds. "Killed by" lists the tests that
failed.

| Mutant | Edit | Killed by |
| --- | --- | --- |
| A | `future_end` drops the loop period | duality, rewriting, provider-vs-reference, `control_no_liveness_checker_sees_a_proved_result`; fuzz `lasso_differential` (crash) |
| B | `aligned` uses the smaller entry | duality, rewriting, provider-vs-reference |
| C | unbounded past entry drops the period | duality, rewriting, provider-vs-reference |
| D | fairness read at the first loop window only | provider-vs-reference |
| E | until guard starts at the lower bound | duality, rewriting, provider-vs-reference, `until_guard_range_differs_between_the_finite_and_infinite_profiles` |
| F | prefix refutation accepts an undecided horizon | prefix-refutation soundness |
| G | unsettled prefix reported proved | no-liveness-from-prefix |
| H | conflicting reported as missing | missing-vs-conflicting, provider-vs-reference |
| I | proved when any admitted completion is true | embedding, prefix-refutation, duality control, rewriting, provider-vs-reference |
| J | fairness filter admits everything | provider-vs-reference, `control_weakening_is_not_vacuous`; fuzz `lasso_differential` (crash) |
| K | release not dual | embedding, prefix-refutation, duality, rewriting, provider-vs-reference |
| L | once/historically existential flipped | duality, rewriting, provider-vs-reference |
| N | missing cells take one fixed completion | partial monotonicity, provider-vs-reference; fuzz `partial_valuation_monotone` (crash) |
| O | parity fairness filter (non-monotone) | fairness weakening, provider-vs-reference |

## Fuzz execution record

Source: branch `tl-infinite-fuzz-mutation`, fuzz targets and seeds at revision
`77914ba`. The campaign ran on the same sources as `d22aff6`, before a rebase
onto `main`; later commits change only tests and a `#[cfg(test)]` unit test.
Host: Darwin 25.4.0, arm64, 2026-11-08. Toolchain: rustc
`1.100.0-nightly (1303417c4 2026-09-21)`, cargo-fuzz 0.13.1, libFuzzer via
libfuzzer-sys 0.4.13, quire 0.38.3 (engine 0.52.1), quoin 0.29.4. Reference
procedures: `tl-oracle` rev `3e6743a` (`main`; the runs predate the repin and used the same `fixpoint` source on an unmerged branch).

Each target ran `cargo +nightly fuzz run -s none <target> <scratch-corpus>
corpus/<target> -- -seed=20261108 -max_total_time=180 -fork=8`. The 12
committed seeds per target (six hand-built shapes, six from a fixed PRNG) were
the starting corpus; generated inputs went to an empty scratch directory.

| Target | Seed | Wall time | Executions | Final cov / features | Generated corpus | Outcome |
| --- | ---: | ---: | ---: | --- | ---: | --- |
| `lasso_differential` | 20261108 | 184 s | 2,440,918 | 2594 / 11095 | 965 units | pass, exit 0, no artifact |
| `partial_valuation_monotone` | 20261108 | 184 s | 2,662,467 | 2290 / 9468 | 882 units | pass, exit 0, no artifact |

Fault controls (60-second runs, seed 20261108, mutant reverted afterwards):
mutant A crashed `lasso_differential` with `disposition Failed != Proved` on
`F[0,) p0`; mutant J crashed it with `Inconclusive != Proved` on a
fairness-bearing case; mutant N crashed `partial_valuation_monotone` with
`Refuted != Inconclusive` on a refined lasso.

Coverage and feature counts are observations from one run and do not show a
plateau. Generated corpora are scratch output and are not committed. No crash
occurred, so no regression test was added from a crash.

## Mutation pilot record

Tool: cargo-mutants 27.0.0 on rustc 1.98.1, `cargo test` run in release
profile with `CARGO_PROFILE_RELEASE_LTO=off`,
`CARGO_PROFILE_RELEASE_CODEGEN_UNITS=16` and incremental builds to bound
time. The build script needs `TL_MLTL_SOURCE_REVISION` and
`TL_MLTL_SOURCE_STATE=clean` because cargo-mutants builds in a copy outside
the git tree. Order is fixed (`--no-shuffle`), four jobs, 300 s per-mutant
timeout.

```
cargo mutants --features infinite-trace --cargo-arg=--release -j4 --no-shuffle \
  --timeout 300 -f src/infinite/periodic.rs -f src/infinite/mod.rs \
  -F 'periodic\.rs|evaluate_trace|evaluate_prefix_safety_at|finite_horizons' \
  -- --lib --test infinite_oracle --test infinite_trace --test infinite_matrix_rows \
  --test infinite_corpus --test infinite_export --test tc149_semantic_laws \
  --test infinite_laws --test infinite_reference_corpus --test infinite_fuzz_seeds
```

| Function | Caught | Missed | Unviable |
| --- | ---: | ---: | ---: |
| `evaluate_trace` | 21 | 0 | 1 |
| `evaluate_prefix_safety_at` | 18 (first pass) + 10 killed | 3 | 1 |
| `finite_horizons` | 4 | 0 | 2 |
| `periodic::evaluate` | 28 (first pass) + 1 killed | 0 | 4 |
| `Word::at`, `Budget::{step,word}`, `aligned`, `child`, `future_*`, `past_*`, `upper` | 47 | 0 | 2 |

First pass: 118 caught, 14 missed, 10 unviable. The 14 survivors were
re-run after the kill tests with `-F` selecting only their locations (35
mutants in the second run, 8 in the third, which added
`--test infinite_prefix_limits`); 11 were killed and 3 remain with the
dispositions in FND-2234.

Kill tests: `every_prefix_work_limit_is_inclusive_and_independent` (node,
position and cell bounds each inclusive and independent, and the position
pre-check runs before the safety-fragment check),
`a_prefix_is_refuted_only_if_no_completion_satisfies_the_body` (every
completion enumerated, counted as four for two unknown cells),
`each_prefix_identity_axis_refuses_on_its_own`, and the unit test
`a_valuation_set_without_a_loop_is_an_invalid_lasso`.

## Run report

| Route | Count |
| --- | ---: |
| Property laws with recorded seeds | 9 |
| Property laws shown able to fail by mutation | 9 |
| Fuzz targets executed | 2 |
| Fuzz executions | 5,103,385 |
| Crashes turned into regression tests | 0 |
| Mutants generated / caught / missed / unviable | 142 / 129 / 3 / 10 |
| Missed mutants with a written disposition | 3 |
