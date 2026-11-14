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

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-020 | FR-020-AC-1 through FR-020-AC-5 | TC-104 through TC-108 | 🚧 planned |
| FR-021 | FR-021-AC-1 through FR-021-AC-5 | TC-109 through TC-112, TC-121, TC-122 | 🚧 planned |
| FR-022 | FR-022-AC-1 through FR-022-AC-5 | TC-113 through TC-118 | 🚧 partial — FR-022-AC-2 refuses a failing baseline; the unstable-baseline half has no mechanism or test yet |
| FR-023 | FR-023-AC-1 through FR-023-AC-4 (FR-023-AC-5 withdrawn) | TC-120, TC-121, TC-123 | 🚧 planned |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-005 | seeded faults, weakened test selections, and campaign report replay | TC-107, TC-114, TC-118, TC-120 through TC-122, TC-127 | 🚧 planned |

## Allocation

| Repository | Owned campaign surface |
|---|---|
| tl-syntax | formula, profile, and schema property, fuzz, and mutation work |
| tl-parse | parse, format, and span properties and input-boundary fuzz work |
| tl-rewrite | equivalence and counterexample properties and fuzz work |
| tl-mltl | evaluator, horizon, history, and mapping properties, fuzz targets, mutation, and Kani harnesses |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-104 | Run each law in the evaluator law set as a tagged generated-input or finite-exhaustive test bound to its criterion id | Property | P0 | FR-020-AC-1 | ✅ implemented |
| TC-105 | Compare production evaluation with the independent oracle over generated and exhaustive domains for laws that need a reference | Property | P0 | FR-020-AC-1, FR-020-AC-2 | ✅ implemented |
| TC-106 | Fail a property that passes against a seeded-broken evaluator, uses a self-oracle, or discards beyond its ceiling | Property | P0 | FR-020-AC-2, FR-020-AC-3 | ✅ implemented |
| TC-107 | Reproduce property runs from recorded seed and case count, including accepted/discarded counts and shrunk counterexample | Property | P0 | FR-020-AC-3, FR-020-AC-4, NFR-005-AC-1 | ✅ implemented |
| TC-108 | Keep generated inputs outside the canonical corpus and commit a minimised counterexample as a regression fixture that replays clean | Integration | P0 | FR-020-AC-5 | 🚧 planned |
| TC-109 | Run every fuzz target under at least three distinct recorded seeds with a fixed budget and record target, seed, duration, executions, and coverage | Fuzz | P0 | FR-021-AC-1, FR-021-AC-4 | 🚧 planned |
| TC-110 | Stop a run at the first of budget, crash, or plateau over the stated window, and record a run still growing at the budget as growth observed | Fuzz | P0 | FR-021-AC-2 | 🚧 planned |
| TC-111 | Record crash, timeout, out-of-memory, tool error, resource refusal, cancellation, and not-run outcomes as distinct states | Integration | P0 | FR-021-AC-2, FR-021-AC-3 | 🚧 planned |
| TC-112 | Replay every committed crash input and checked-in seed under `cargo test`, reaching accepting and refusing paths | Integration | P0 | FR-021-AC-3, FR-021-AC-4 | 🚧 planned |
| TC-113 | Mutate the evaluator core and the crate as a whole and exclude the CI-plumbing modules by configuration | Integration | P0 | FR-022-AC-1 | ✅ implemented |
| TC-114 | Refuse the mutation campaign and emit no kill rate when the unmutated baseline fails or is unstable | Integration | P0 | FR-022-AC-2, NFR-005-AC-1 | 🚧 partial — a failing baseline is refused with no kill rate; an unstable baseline (passes on one run, fails on another) has no mechanism or test yet |
| TC-115 | Report caught, missed, timeout, unviable, and tool-error outcomes separately | Integration | P0 | FR-022-AC-3 | ✅ implemented |
| TC-116 | Compute the kill rate as caught over caught plus missed plus timeout beside the raw counts and fail below 95% for the evaluator core or 85% for the crate | Integration | P0 | FR-022-AC-3, FR-022-AC-4 | ✅ implemented |
| TC-117 | List every surviving mutant with exactly one reason (killing test, stated equivalence, or named Kani harness) and fail a survivor without one | Integration | P0 | FR-022-AC-5 | ✅ implemented |
| TC-118 | Weaken the test selection and require the threshold gate to fail while the unweakened selection passes | Integration | P0 | FR-022-AC-4, NFR-005-AC-2 | ✅ implemented |
| TC-119 | Census the finite proof-candidate registry under candidate, proposition, and ledger digest preimages | Analysis | P0 | FR-023-AC-1 | ⛔ retired — proof-candidate ledger and digest ceremony; the harness list and its bounds are checked by TC-120 |
| TC-120 | Check that each Kani harness states its bounds and non-claims, that the gate runs every listed harness by exact name, and that a vacuous proof, incomplete unwinding, or timeout fails | Integration | P0 | FR-023-AC-1, FR-023-AC-2, FR-023-AC-3, NFR-005-AC-2 | 🚧 planned |
| TC-121 | Record source revision, tool versions, seeds, and per-member outcome in the campaign report, and refuse a replay against a moved or dirty source | Integration | P0 | FR-021-AC-4, FR-023-AC-3, NFR-005-AC-1 | 🚧 planned |
| TC-122 | Refuse every count, ratio, plateau, no-crash result, proof, or aggregate that widens into an automated authority claim | Integration | P0 | FR-021-AC-5, NFR-005-AC-3 | 🚧 planned |
| TC-123 | Prove verifier-only code cannot change ordinary or MSRV behavior or add a production dependency on Kani or another verifier | Integration | P0 | FR-023-AC-4 | 🚧 planned |
| TC-124 | Reject closed-unmerged #35 artifacts as current proof evidence and accept only an exact-candidate successor run | Integration | P0 | FR-023-AC-5 | ⛔ retired — obsolete process gate for a closed PR's artifacts; a harness result is current only when run at the candidate revision, which the campaign report already records |
| TC-127 | Report not-run, unavailable, and planned campaign members as non-pass and refuse an unregistered evidence target | Integration | P0 | NFR-005-AC-1, NFR-005-AC-2 | 🚧 planned |
| TC-128 | Exercise path, digest, count, length, size, depth, checked-arithmetic, and unknown-limit boundaries of a corpus manifest loader | Integration | P0 | NFR-005-AC-2 | ⛔ retired — loader resource profile from the withdrawn FR-009-AC-6; no corpus manifest loader exists |
