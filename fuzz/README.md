# TL-216 past C2PO mapping fuzz target

`c2po_map` passes formula-v2 bytes through the strict `tl-syntax` reader and
calls `tl-mltl::map_past_to_c2po` for admitted past-profile graphs. Successful
mappings must preserve the input digest and the digest of their expression.
The target uses the retained reviewed R2U2 4.2 origin identity; it does not
execute C2PO or R2U2. Eight checked-in seeds cover Boolean nesting and the
past operators at origin and interval boundaries.

Run with `cargo fuzz run c2po_map` from this directory. The checked-in
`tests/c2po_map_fuzz.rs` exercises every seed without requiring cargo-fuzz.
