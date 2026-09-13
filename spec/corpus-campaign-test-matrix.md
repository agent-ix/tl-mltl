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

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-008 | FR-008-AC-1 through FR-008-AC-5 | TC-037 through TC-043, TC-045, TC-047 | 🚧 planned |
| FR-009 | FR-009-AC-1 through FR-009-AC-6 | TC-044, TC-045, TC-047, TC-048 | 🚧 planned |
| FR-010 | FR-010-AC-1 through FR-010-AC-5 | TC-039, TC-043, TC-046, TC-048 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-004 | deterministic census/replay, mutation, and claim-boundary checks | TC-045 through TC-049 | 🚧 planned |

## Coverage-family allocation

| Family | Positive/boundary | Negative/refusal | Identity/retention | Conditional gate |
|---|---|---|---|---|
| current Boolean/future v1 | TC-039, TC-040 | TC-041 | TC-044, TC-045 | none after M0 |
| W/M canonical lowering | TC-042 | TC-047 | TC-044 | landed #37/#42, parse #32, rewrite #36, evaluator #49/#50 |
| past/history O/H/S/T | TC-043 | TC-041, TC-047 | TC-044 | accepted tl-syntax #38 plus evaluator implementation |
| native predicate/temporal | TC-043, TC-046 | TC-041, TC-046 | TC-044 | accepted and implemented #63/#64 |
| target mapping/observation | TC-046 | TC-046, TC-048 | TC-044, TC-045 | exact target mapping profile |
| generated campaigns | reported by TC-049 | promotion refusal TC-045 | separate population TC-045 | never canonical without promotion |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-037 | Strictly round-trip the closed campaign, dimension, and coverage-cell schemas, exact tuple/set digest preimages, and refuse duplicate, omitted, unknown, reordered, or contradictory entries | Integration | P0 | FR-008-AC-1 | 🚧 planned |
| TC-038 | Enumerate every reviewed cell-obligation tuple exactly once, require every dimension class, pin the ordered set digest, and mutation-test each census status | Property | P0 | FR-008-AC-1, FR-008-AC-2 | 🚧 planned |
| TC-039 | Replay every applicable current-profile semantic cell against stored independent Rust-oracle or reviewed boundary-proof results | Property | P0 | FR-008-AC-2, FR-008-AC-3, FR-008-AC-5, FR-010-AC-4 | 🚧 planned |
| TC-040 | Exercise empty/singleton/short/exact/long and early/late witness/counterexample closed-prefix progress transitions at every interval boundary class | Property | P0 | FR-008-AC-3 | 🚧 planned |
| TC-041 | Refuse malformed, profile-incompatible, missing-history, stale-context, and resource-over-limit cells at their declared stage and reason | Integration | P0 | FR-008-AC-3 | 🚧 planned |
| TC-042 | Compare W/M internal source fixtures with exact canonical graphs and prove no second evaluator branch or semantic-cell population exists | Integration | P0 | FR-008-AC-4 | 🚧 planned |
| TC-043 | Admit past/history and native-bridge cells only after their exact dependencies land, then bind anchor/history/capture/clock and superseding-result states | Integration | P0 | FR-008-AC-4, FR-010-AC-5 | 🚧 planned |
| TC-044 | Strictly round-trip every v1 fixture family; verify one owner plus exact consumer revision, manifest digest, license, provenance, oracle, and limitation fields; and exercise every path and `tl-mltl.corpus-limits/v1` boundary | Integration | P0 | FR-009-AC-1, FR-009-AC-2, FR-009-AC-6 | 🚧 planned |
| TC-045 | Detect canonical mutation/removal/restamping, unsafe/digest-mismatched artifacts, and generated-input promotion without a minimal reproducer, independent oracle, provenance, review, and successor digest | Integration | P0 | FR-008-AC-5, FR-009-AC-1, FR-009-AC-3, FR-009-AC-4, FR-009-AC-5, FR-009-AC-6, NFR-004-AC-2 | 🚧 planned |
| TC-046 | Round-trip every allowed loss/availability/comparison combination, refuse every disallowed combination, preserve each admission refusal and non-conclusive cause, and verify target claims without executing a foreign runtime | Integration | P0 | FR-010-AC-1, FR-010-AC-2, FR-010-AC-3, FR-010-AC-4, FR-010-AC-5, NFR-004-AC-3 | 🚧 planned |
| TC-047 | Mutate profile/operator/status/oracle/denominator identities and prove every incompatible, stale, self-oracled, or silently removed cell turns its owning gate red | Integration | P0 | FR-008-AC-4, FR-009-AC-3, NFR-004-AC-1, NFR-004-AC-2 | 🚧 planned |
| TC-048 | Validate PLAN-006 and its linked GitHub tickets for one owner, explicit consumers, evidence method, predecessors, external resume conditions, and Rust-only executable dependencies; reject a missing/stale route, foreign runtime, or local evidence/retention framework | Integration | P0 | FR-009-AC-2, FR-009-AC-5, FR-010-AC-2 | 🚧 planned |
| TC-049 | Reproduce byte-identical ordered census/replay records and report raw applicable, excluded, blocked, covered, generated, unsupported, unavailable, and non-conclusive populations | Integration | P0 | NFR-004-AC-1, NFR-004-AC-2, NFR-004-AC-3 | 🚧 planned |
