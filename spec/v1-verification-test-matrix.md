---
id: TM-006
title: MLTL V1 executable verification test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-004
    type: covers
---

# MLTL V1 executable verification test matrix

Rows are planned until TL-215 accepts the combined V1 spec. The red stubs
under `spec/stubs/` do not count as executed coverage.

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-043 | FR-043-AC-1 through FR-043-AC-2 | TC-175, TC-176 | 🚧 planned |
| FR-044 | FR-044-AC-1 through FR-044-AC-2 | TC-177, TC-178 | 🚧 planned |
| FR-045 | FR-045-AC-1 through FR-045-AC-2 | TC-179, TC-180 | 🚧 planned |
| FR-046 | FR-046-AC-1 through FR-046-AC-2 | TC-181, TC-182 | 🚧 planned |
| FR-047 | FR-047-AC-1 through FR-047-AC-2 | TC-183, TC-184 | 🚧 planned |
| FR-048 | FR-048-AC-1 through FR-048-AC-2 | TC-185, TC-186 | 🚧 planned |
| FR-049 | FR-049-AC-1 through FR-049-AC-2 | TC-187, TC-188 | 🚧 planned |
| FR-050 | FR-050-AC-1 | TC-189 | 🚧 planned |
| FR-051 | FR-051-AC-1 | TC-190 | 🚧 planned |
| FR-052 | FR-052-AC-1 through FR-052-AC-2 | TC-191, TC-192 | 🚧 planned |
| FR-053 | FR-053-AC-1 through FR-053-AC-2 | TC-193, TC-194 | 🚧 planned |
| FR-054 | FR-054-AC-1 through FR-054-AC-2 | TC-195, TC-196 | 🚧 planned |
| FR-055 | FR-055-AC-1 through FR-055-AC-4 | TC-197 through TC-200 | ✅ implemented |

## Non-Functional Requirement Coverage

| Non-Functional Req | Verification Method | Evidence/Test Cases | Status |
|---|---|---|---|
| NFR-007 | Repeat under exact pins and mutate one pin | TC-175, TC-196 | 🚧 planned |
| NFR-008 | Reconcile declared and visited populations, seed omissions | TC-177, TC-180, TC-183, TC-185, TC-189 | 🚧 planned |
| NFR-009 | Force work and benchmark comparability boundaries | TC-188, TC-190, TC-199 | 🚧 planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-175 | Build dev-only tl-oracle with no production imports and compare reviewed finite, past and lasso fixtures | Integration | P0 | FR-043-AC-1, NFR-007-AC-1 | ✅ implemented |
| TC-176 | Seed independent wrong production/oracle clauses and reject self-oracled comparisons | Analysis | P0 | FR-043-AC-2 | 🚧 planned |
| TC-177 | Reconcile exact generated depth/operator/interval/trace/profile partition cardinality with visits and refusals | Property | P0 | FR-044-AC-1, NFR-008-AC-1 | 🚧 planned |
| TC-178 | Compare every visited admitted case with oracle and detect omission, duplication and wrong verdict | Property | P0 | FR-044-AC-2 | 🚧 planned |
| TC-179 | Classify each V1 criterion and check duality, embedding, unrolling, fairness, monotonicity and refutation laws | Property | P0 | FR-045-AC-1 | 🚧 planned |
| TC-180 | Strictly round-trip affected editions and reject vacuous generators and seeded faults | Property | P0 | FR-045-AC-2, NFR-008-AC-1 | 🚧 planned |
| TC-181 | Drive actual reader and operation through one fuzz target per crate with seed and budget | Fuzz | P0 | FR-046-AC-1 | 🚧 planned |
| TC-182 | Minimize and replay failures; refuse stale or unconfirmed fuzz artifacts | Fuzz | P0 | FR-046-AC-2 | 🚧 planned |
| TC-183 | Measure viable mutant population and kill-rate on a stable green critical-scope baseline | Analysis | P0 | FR-047-AC-1, NFR-008-AC-1 | 🚧 planned |
| TC-184 | Detect surviving/timed-out seeded mutants and source/test-selection contamination | Analysis | P0 | FR-047-AC-2 | 🚧 planned |
| TC-185 | Run exact Kani bounds, assumptions, unwind and path checks; reject vacuity/incompleteness | Analysis | P0 | FR-048-AC-1, NFR-008-AC-1 | 🚧 planned |
| TC-186 | Reproduce false-assertion counterexample and verify production subject body unchanged | Analysis | P0 | FR-048-AC-2 | 🚧 planned |
| TC-187 | Build tl-syntax no_std on a named embedded target and check other crates make no claim | Compile | P0 | FR-049-AC-1 | 🚧 planned |
| TC-188 | Run applicable Miri, at-limit/one-over and typed error probes without panic | Integration | P0 | FR-049-AC-2, NFR-009-AC-1 | 🚧 planned |
| TC-189 | Record llvm-cov line/branch counts and list every uncovered critical semantic branch | Analysis | P0 | FR-050-AC-1, NFR-008-AC-1 | 🚧 planned |
| TC-190 | Compare Criterion distributions on matching hardware and classify confirmed/noisy regressions | Benchmark | P1 | FR-051-AC-1, NFR-009-AC-1 | 🚧 planned |
| TC-191 | Execute a fresh exact-pin R2U2 run and compare reviewed bounded/past cases per step | Integration | P0 | FR-052-AC-1 | 🚧 planned |
| TC-192 | Replay infinite safety bad prefixes and keep pass/origin/unavailable results non-proving | Integration | P0 | FR-052-AC-2 | 🚧 planned |
| TC-193 | Compare complete/partial fair lassos including mixed past/future on repeated loops with tl-oracle | Property | P0 | FR-053-AC-1 | ✅ implemented |
| TC-194 | Detect wrong fixed point, fairness filter, prefix proof and FR-341 axis with seeded faults | Analysis | P0 | FR-053-AC-2 | 🚧 planned |
| TC-195 | Bind one report to actual lane outputs, exact pins, seeds and digests; reject stale substitute | Integration | P0 | FR-054-AC-1 | 🚧 planned |
| TC-196 | Reproduce deterministic payload and preserve all incomplete/refused/failed populations | Integration | P0 | FR-054-AC-2, NFR-007-AC-1 | 🚧 planned |
| TC-197 | Map V1–V11 milestones to named gates and pass/failure/incomplete states | Inspection | P0 | FR-055-AC-1 | ✅ implemented |
| TC-198 | Force one missing, failed and stale lane each and keep siblings independently visible | Integration | P0 | FR-055-AC-2 | ✅ implemented |
| TC-199 | Reject automated acceptance, release, parity or certification claims and exhausted success | Integration | P0 | FR-055-AC-3, NFR-009-AC-1 | ✅ implemented |
| TC-200 | Refuse mismatched Rust/Cargo identity and environment; accept exact member stable/nightly bindings | Integration | P0 | FR-055-AC-4 | ✅ implemented |
