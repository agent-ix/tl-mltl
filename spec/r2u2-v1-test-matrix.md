---
id: TM-005
title: MLTL V1 R2U2 mapping test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: covers
---

# MLTL V1 R2U2 mapping test matrix

TL-211 rows remain planned until TL-215 accepts the combined V1 spec.

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-038 | FR-038-AC-1 through FR-038-AC-3 | TC-160 through TC-164 | 🚧 planned |
| FR-039 | FR-039-AC-1 through FR-039-AC-2 | TC-165 through TC-167 | 🚧 planned |
| FR-040 | FR-040-AC-1 through FR-040-AC-3 | TC-168 through TC-171 | 🚧 planned |
| FR-041 | FR-041-AC-1 through FR-041-AC-2 | TC-172, TC-173 | 🚧 planned |
| FR-042 | FR-042-AC-1 through FR-042-AC-2 | TC-164, TC-174 | 🚧 planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-160 | Render H/O/S/Y from validated past graphs through parsed C2PO expressions and preserve identities | Integration | P0 | FR-038-AC-1 | 🚧 planned |
| TC-161 | Render T only through a verified direct or dual target form under the exact origin contract | Integration | P0 | FR-038-AC-1 | 🚧 planned |
| TC-162 | Refuse unsupported past node and interval forms without any expression or manifest | Integration | P0 | FR-038-AC-2 | 🚧 planned |
| TC-163 | Keep bounded future v1/v2 manifest and expression bytes stable after past mapping is added | Snapshot | P0 | FR-038-AC-2 | 🚧 planned |
| TC-164 | Replay pinned past corpus expectations per step against independent source evaluation and classified target observations | Integration | P0 | FR-038-AC-3, FR-042-AC-1 | 🚧 planned |
| TC-165 | Exercise H/O/S/Y/T at zero and adjacent positions under the source false-before-origin rule | Property | P0 | FR-039-AC-1 | 🚧 planned |
| TC-166 | Show every admitted C2PO past form has equivalent target origin behavior or an explicit guard | Integration | P0 | FR-039-AC-1 | 🚧 planned |
| TC-167 | Refuse a missing or mismatched target origin contract before writing output | Integration | P0 | FR-039-AC-2 | 🚧 planned |
| TC-168 | Export only `G[0,)ψ` with past-only or bounded-future ψ and retain exact profile/target identities | Integration | P0 | FR-040-AC-1 | 🚧 planned |
| TC-169 | Show target pass and unfinished input never produce `proved` | Property | P0 | FR-040-AC-1 | 🚧 planned |
| TC-170 | Replay each target violation as a decisive bad prefix under infinite semantics | Integration | P0 | FR-040-AC-2 | 🚧 planned |
| TC-171 | Show feature-off build has no infinite export and existing bounded mapping bytes are stable | Integration | P0 | FR-040-AC-3 | 🚧 planned |
| TC-172 | Census every node × interval × context cell with mapped or typed-refused status and no wildcard success | Property | P0 | FR-041-AC-1 | 🚧 planned |
| TC-173 | Partition unbounded liveness/until, fairness, partial valuation, origin, signal, identity and resource refusals with no partial artifact and exact FR-341 projection | Property | P0 | FR-041-AC-2 | 🚧 planned |
| TC-174 | Verify corpus digest/provenance and drive the strict reader plus real c2po_map entry point under recorded fuzz seed and budget | Fuzz | P0 | FR-042-AC-1, FR-042-AC-2 | 🚧 planned |
