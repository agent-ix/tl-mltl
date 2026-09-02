---
id: TM-001
title: tl-mltl v0.1 test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: covers
---

# tl-mltl v0.1 Test Matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-001 | FR-001-AC-1 through FR-001-AC-3 | TC-001 through TC-004 | ✅ covered |
| FR-002 | FR-002-AC-1 through FR-002-AC-3 | TC-005 through TC-008 | ✅ covered |
| FR-003 | FR-003-AC-1 through FR-003-AC-3 | TC-008 through TC-010 | ✅ covered |
| FR-004 | FR-004-AC-1 through FR-004-AC-3 | TC-011 through TC-013 | ✅ covered |
| FR-005 | FR-005-AC-1 through FR-005-AC-3 | TC-014 through TC-016 | ✅ covered |
| FR-006 | FR-006-AC-1, FR-006-AC-2, FR-006-AC-3, FR-006-AC-5, FR-006-AC-6, FR-006-AC-7 | TC-018, TC-019, TC-020, TC-022, TC-023, TC-024 | ✅ covered |

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-001, FR-005 | TC-001, TC-002, TC-015 | ✅ covered |
| StR-002 | FR-002, FR-003, FR-004, FR-006 | TC-006, TC-009, TC-011, TC-023 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic and resource-limit tests | TC-003, TC-004, TC-006, TC-013 | ✅ covered |
| NFR-002 | schema-negative tests and compiled-test census | TC-012, TC-014, TC-016, TC-017 | ✅ covered |
| NFR-003 | producer-boundary, state-vocabulary, and mutation-probe tests | TC-019, TC-022 | ✅ covered |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-001 | Evaluate Boolean and unary temporal primitives | Unit | P0 | FR-001-AC-1 | ✅ implemented |
| TC-002 | Evaluate Until, Release, nesting, and closed boundaries | Unit | P0 | FR-001-AC-1 | ✅ implemented |
| TC-003 | Preserve result identities and deterministic outcomes | Unit | P0 | FR-001-AC-2, NFR-001-AC-1 | ✅ implemented |
| TC-004 | Reject profile mismatch and work-limit exhaustion | Unit | P0 | FR-001-AC-3, NFR-001-AC-2 | ✅ implemented |
| TC-005 | Match shared-corpus horizon oracles | Integration | P0 | FR-002-AC-1 | ✅ implemented |
| TC-006 | Detect checked arithmetic/resource overflow | Unit | P0 | FR-002-AC-2, NFR-001-AC-2 | ✅ implemented |
| TC-007 | Retain horizon identities and units | Unit | P1 | FR-002-AC-3 | ✅ implemented |
| TC-008 | Match prefix decision deadlines | Unit | P0 | FR-002-AC-1, FR-003-AC-1 | ✅ implemented |
| TC-009 | Preserve pending and early decisive verdicts | Unit | P0 | FR-003-AC-1, FR-003-AC-3 | ✅ implemented |
| TC-010 | Make closed prefixes equal closed evaluation | Unit | P0 | FR-003-AC-2 | ✅ implemented |
| TC-011 | Emit stable supported monitor mapping | Unit | P0 | FR-004-AC-1 | ✅ implemented |
| TC-012 | Reject unsupported adapter inputs | Unit | P0 | FR-004-AC-2, NFR-002-AC-1 | ✅ implemented |
| TC-013 | Verify mapping identities and digests | Unit | P0 | FR-004-AC-3, NFR-001-AC-1 | ✅ implemented |
| TC-014 | Exercise deterministic CLI schemas | Integration | P0 | FR-005-AC-1, NFR-001-AC-1, NFR-002-AC-1 | ✅ implemented |
| TC-015 | Compare supported and non-conclusive differential cases | Integration | P0 | FR-005-AC-2, StR-001-VC-2 | ✅ implemented |
| TC-016 | Verify retained differential inputs and non-conclusive cases are complete | Integration | P0 | FR-005-AC-3, NFR-002-AC-2 | ✅ implemented |
| TC-017 | Bind every requirement-tagged Rust test to a test Cargo compiles and runs, with none ignored or configured out | Integration | P0 | NFR-002-AC-3 | ✅ implemented |
| TC-018 | Classify every shared pin through the packaged compatibility matrix and refuse a mirror reference | Integration | P0 | FR-006-AC-1 | ✅ implemented |
| TC-019 | Reach Quoin without Quoin, Quire, or any gate executing a producer or the external monitor | Integration | P0 | FR-006-AC-2, NFR-003-AC-1, NFR-003-AC-2 | ✅ implemented |
| TC-020 | Bind the sealed record's impact snapshot to a Quire export that names every requirement | Integration | P0 | FR-006-AC-3 | ✅ implemented |
| TC-022 | Demonstrate twelve verification outcomes, each paired with an accepted positive control, and refuse a control naming a scenario that does not exist | Integration | P0 | FR-006-AC-5, NFR-003-AC-3 | ✅ implemented |
| TC-023 | Keep the R2U2 differential a comparison: three classifications, four external states, counts from the corpus manifests, and survival into the retained bytes | Integration | P0 | FR-006-AC-6, StR-002-VC-1, StR-002-VC-2 | ✅ implemented |
| TC-024 | Leave no local evidence framework, no retained legacy evidence, and no live reference to the deleted reader, fixtures, or schemas | Integration | P0 | FR-006-AC-7 | ✅ implemented |
