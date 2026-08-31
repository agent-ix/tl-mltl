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

## Stakeholder Requirement Coverage

| Stakeholder Req | Trace to US/FR | Test/Validation | Coverage Status |
|---|---|---|---|
| StR-001 | FR-001, FR-005 | TC-001, TC-002, TC-015 | ✅ covered |
| StR-002 | FR-002, FR-003, FR-004 | TC-006, TC-009, TC-011 | ✅ covered |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-001 | deterministic and resource-limit tests | TC-003, TC-004, TC-006, TC-013 | ✅ covered |
| NFR-002 | schema tests and evidence inspection | TC-012, TC-014, TC-016 | ✅ covered |

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
| TC-016 | Verify retained evidence completeness and reject false-success classifications | Integration | P0 | FR-005-AC-3, NFR-002-AC-2, NFR-002-AC-3 | ✅ implemented |
