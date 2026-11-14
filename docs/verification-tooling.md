# Verification tooling

How tl-mltl's mutation, coverage and Kani evidence is produced and gated (TL-231, TL-232, TL-234). This page describes the targets and what
the gates enforce; it carries no measured results. Floors live in `mutation-thresholds.toml` and the
`COVERAGE_MIN_LINES` variable of the Makefile, and dated measurements live in `spec/reviews/SR-231-*`
and the Linear tickets.

Tools (inputs): cargo-mutants 27.0.0, cargo-llvm-cov 0.8.7 (branch coverage needs a nightly toolchain with
llvm-tools), Kani 0.68.0.

## Targets

| Target | What it does |
| --- | --- |
| `make mutants` | Full cargo-mutants run (`--no-shuffle`, so the population and order are fixed), then `tl-gate gate` against `mutation-thresholds.toml`. Takes hours; run it weekly. |
| `make mutants-diff` | cargo-mutants on `git diff $(MUTANTS_BASE)...HEAD` (default `origin/main`) only; every mutant in the diff must be killed. Only committed changes are mutated; uncommitted edits are not. |
| `make coverage` | `cargo llvm-cov` line, function and region coverage on the pinned toolchain, failing below `COVERAGE_MIN_LINES`. |
| `make coverage-branch` | The same on nightly, adding branch coverage. |
| `make kani-check` | Every Kani harness; part of `make ci`. |

None of the mutation or coverage targets is part of `make ci`.

## Mutation gate

Kill rate is `caught / (caught + missed + timeout)`: a timeout counts as not killed, and unviable mutants
count on neither side. Each group in `mutation-thresholds.toml` has a floor; the gate prints a table and
exits 1 when a group is below its floor, when a group has no viable mutants, or when the unmutated
baseline failed. The campaign targets are a core group and a whole-crate group (the core is `src/future/*.rs` (evaluation, horizon, interval arithmetic)).

Excluded from every threshold: `src/ci_guard.rs` and `src/bin/ci_guard.rs` (CI plumbing), the process-spawning drivers `src/bin/kani_gate.rs` and `src/bin/fuzz_campaign.rs` (their pure parts are unit-tested; the spawn loops need cargo-kani or a fuzz binary), and the two verifier-only Kani files `src/future/kani_evaluate.rs` and `src/wire/kani_wire.rs`, through `exclude_globs` in `.cargo/mutants.toml` and `exclude` in `mutation-thresholds.toml`. One equivalent mutant is listed in `exclude_re`: the inline `cfg(kani)` harness in `src/future/horizon.rs` is never compiled by `cargo test`, so no test can observe a mutation of it.

`tl-gate mutants` accepts cargo-mutants exit statuses 0, 2 (survivors) and 3 (timeouts) so that
the gate always runs and prints its table; any other status is passed on.
The NFR-006 Makefile guard (`src/ci_guard.rs`) rejects Makefile recipes that join commands with a bare `;`, so the cargo-mutants exit-status handling lives in `tl-gate mutants` and the Makefile only invokes it (help strings must not contain `;` either).

`--source-env` (tl-mltl only) exports `TL_MLTL_SOURCE_REVISION` and `TL_MLTL_SOURCE_STATE`, because `build.rs` reads git and cargo-mutants builds in a copy of the tree without `.git`.

The gate and the runner are one Rust package, `tools/tl-gate` in tl-syntax; this repository builds it through the `tools/gate` shim, whose `Cargo.toml` names the tl-syntax revision. A thresholds file may set `require_survivor_reasons = true` to demand a `[[survivor]]` entry with a reason for every missed or timed-out mutant; this repository's file does not yet.

## Limits of the recorded measurements

Crate-wide kill rates recorded so far are core plus one `--shard 0/3` file sample, not a full run, and the
floors are the sampled rate minus a margin. The first full `make mutants` run should replace them, and floors
only ratchet up. In the recorded run the evaluator core met its mutation target after `tests/mutation_survivors.rs` was added; the crate-wide figure is a file sample, not a full run (see SR-231). Open survivors are listed in SR-231.

## Coverage

Stable and nightly toolchains report different line totals, so `COVERAGE_MIN_LINES` is set below the lower of
the two. It is a floor, not a target. Coverage reports exclude CI-plumbing code and verifier-only code.

## Kani proofs (TL-232)

