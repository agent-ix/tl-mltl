# TL-216 past C2PO mapping fuzz target

`c2po_map` passes formula-v2 bytes through the strict `tl-syntax` reader and
calls `tl-mltl::map_past_to_c2po` for admitted past-profile graphs. Successful
mappings must preserve the input digest and the digest of their expression.
The target uses the retained reviewed R2U2 4.2 origin identity; it does not
execute C2PO or R2U2. Eight checked-in seeds cover Boolean nesting and the
past operators at origin and interval boundaries.

Run with `cargo fuzz run c2po_map` from this directory. The checked-in
`tests/c2po_map_fuzz.rs` exercises every seed without requiring cargo-fuzz.

# TL-228..230 robustness and differential fuzz targets

| Target | Input | Check |
| --- | --- | --- |
| `evaluator_differential` | `arbitrary`-built closed-future or origin-past formula DAG (interval endpoints up to 8, up to 32 nodes) and a trace of up to 12 positions | production `evaluate_closed_at` / `evaluate_past` equals tl-oracle at every position the oracle decides |
| `wire_cli_decode` | arbitrary bytes (at most 16 KiB) | the CLI request path (parse, validate formula, analyze/evaluate) never panics; `ValidatedCommand` and `ValidatedTrace` readers return a typed error or a value whose canonical bytes are the input and which re-reads to itself, under owner maxima and tight limits |
| `history_intake` | arbitrary bytes read as `position-history/v1`, plus a structured builder of out-of-order, duplicated, sparse, huge-position observations | construction and `corrected` revisions return a typed error or a value that `derive`/`read` round-trips under owner and tight limits, and that `evaluate_past` accepts or refuses at any anchor |

Run any target from this directory with `cargo +nightly fuzz run <target>
fuzz/corpus/<target> -- -seed=<n> -max_total_time=<s> -rss_limit_mb=8192`. The
RSS limit matters on macOS: libFuzzer counts the address sanitizer's freed
memory toward RSS, so the default 2048 MB aborts runs with a small live heap.
`make fuzz-check` builds every target and audits the fuzz crate;
`make fuzz-smoke` replays each checked-in corpus once. The oracle crate pins its
own `tl-syntax`, so proposition identities cross to it as integers.
`tests/fuzz_seed_replay.rs` checks that the seeds reach accepting and refusing
paths. Run records are in `spec/reviews/` and `spec/evidence/`.

# TL-223 infinite-trace fuzz targets

`lasso_differential` and `partial_valuation_monotone` decode any byte string
into a formula over the whole infinite profile, up to two fairness premises, a
partial lasso (at most eight missing or conflicting values) and a position. The
bodies live in `support/infinite.rs`, so `tests/infinite_fuzz_seeds.rs` runs
every committed seed under `cargo test` without cargo-fuzz.

* `lasso_differential`: the provider must agree with the `tl-oracle`
  successor-fixpoint procedure on disposition, admitted completions, witness
  flags and reason, and the two `tl-oracle` procedures must agree with each
  other.
* `partial_valuation_monotone`: a conclusive verdict on a partial lasso must
  survive refining its missing values (unless fairness then admits nothing),
  and the refined result must match the reference.

Run with `cargo +nightly fuzz run <target> <scratch-corpus> corpus/<target> --
-seed=<n> -max_total_time=180`, passing a scratch directory first so generated
inputs do not land in the committed seeds. The 12 seeds per target are six
hand-built shapes and six from a fixed PRNG. The campaign record is
`spec/reviews/SR-223-infinite-fuzz-and-mutation-pilot.md`.

# TL-29 fuzz baseline (FR-021)

`cargo run --release --bin fuzz_campaign -- OUT_DIR` (run from the repository root after
`rustup run nightly cargo fuzz build`) runs every target under seeds 101, 102 and 103 with a 900 s
budget and a trailing 300 s no-new-feature plateau stop, each run from an empty scratch corpus plus
the checked-in seeds as read-only input. It writes `OUT_DIR/record.json` in the
`tl-mltl.fuzz-run-record/v1` format that `src/fuzz_record.rs` reads back strictly, plus the libFuzzer
logs and any crash inputs. The plateau rule is `fuzz_record::decide_stop`. Progress is sampled on a
one-second clock as well as on each libFuzzer progress line, so a target that prints rarely still
reaches the plateau decision. Timeouts and out-of-memory are recorded by state only; their inputs stay
under `OUT_DIR/crashes` and are not replayed by process (not done).

The 2026-11-08 record, written by the Python driver this one replaces (format: see the file), is `fuzz/evidence/2026-11-08/campaign-900s.json` (18
runs, no crash, timeout or out-of-memory). No run reached a plateau; every run
ended at the budget. For the five targets with regular progress output, features
were still appearing within the last window (growth observed). For
`evaluator_differential` the growth label and last-new-feature time are
unverified (`budget_growth_unverified`, null): its low execution rate left
libFuzzer's block-buffered stderr empty until flush. The driver checks the
plateau condition only when a stderr line arrives, so a quiet stretch cannot
trigger it; that applies to any low-output target.

The grown corpora are generated inputs and are not committed; they were kept
locally, not retained, and re-running the driver with the recorded seeds and
budget reproduces them. A no-crash result is a bounded observation, not a
correctness claim.
