---
id: TM-005
title: Past C2PO mapping feature test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/FR-038
    type: covers
  - target: ix://agent-ix/tl-mltl/FR-039
    type: covers
---

# Past C2PO mapping feature test matrix

| Test ID | Feature behavior | Status |
|---|---|---|
| TC-160 | H/O/S/Y render with graph, clock, source, and target identities | Implemented in `tests/past_mapping.rs` and `tests/past_c2po_corpus.rs` |
| TC-161 | Guarded T renders through the target's admitted S dual | Implemented in `tests/past_mapping.rs` and `tests/past_c2po_corpus.rs` |
| TC-162 | Unsupported nodes and intervals return typed refusal without artifact | Implemented in `tests/past_mapping.rs` |
| TC-163 | Existing bounded future mapping bytes remain stable | Existing `tests/interop.rs` and `tests/contextual.rs` fixtures, plus `make guarded-ci` |
| TC-164 | Retained per-step target observations compare with source evaluation | Implemented in `tests/past_c2po_corpus.rs` |
| TC-165 | Past operators at positions zero and adjacent positions | Implemented in `tests/past_c2po_corpus.rs` |
| TC-166 | Each admitted origin behavior or guard is pinned to reviewed target evidence | Implemented in `tests/past_mapping.rs` |
| TC-167 | Missing or mismatched origin contract refuses before output | Implemented in `tests/past_mapping.rs` |
| TC-174 | Strict-reader and real mapper fuzz target plus checked seeds | `fuzz/fuzz_targets/c2po_map.rs`; exercised by `tests/c2po_map_fuzz.rs` |
