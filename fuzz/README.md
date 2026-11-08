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
| `evaluator_differential` | `arbitrary`-built closed-future or origin-past formula DAG (intervals up to width 8, up to 32 nodes) and a trace of up to 12 positions | production `evaluate_closed_at` / `evaluate_past` equals tl-oracle at every position the oracle decides |
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
