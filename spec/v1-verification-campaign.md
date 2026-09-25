---
id: MRS-004
title: MLTL V1 executable verification campaign
type: MasterRequirements
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: depends_on
---

# MLTL V1 executable verification campaign

## Purpose

The V1 campaign measures correctness and robustness of the four production TL
crates with an independent oracle, exhaustive finite domains, property and
round-trip tests, fuzzing, mutation, bounded proof, embedded and Miri checks,
coverage, performance, live R2U2 comparison, and infinite-trace validation.
Each result is a reproducible measurement with explicit scope and limits.

## Boundaries

`tl-oracle` is a separate, unpublished dev-only repository and crate. It
consumes tl-syntax's public contracts but imports no production evaluator,
parser or rewriter code. tl-mltl and tl-rewrite take it only as a
dev-dependency. The provisional test-only lasso scaffold under
`spec/reference/` moves there when TL-35/TL-221 begin after TL-215
acceptance; it is not production or qualified evidence today.

The embedded `no_std` check applies to tl-syntax, which declares `#![no_std]`.
tl-parse, tl-rewrite and tl-mltl currently depend on `std`; this campaign
does not invent a `no_std` claim for them. A live R2U2 differential is an
explicit later lane and never retroactively labels the retained v4.2 exchange
as a new run.

## Verification order

| Milestone | Required result |
|---|---|
| V1 | Independent tl-oracle, fault injection proving independence |
| V2 | Complete 518,094-cell depth-one oracle partition; disclose literal depth-three cardinality and unvisited count |
| V3 | Property laws and strict round-trips |
| V4 | Per-crate fuzz targets and retained minimized corpus |
| V5 | Mutation kill-rate and reviewed survivor outcomes |
| V6 | Bounded Kani proofs with assumptions and counterexamples |
| V7 | Embedded tl-syntax, Miri and limit/error robustness |
| V8 | llvm-cov coverage by crate and critical branch |
| V9 | Criterion performance baselines and regression thresholds |
| V10 | Fresh live R2U2 differential under exact target pin |
| V11 | Infinite lasso, fairness and partial-valuation cross-checks |

FR-043 through FR-055 and NFR-007 through NFR-009 own these measurements;
TM-006 assigns TC-175 through TC-199. Existing MRS-002/MRS-003 campaign
paperwork is superseded for V1 by the disposition table in this document; its
real tests and historical evidence remain readable. No automated run grants
human TL-215 acceptance, release, native parity or certification.

## Prior campaign disposition

| Former scope | V1 disposition |
|---|---|
| FR-008 and FR-009 cell ledgers, fixture preimage and intake rules | Retire paperwork; keep existing corpus bytes and digest checks; FR-044/054 measure actual enumerated populations and replay |
| FR-010 target disposition paperwork | Retire wrapper schema; keep real mapping and target checks under FR-052 and TL-211 |
| FR-020 property obligation ledger | Replace ledger and preimage rules with FR-045's per-criterion executable classification |
| FR-021 fuzz campaign wrapper and plateau paperwork | Keep fuzz target, reproducible seed/budget and crash replay under FR-046 |
| FR-022 mutation population and survivor paperwork | Keep actual baseline, score and reviewed survivor evidence under FR-047 |
| FR-023 bounded-proof record machinery | Keep Kani harness, bounds, checks and counterexample evidence under FR-048 |
| FR-024 external intake/routing | Retire for this campaign; FR-054 records one local reproducible report |
| NFR-004/NFR-005 and MP-002 through MP-006 | Replace duplicate measurement plans and generic retention rules with NFR-007 through NFR-009 |
| TC-091 through TC-129 | Historical planned rows, not V1 completion credit; implemented test-bearing cases remain runnable and are linked as predecessor evidence |

The old requirements and matrices retain `superseded` status and the old
measurement plans use `retired` status,
not silently deleted or represented as completed V1 tests. The local V1 gate
executes the new cases; deeper nightly and external lanes state their status
without claiming they ran in the local gate.
