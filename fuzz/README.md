# C2PO mapping fuzz lane

`c2po_map` passes every admitted byte sequence through the real, bounded
`tl-syntax` formula-v2 strict reader and `tl-mltl::map_past_to_c2po`. It checks
that a successful mapping has exact input and output digests, and lets the
fuzzer catch panics or hangs. The origin contract in this lane is a synthetic
fixture for renderer robustness; it provides no target qualification.

The eight checked-in inputs are digest-pinned in `corpus/c2po_map/SHA256SUMS`.
`tests/c2po_map_fuzz.rs` verifies they all reach the strict reader and the
real mapper before a campaign starts. For a recorded bounded V4 run on a
clean commit, select the installed nightly toolchain and run from the repo
root:

```sh
PYTHONPATH=fuzz python3 -m unittest fuzz.test_run_v4_campaign
python3 fuzz/run_v4_campaign.py --output fuzz/evidence/v4-2026-09-23 \
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
