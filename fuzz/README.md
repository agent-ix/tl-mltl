# TL-216 past C2PO mapping fuzz target

`c2po_map` passes formula-v2 bytes through the strict `tl-syntax` reader and
calls `tl-mltl::map_past_to_c2po` for admitted past-profile graphs. Successful
mappings must preserve the input digest and the digest of their expression.
The target uses the retained reviewed R2U2 4.2 origin identity; it does not
execute C2PO or R2U2. Eight checked-in seeds cover Boolean nesting and the
past operators at origin and interval boundaries.

Run with `cargo fuzz run c2po_map` from this directory. The checked-in
`tests/c2po_map_fuzz.rs` exercises every seed without requiring cargo-fuzz.

## TL-246 Campaign V4 targets

The Campaign runs five separate mlTL targets. `closed_eval` admits strict
closed-trace formulas and checks a fixed finite trace at every position.
`wire_cli_decode` exercises the command decoder and strict owner reader.
`trace_history_intake` exercises trace and history byte intake plus bounded
append and correction constructors. `finite_oracle_differential` generates
bounded formulas and compares production verdicts with the independently
pinned `tl-oracle` implementation. `c2po_map` retains its past C2PO mapping
boundary above. A clean 1,000-execution fuzz result is evidence of that
bounded run only.

Each target has a checked-in corpus with `SHA256SUMS`. Root smoke tests
`closed_eval_fuzz` and `fuzz_boundaries`, and the fuzz workspace
`finite_oracle` test, execute the admitted seeds and negative controls without
requiring a sanitizer. Run the sealed Campaign procedure for source-pinned
address-sanitized V4 evidence.
