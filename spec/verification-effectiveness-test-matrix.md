---
id: TM-003
title: "MLTL verification-effectiveness campaign test matrix"
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: covers
---

# MLTL verification-effectiveness campaign test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Coverage Status |
|---|---|---|---|
| FR-011 | FR-011-AC-1 through FR-011-AC-5 | TC-050 through TC-054 | 🚧 planned |
| FR-012 | FR-012-AC-1 through FR-012-AC-5 | TC-055 through TC-058, TC-067, TC-068 | 🚧 planned |
| FR-013 | FR-013-AC-1 through FR-013-AC-5 | TC-059 through TC-064 | 🚧 planned |
| FR-014 | FR-014-AC-1 through FR-014-AC-5 | TC-065 through TC-067, TC-069, TC-070 | 🚧 planned |
| FR-015 | FR-015-AC-1 through FR-015-AC-6 | TC-067, TC-068, TC-071 through TC-075 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-005 | deterministic census/replay, fault seeding, mutation, and claim-boundary checks | TC-053, TC-060, TC-064, TC-066 through TC-068, TC-071, TC-073 through TC-075 | 🚧 planned |

## Cross-repository allocation and gates

| Repository | Owned campaign surface | Admission gate |
|---|---|---|
| tl-syntax | formula/profile/schema property and mutation work under #26 | accepted MRS-003 plus landed applicable syntax profiles |
| tl-parse | parse/format/span properties and input-boundary fuzz work under #25 | accepted MRS-003 plus exact parser dialect/profile |
| tl-rewrite | equivalence/counterexample properties and reviewed fuzz decision under #27 | accepted MRS-003 plus exact evaluator oracle/profile |
| tl-mltl | evaluator/horizon/history/mapping properties, mutation, and bounded Kani under #31 | accepted MRS-003 plus landed MRS-002 applicable cells |
| shared assurance | Quire static facts, Quoin validation/retention/receipts, Engineering Assurance compatibility | released compatible shared contracts; no local substitute |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-050 | Census every exact Quire criterion once and reject omitted, duplicate, stale, or unknown ledger rows | Integration | P0 | FR-011-AC-1 | 🚧 planned |
| TC-051 | Round-trip applicable, excluded, and blocked property rows with complete finite domain and ownership fields | Property | P0 | FR-011-AC-1, FR-011-AC-2 | 🚧 planned |
| TC-052 | Reject self-oracles, shared derivations, vacuous partitions, and seeded faults missed by the claimed oracle | Integration | P0 | FR-011-AC-2, FR-011-AC-3 | 🚧 planned |
| TC-053 | Reproduce generated and exhaustive property runs, including seed, accepted/discarded classes, failures, and shrinking | Property | P0 | FR-011-AC-3, FR-011-AC-4, NFR-005-AC-1 | 🚧 planned |
| TC-054 | Keep generated inputs outside canonical corpus and refuse incomplete promotion | Integration | P0 | FR-011-AC-5 | 🚧 planned |
| TC-055 | Enforce three RNG seeds, both finite caps, first-stop semantics, and requested-versus-observed budgets | Fuzz | P0 | FR-012-AC-1, FR-012-AC-4 | 🚧 planned |
| TC-056 | Compute plateau only from exact feature-identity sets over the complete final target-time-and-execution window under unchanged instrumentation | Fuzz | P0 | FR-012-AC-1, FR-012-AC-2 | 🚧 planned |
| TC-057 | Preserve fuzz growth, plateau, crash, timeout, resource, cancellation, tool, instrumentation, and not-run states | Integration | P0 | FR-012-AC-2, FR-012-AC-3 | 🚧 planned |
| TC-058 | Retain and independently replay exact fuzz corpora and crashes, refusing missing or stale identities | Integration | P0 | FR-012-AC-3, FR-012-AC-4 | 🚧 planned |
| TC-059 | Deterministically discover and identity-bind every mutant before exclusion and selection | Integration | P0 | FR-013-AC-1 | 🚧 planned |
| TC-060 | Reproduce selected/unselected/excluded populations and refuse failed, inconsistent, drifting, or partial controls | Integration | P0 | FR-013-AC-1, FR-013-AC-2, NFR-005-AC-1 | 🚧 planned |
| TC-061 | Isolate each mutant, interleave controls, enforce timeout and clean restoration, and keep every execution state distinct | Integration | P0 | FR-013-AC-2, FR-013-AC-3 | 🚧 planned |
| TC-062 | Emit the shared score only for a complete nonzero viable denominator and retain every raw count | Integration | P0 | FR-013-AC-3, FR-013-AC-4 | 🚧 planned |
| TC-063 | Route every missed or timed-out mutant by reviewed matrix priority to exactly one disposition and required follow-up evidence | Inspection | P0 | FR-013-AC-4 | 🚧 planned |
| TC-064 | Seed faults in population, status, denominator, isolation, restoration, and survivor controls and require red | Integration | P0 | FR-013-AC-5, NFR-005-AC-2 | 🚧 planned |
| TC-065 | Census exact proof candidates, round-trip bounded-proof identities, and admit proved-within-bounds only after every required check succeeds | Analysis | P0 | FR-014-AC-1, FR-014-AC-2 | 🚧 planned |
| TC-066 | Mutate assumptions, bounds, cover/unwind checks, outcome, and claim scope and require invalidation or red | Integration | P0 | FR-014-AC-2, FR-014-AC-3, NFR-005-AC-2 | 🚧 planned |
| TC-067 | Bind every campaign source, tool, environment, configuration, artifact, limitation, and retention identity | Integration | P0 | FR-012-AC-4, FR-014-AC-1, FR-014-AC-3, FR-015-AC-3, NFR-005-AC-1 | 🚧 planned |
| TC-068 | Refuse every count, ratio, plateau, no-crash, proof, or aggregate that widens into an automated authority claim | Integration | P0 | FR-012-AC-5, FR-015-AC-6, NFR-005-AC-3 | 🚧 planned |
| TC-069 | Prove verifier-only code cannot change ordinary/MSRV behavior or introduce a production/runtime dependency | Integration | P0 | FR-014-AC-4 | 🚧 planned |
| TC-070 | Reject closed-unmerged #35 artifacts as current proof evidence and accept only an exact-candidate successor run | Integration | P0 | FR-014-AC-5 | 🚧 planned |
| TC-071 | Enforce Rust producer and shared Quire/Quoin/Engineering-Assurance responsibility boundaries across all owners | Integration | P0 | FR-015-AC-1, FR-015-AC-6, NFR-005-AC-3 | 🚧 planned |
| TC-072 | Demonstrate that Quire and Quoin do not execute campaign producers and cannot synthesize missing producer bytes | Integration | P0 | FR-015-AC-1, FR-015-AC-2 | 🚧 planned |
| TC-073 | Round-trip all domain/shared states and mutation-test stale, partial, collapsed, substituted, and digest-bad records | Integration | P0 | FR-015-AC-2, FR-015-AC-3, NFR-005-AC-1, NFR-005-AC-2 | 🚧 planned |
| TC-074 | Refuse unsafe paths and resource-cap violations before artifact decoding or allocation | Integration | P0 | FR-015-AC-4, NFR-005-AC-2 | 🚧 planned |
| TC-075 | Refuse non-local, proposed, missing, definition-mismatched, or falsely release-labelled MeasurementPlan and verification-stack inputs | Integration | P0 | FR-015-AC-5 | 🚧 planned |
