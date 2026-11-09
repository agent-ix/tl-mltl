---
id: TM-002
title: "MLTL corpus and interoperability campaign test matrix"
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: covers
---

# MLTL corpus and interoperability campaign test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-008 | FR-008-AC-1 through FR-008-AC-5 | TC-092 through TC-097, TC-099, TC-101 | 🚧 planned |
| FR-009 | FR-009-AC-1 through FR-009-AC-5 (FR-009-AC-6 withdrawn) | TC-098, TC-099, TC-101 | 🚧 planned |
| FR-010 | FR-010-AC-1 through FR-010-AC-4 | TC-093, TC-100, TC-101 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-004 | deterministic replay, seeded faults, and report checks | TC-099 through TC-101, TC-103 | 🚧 planned |

## Coverage-family allocation

| Family | Positive/boundary | Negative/refusal | Provenance |
|---|---|---|---|
| closed-trace and online-prefix | TC-092, TC-093, TC-094 | TC-095 | TC-098, TC-099 |
| past-history | TC-097 | TC-095 | TC-098 |
| derived W/M | verified by FR-016 (TC-076 through TC-080) | TC-096 | TC-098 |
| target mapping and observation | TC-100 | TC-100 | TC-098, TC-099 |
| generated populations | reported by TC-103 | promotion refusal TC-099 | TC-099 |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-091 | Round-trip the campaign, dimension, and coverage-cell schemas under exact tuple and set digest preimages | Integration | P0 | FR-008-AC-1 | ⛔ retired — schema and digest ceremony around the cell list; the cell list is plain data checked by TC-092 |
| TC-092 | Check that every profile, operator, and interval-shape cell has a positive, a negative, and a boundary case, that every named edge condition is exercised, and that a missing case fails naming the cell | Integration | P0 | FR-008-AC-1, FR-008-AC-2 | 🚧 planned |
| TC-093 | Replay every stored expected result against the independent oracle and production, and check mapping output against canonical semantics | Property | P0 | FR-008-AC-2, FR-008-AC-3, FR-008-AC-5, FR-010-AC-4 | 🚧 planned |
| TC-094 | Exercise empty/singleton/short/exact/long and early/late witness/counterexample closed-prefix progress transitions at every interval boundary class | Property | P0 | FR-008-AC-3 | ✅ implemented |
| TC-095 | Refuse malformed, profile-incompatible, missing-history, stale-context, and resource-over-limit cases with their stored typed refusal | Integration | P0 | FR-008-AC-3 | 🚧 planned |
| TC-096 | Check that derived W/M operators add no evaluator branch or case population and that their verdicts equal the lowered primitive graph | Integration | P0 | FR-008-AC-4 | 🚧 planned |
| TC-097 | Check that every past operator has positive, negative, and boundary cases under the origin-complete-history profile, confirmed by the oracle | Integration | P0 | FR-008-AC-4 | 🚧 planned |
| TC-098 | Replay each fixture family from its owner, verify its recorded identity and provenance, and refuse a record with a missing or unknown field | Integration | P0 | FR-009-AC-1, FR-009-AC-2 | 🚧 planned |
| TC-099 | Detect a changed expected result, removed case, and a generated input presented as canonical, and accept a generated disagreement only as a minimised oracle-confirmed reproducer | Integration | P0 | FR-008-AC-5, FR-009-AC-1, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, NFR-004-AC-2 | 🚧 planned |
| TC-100 | Report agreement, disagreement, or not-comparable with a reason for every comparison, refuse mismatched observations before comparison, bound claims to the retained cases, and replay without executing a foreign runtime | Integration | P0 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4, NFR-004-AC-3 | 🚧 planned |
| TC-101 | Apply seeded evaluator and stored-result faults and prove each turns its owning check red | Integration | P0 | FR-008-AC-5, FR-009-AC-3, FR-010-AC-1, NFR-004-AC-1, NFR-004-AC-2 | 🚧 planned |
| TC-102 | Validate the PLAN-007 routing bundle and its linked tickets for owners, consumers, predecessors, and resume conditions | Integration | P0 | FR-009-AC-2 | ⛔ retired — obsolete planning gate; ownership is stated in FR-009 and ticket routing lives in the tracker |
| TC-103 | Reproduce the campaign report at the same commit and report admitted, refused, unsupported, unavailable, and not-comparable counts separately | Integration | P0 | NFR-004-AC-1, NFR-004-AC-2, NFR-004-AC-3 | 🚧 planned |
