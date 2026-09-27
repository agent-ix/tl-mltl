---
id: TM-005
title: R2U2 C2PO mapping feature test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/FR-038
    type: covers
  - target: ix://agent-ix/tl-mltl/FR-039
    type: covers
  - target: ix://agent-ix/tl-mltl/FR-040
    type: covers
  - target: ix://agent-ix/tl-mltl/FR-041
    type: covers
---

# R2U2 C2PO mapping feature test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-038 | FR-038-AC-1 through FR-038-AC-3 | TC-160 through TC-164, TC-174 | ✅ covered |
| FR-039 | FR-039-AC-1 through FR-039-AC-2 | TC-165 through TC-167 | ✅ covered |
| FR-040 | FR-040-AC-1 through FR-040-AC-3 | TC-168 through TC-171 | ✅ covered |
| FR-041 | FR-041-AC-1 through FR-041-AC-2 | TC-172, TC-173 | ✅ covered |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-160 | Export O[0,1] and Y from pinned target observations; lower zero-width H/S to equivalent Boolean expressions with preserved identities | Integration | P0 | FR-038-AC-1 | ✅ implemented |
| TC-161 | Lower T[0,0] through its explicit Boolean dual and check the source lowering law | Property | P0 | FR-038-AC-1 | ✅ implemented |
| TC-162 | Refuse unsupported nodes, intervals, target identity, and temporal mixes without an artifact | Integration | P0 | FR-038-AC-2 | ✅ implemented |
| TC-163 | Preserve the exact pre-feature bounded future mapping manifest bytes | Snapshot | P0 | FR-038-AC-2 | ✅ implemented |
| TC-164 | Compare retained per-step target observations with independent past source evaluation | Integration | P0 | FR-038-AC-3 | ✅ implemented |
| TC-165 | Check O/Y origin behavior and zero-width H/S/T source laws at positions zero and adjacent positions | Property | P0 | FR-039-AC-1 | ✅ implemented |
| TC-166 | Bind every admitted nontrivial past target form to reviewed target-origin observations and zero-width forms to Boolean lowering laws | Integration | P0 | FR-039-AC-1 | ✅ implemented |
| TC-167 | Refuse missing or mismatched target origin identity before output | Integration | P0 | FR-039-AC-2 | ✅ implemented |
| TC-168 | Export only `G[0,)ψ` with past-only or bounded-future ψ and retain exact profile/target identities | Integration | P0 | FR-040-AC-1 | ✅ implemented |
| TC-169 | Show target pass and unfinished input never produce `proved` | Property | P0 | FR-040-AC-1 | ✅ implemented |
| TC-170 | Bind the exact exported `O[0,1](p)` safety body to digest-pinned retained C2PO/R2U2 steps and replay target false as a decisive infinite bad prefix | Integration | P0 | FR-040-AC-2 | ✅ implemented |
| TC-171 | Show feature-off build has no infinite export and existing bounded mapping bytes are stable | Integration | P0 | FR-040-AC-3 | ✅ implemented |
| TC-172 | Census every node × interval × context cell with mapped or typed-refused status and no wildcard success | Property | P0 | FR-041-AC-1 | ✅ implemented |
| TC-173 | Partition unbounded liveness/until, fairness, partial valuation, origin, signal, identity and resource refusals with no partial artifact and exact FR-341 projection | Property | P0 | FR-041-AC-2 | ✅ implemented |
| TC-174 | Compile the real `c2po_map` fuzz target and exercise digest-pinned strict-reader/mapper seeds | Fuzz | P1 | FR-038-AC-2 | ✅ implemented |
