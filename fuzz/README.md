# C2PO mapping fuzz lane

`c2po_map` passes every admitted byte sequence through the real, bounded
`tl-syntax` formula-v2 strict reader and `tl-mltl::map_past_to_c2po`. It checks
that a successful mapping has exact input and output digests, and lets the
fuzzer catch panics or hangs. The origin contract in this lane is a synthetic
fixture for renderer robustness; it provides no target qualification.

Run from this directory with Rust 1.98.1 and an installed nightly toolchain:

```sh
TMPDIR=/private/tmp CARGO_NET_OFFLINE=true rustup run nightly cargo fuzz run c2po_map -- -seed=174 -max_total_time=60 -timeout=5
```

Promote a minimized failure to a reviewed corpus case with its seed, input
bytes, source and target provenance, and expected oracle before updating any
reviewed corpus count. The checked-in seed files come from the pinned syntax
owner past-history corpus and do not expand that denominator.
The first bounded campaign is recorded in `runs/2026-09-22-c2po_map.json`;
generated coverage seeds were discarded.
