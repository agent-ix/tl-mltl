# TL fuzz lanes

## TL-229 untrusted-input boundaries

`wire_cli_decode` sends each input through the CLI's actual
`CommandDocument` JSON decode and the strict `ValidatedCommand` owner reader.
`trace_history_intake` sends bytes through the strict trace reader and typed
history decoder, then constructs bounded histories with appended, duplicate,
out-of-order, missing-sample, and over-limit observations. The binaries and
`tests/fuzz_boundaries.rs` call the same target logic. Each target discards
inputs over 64 KiB before parsing; the CLI itself refuses requests over the
published 8 MiB owner input limit before unbounded allocation or JSON decode.

The seven command seeds and nine trace/history seeds are pinned by a
`SHA256SUMS` in each corpus directory. To run a bounded nightly smoke campaign,
copy seed files (excluding `SHA256SUMS`) into separate scratch directories and
run, from the repository root:

```sh
cargo fuzz run --sanitizer address wire_cli_decode SCRATCH_WIRE -- \
  -runs=1000 -seed=229 -max_total_time=30 -max_len=65536
cargo fuzz run --sanitizer address trace_history_intake SCRATCH_HISTORY -- \
  -runs=1000 -seed=230 -max_total_time=30 -max_len=65536
```

Use the pinned `nightly-2026-08-21` toolchain for these commands. Scratch
corpora can grow during fuzzing; the checked-in seeds remain the reviewed
starting corpus. Execution counts, coverage and any findings are evidence for
the V9 Campaign report, not a proof of absence of crashes. Minimize and replay
each crash on the measured source before promoting it to a regression test.

`c2po_map` passes every admitted byte sequence through the real, bounded
`tl-syntax` formula-v2 strict reader and `tl-mltl::map_past_to_c2po`. It checks
that a successful mapping has exact input and output digests, and lets the
fuzzer catch panics or hangs. The origin contract uses the exact retained
R2U2 4.2 observation identity to exercise admitted renderer cells. The fuzz
lane does not execute the external target or add target qualification.

`closed_eval` passes formula-v2 bytes through the same strict syntax reader,
then calls the real closed-trace evaluator at every position of a fixed
three-instant trace with explicit work limits. Its checked corpus has three
admitted formulas and one malformed refusal.

The eight checked-in inputs are digest-pinned in `corpus/c2po_map/SHA256SUMS`.
`tests/c2po_map_fuzz.rs` verifies they all reach the strict reader and the
real mapper before a campaign starts. `tests/closed_eval_fuzz.rs` verifies
that evaluation seeds reach the strict reader and evaluator. For recorded
bounded V4 runs on clean commits, select the installed nightly toolchain and
run from the repo root:

```sh
PYTHONPATH=fuzz python3 -m unittest fuzz.test_run_v4_campaign
python3 fuzz/run_v4_campaign.py --output fuzz/evidence/v4-2026-09-23 \
  --runs 1000 --seed 181 --seconds 30
python3 fuzz/run_v4_eval_campaign.py \
  --output fuzz/evidence/v4-eval-2026-09-23 \
  --runs 1000 --seed 181 --seconds 30
```

The runner copies checked seeds to scratch, uses AddressSanitizer, and retains
lossless gzip engine streams, exact source/tool/lock identities, budget,
observed execution count, stop reason, and artifact digests. A nonzero exit,
short run, or crash artifact receives no clean-run credit. A crash artifact
requires minimization and replay on the measured source revision.

Promote a minimized failure to a reviewed corpus case with its seed, input
bytes, source and target provenance, and expected oracle before updating any
reviewed corpus count. The checked-in seed files come from the pinned syntax
owner past-history corpus and do not expand that denominator.
The earlier 200-execution campaign remains in
`runs/2026-09-22-c2po_map.json`; its source and syntax pins differ from this
V4 campaign. Generated coverage seeds are scratch, not reviewed corpus cases.

## Finite oracle differential lane

`finite_oracle_differential` uses `arbitrary::Unstructured` to construct valid
closed-future or origin-complete-past formula graphs directly, with nested
Boolean and temporal operators, ordered bounded intervals, and nonempty
complete traces. It evaluates each trace position through the production
evaluator and the separate dev-only `tl-oracle`. Every verdict mismatch fails
the target. The checked seed corpus and its SHA-256 manifest are in
`corpus/finite_oracle_differential/`.

`finite_oracle_fault_seed` uses the same generator and comparator with a
deliberately inverted production Until verdict at the origin. It is a
diagnostic target for proving that the comparison detects a seeded evaluator
fault, not a clean-run target. Run it with the same corpus and expect a crash
artifact; minimize and replay that artifact before citing the fault result.

The V9 runner records toolchain, source and corpus digests, requested budget,
observed executions, raw streams, and any crash artifacts. From a clean
reviewed commit with a nightly Rust toolchain selected:

```sh
python3 fuzz/run_v9_finite_campaign.py --output fuzz/evidence/v9-finite-YYYY-MM-DD \
  --runs 1000 --seed 230 --seconds 30
```

A bounded clean run does not prove all finite semantics correct. Any real
disagreement must be minimized, replayed on the same source revision, and
promoted to a reviewed regression fixture before the Campaign V9 report
claims its exit criteria.
