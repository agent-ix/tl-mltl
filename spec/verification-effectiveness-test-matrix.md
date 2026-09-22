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
| FR-020 | FR-020-AC-1 through FR-020-AC-5 | TC-104 through TC-108 | 🚧 planned |
| FR-021 | FR-021-AC-1 through FR-021-AC-5 | TC-109 through TC-112, TC-121, TC-122 | 🚧 planned |
| FR-022 | FR-022-AC-1 through FR-022-AC-5 | TC-113 through TC-118 | 🚧 planned |
| FR-023 | FR-023-AC-1 through FR-023-AC-5 | TC-119 through TC-121, TC-123, TC-124 | 🚧 planned |
| FR-024 | FR-024-AC-1 through FR-024-AC-6 | TC-121, TC-122, TC-125 through TC-129 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-005 | deterministic census/replay, fault seeding, mutation, and claim-boundary checks | TC-107, TC-114, TC-118, TC-120 through TC-122, TC-125, TC-127 through TC-129 | 🚧 planned |

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
| TC-104 | Census every exact Quire criterion once under the exact row/domain/ledger preimages and reject omitted, duplicate, reordered, stale, unknown, or invalid-priority rows | Integration | P0 | FR-020-AC-1 | 🚧 planned |
| TC-105 | Round-trip applicable, excluded, and blocked property rows with complete finite domain, ownership, predecessor, and digest fields | Property | P0 | FR-020-AC-1, FR-020-AC-2 | 🚧 planned |
| TC-106 | Reject self-oracles, shared derivations, vacuous partitions, and seeded faults missed by the claimed oracle | Integration | P0 | FR-020-AC-2, FR-020-AC-3 | 🚧 planned |
| TC-107 | Reproduce generated and exhaustive property runs, including seed, accepted/discarded classes, failures, and shrinking | Property | P0 | FR-020-AC-3, FR-020-AC-4, NFR-005-AC-1 | 🚧 planned |
| TC-108 | Keep generated inputs outside canonical corpus and refuse incomplete promotion | Integration | P0 | FR-020-AC-5 | 🚧 planned |
| TC-109 | Enforce the exact campaign preimage, three pairwise-distinct ordered `u64` seeds, both finite caps, first-stop semantics, and requested-versus-observed budgets | Fuzz | P0 | FR-021-AC-1, FR-021-AC-4 | 🚧 planned |
| TC-110 | Compute plateau only from cadence-valid exact feature sets at both final-window anchors under unchanged instrumentation; reject missing anchors and insufficient windows | Fuzz | P0 | FR-021-AC-1, FR-021-AC-2 | 🚧 planned |
| TC-111 | Preserve fuzz growth, plateau, crash, timeout, resource, cancellation, tool, instrumentation, and not-run states | Integration | P0 | FR-021-AC-2, FR-021-AC-3 | 🚧 planned |
| TC-112 | Retain and independently replay exact fuzz corpora and crashes, refusing missing or stale identities | Integration | P0 | FR-021-AC-3, FR-021-AC-4 | 🚧 planned |
| TC-113 | Deterministically discover and bind every mutant under the exact path/span/identity, closed-priority ordering, and population digest before exclusion and selection | Integration | P0 | FR-022-AC-1 | 🚧 planned |
| TC-114 | Reproduce selected/unselected/excluded populations and refuse failed, inconsistent, drifting, or partial controls | Integration | P0 | FR-022-AC-1, FR-022-AC-2, NFR-005-AC-1 | 🚧 planned |
| TC-115 | Isolate each mutant, interleave controls, enforce timeout and clean restoration, and keep every execution state distinct | Integration | P0 | FR-022-AC-2, FR-022-AC-3 | 🚧 planned |
| TC-116 | Emit the shared score only for a complete nonzero viable denominator and retain every raw count | Integration | P0 | FR-022-AC-3, FR-022-AC-4 | 🚧 planned |
| TC-117 | Route raw missed/timeout outcomes into proof review without a circular final disposition, then require exactly one reviewed outcome with acyclic duplicates and unexpired risk evidence | Inspection | P0 | FR-022-AC-4 | 🚧 planned |
| TC-118 | Seed faults in population, status, denominator, isolation, restoration, and survivor controls and require red | Integration | P0 | FR-022-AC-5, NFR-005-AC-2 | 🚧 planned |
| TC-119 | Census the finite proof-candidate registry under exact candidate/proposition/ledger preimages and admit proved-within-bounds only after every required check succeeds | Analysis | P0 | FR-023-AC-1, FR-023-AC-2 | 🚧 planned |
| TC-120 | Mutate assumptions, bounds, cover/unwind checks, outcome, and claim scope and require invalidation or red | Integration | P0 | FR-023-AC-2, FR-023-AC-3, NFR-005-AC-2 | 🚧 planned |
| TC-121 | Bind every campaign source, tool, environment, configuration, artifact, limitation, and retention identity | Integration | P0 | FR-021-AC-4, FR-023-AC-1, FR-023-AC-3, FR-024-AC-3, NFR-005-AC-1 | 🚧 planned |
| TC-122 | Refuse every count, ratio, plateau, no-crash, proof, or aggregate that widens into an automated authority claim | Integration | P0 | FR-021-AC-5, FR-024-AC-6, NFR-005-AC-3 | 🚧 planned |
| TC-123 | Prove verifier-only code cannot change ordinary/MSRV behavior or introduce a production/runtime dependency | Integration | P0 | FR-023-AC-4 | 🚧 planned |
| TC-124 | Reject closed-unmerged #35 artifacts as current proof evidence and accept only an exact-candidate successor run | Integration | P0 | FR-023-AC-5 | 🚧 planned |
| TC-125 | Enforce Rust producer and shared Quire/Quoin/Engineering-Assurance responsibility boundaries across all owners | Integration | P0 | FR-024-AC-1, FR-024-AC-6, NFR-005-AC-3 | 🚧 planned |
| TC-126 | Demonstrate that Quire and Quoin do not execute campaign producers and cannot synthesize missing producer bytes | Integration | P0 | FR-024-AC-1, FR-024-AC-2 | 🚧 planned |
| TC-127 | Round-trip all domain/shared states and mutation-test stale, partial, collapsed, substituted, and digest-bad records | Integration | P0 | FR-024-AC-2, FR-024-AC-3, NFR-005-AC-1, NFR-005-AC-2 | 🚧 planned |
| TC-128 | Exercise every FR-009 path, digest, count, length, size, depth, checked-arithmetic, and unknown-limit boundary before artifact decoding or allocation | Integration | P0 | FR-024-AC-4, NFR-005-AC-2 | 🚧 planned |
| TC-129 | Validate PLAN-008 and linked tickets for unique ids, owners, consumers, methods, predecessors, resume conditions, local active plans, and truthful stacks; refuse every missing, stale, or falsely release-labelled input | Integration | P0 | FR-024-AC-5 | 🚧 planned |