`make kani-check` first runs `cargo test --test kani_harnesses`, which fails when `scripts/kani-harnesses.txt` and the `#[kani::proof]` functions disagree by name or a harness lacks a stated non-claim or a cover check. It then runs every listed harness through `kani_gate` (`src/bin/kani_gate.rs`) with `cargo kani --lib -Z stubbing --harness NAME --exact`, reporting every failing harness; it fails on any harness that is not verified or has an unsatisfied cover property (Kani itself exits 0 in that case). When `TL_MLTL_EVIDENCE_DIR` is set (a `make campaign-run` of the `kani-horizon-bound` member), `kani_gate` also writes a per-proof record with a distinct outcome state, the Kani and CBMC versions, cover counts, the harness bounds and the command that reproduces the run; the campaign report carries them as `kaniProofs`. Each harness runs under a wall-clock budget (`TL_MLTL_KANI_BUDGET_SECS`, default 1800 s); `kani_gate` kills the run at the budget and fails the gate with `<harness> TIMED OUT`, recording a timeout when evidence is requested. `-Z stubbing` is an unstable
Kani feature used only by the combinator proof. It is part of `make ci`. Tool versions are inputs
(Kani Rust Verifier 0.68.0, CBMC 6.11.0). Each harness doc comment states the bounds it covers and
what it does not claim; `kani::cover!` checks show the paths are reachable, so no proof is vacuous, and
unwinding assertions are on.

| Harness | What is proved |
| --- | --- |
| `horizon_bound_addition_matches_checked_add` (existing) | the bound addition equals `checked_add` for all `u32` bounds and `u64` operands |
| `horizon_temporal_lookahead_is_exact_or_typed_overflow` | `node_lookahead` returns the exact sum or the typed overflow for every temporal operator, full `u32` interval, full `u64` operand values |
| `evaluator_temporal_endpoints_full_range` | the evaluator span computation never overflows, for every `start <= end` over `u32` and every span limit |
| `evaluator_combinators_with_stubbed_at_never_panic` | no-panic and typed errors only, with `Evaluator::at` stubbed: every combinator (and, or, implies, equivalent, future, globally, until, release) returns `Ok` or a typed error and does not panic, for every `u64` verdict time and every interval; wider intervals take the span-limit error. The stub returns an arbitrary verdict, so no verdict is proved correct and `at` itself is not covered; the cover checks show reachability, not correctness |
| `evaluator_dispatch_with_stubbed_operators_never_panics` | no-panic and typed errors only, with every operator method stubbed: the real `at` dispatch, the `Not` recursion and the proposition lookup over a 3-node symbolic formula, traces of 0 to 4 instants over propositions {0, 1}, symbolic time, depth, limits and counters. With the combinator harness this is an assume-guarantee argument that the recursive `at` is panic-free; the induction over depth is the argument, not a Kani result, and the stubs are hand-written over-approximations, not machine-checked contracts |
| `closing_prefix_boolean_operator_lemma`, `closing_prefix_window_operator_lemma`, `closing_prefix_until_release_lemma` | operator-level closing-prefix lemma: given operand verdicts that are stable under appending an instant (a decided verdict is unchanged), each operator's decided verdict is unchanged, for symbolic verdict time and intervals ending at most 2. Typed errors, the `Not` arm and the lookahead-decides statement are not claimed |
| `closing_prefix_atomic_verdict_is_stable` | an open-prefix proposition lookup is decided exactly when `time < len` and a decided answer is unchanged by appending an instant |
| `preflight_never_panics_on_short_input` | the wire lexer `preflight` for ASCII input of 0 to 4 bytes, both byte ceilings, depth ceiling 1 |
| `lexer_never_panics_on_any_short_bytes` | the lexical scan `lex_usage` for every byte value (non-ASCII and invalid UTF-8 included) over 0 to 16 bytes, every depth ceiling up to the owner maximum and every string and field ceiling; the container stack is a fixed-capacity array because a heap `Vec` stack alone exhausted the memory budget |
| `identity_scanners_never_panic_on_short_input` | `scan_string` and `scan_value` for any input of 0 to 16 bytes and any start offset |

Limits of the proofs, recorded in SR-232 with the measurements behind them: a direct whole-formula proof through the real `Evaluator::at` is not proved (replaced by the compositional pair above); wire decode is not proved to the
requested 64 bytes (the lexer to 16, `preflight` as a whole to 4 ASCII bytes), and `serde_json` is not modelled; closing-prefix is proved for the leaf and, as a lemma over verdict tables, for each operator; the whole-formula statement follows only by the induction argument.
`node_lookahead` was extracted from `lookahead` (behavior unchanged) so the arithmetic can be proved.

## Mutation test order

cargo runs test binaries alphabetically and stops at the first failure, so a mutant is detected when the first
binary that can see it runs. The tests written to kill surviving mutants (TL-371) are in files named `a0_*` so they run
first, and the slow campaign tests are in `zz_slow_campaign_survivors.rs` so they run last. Mutation runs skip the single
test `external_consumer_feature_tree_and_complete_bounded_bytes`, a twelve-minute external-consumer rebuild that
checks feature gating, which no mutant changes; every other test runs.
