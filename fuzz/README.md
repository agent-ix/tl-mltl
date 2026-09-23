# Past mapping and closed evaluation fuzz lanes

`c2po_map` passes every admitted byte sequence through the real, bounded
`tl-syntax` formula-v2 strict reader and `tl-mltl::map_past_to_c2po`. It checks
that a successful mapping has exact input and output digests, and lets the
fuzzer catch panics or hangs. The origin contract in this lane is a synthetic
fixture for renderer robustness; it provides no target qualification.

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
