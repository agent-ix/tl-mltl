---
id: SR-051
title: "Rust review — QObs C00 consumer compatibility"
type: SpecReview
analysis: code-review
scope: "FR-019; TC-085; src/wire/observation.rs; QObs C00 dependency and fixture boundary"
review_set: subset
---

# Rust review — QObs C00 consumer compatibility

## Summary

Applied the Rust review checklist to the C00 dependency transition, public
compatibility types, bounded temporal delegation, unsupported repair/query
dispositions, admitted producer fixture, and all affected repository gates.

## Verdict

**PASS after remediation.** The temporal path calls the existing bounded owner
adapter directly; unsupported QObs-owned contracts accept no artifact or value
and return closed typed metadata. No new unsafe block, unchecked numeric
conversion, production panic, async/blocking bridge, lock, recursion, or
unbounded collection was introduced.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5101 | high | **FIXED:** The initial supported compatibility variant carried no contract or compiled-revision metadata even though FR-019 requires every disposition to identify both. A constructor-private `Supported` value now exposes the same exact metadata as `Unsupported`. | `src/wire/observation.rs::Supported`; TC-085 |
| FND-5102 | high | **FIXED:** QObs C00 removed the primitive producer/subject fixture API. TC-084 now admits the exact pinned Filament Core Data static producer bundle and constructs `QualifiedSubject` values through QObs's typed boundary instead of recreating the old semantics. | `tests/tc_084_temporal_owner_wire.rs::qualified_with_clock`; `tests/fixtures/fcd-static-bundle-1.2.json` |
| FND-5103 | medium | **FIXED:** The C00 graph introduces the exact-pinned FCD baseline-producer Git source and AGPL-3.0-only license. Both are explicit in dependency policy and consumer documentation; Cargo Deny no longer treats the transitive owner source as ambient or unknown. | `deny.toml`; `README.md`; Cargo Deny |
| FND-5104 | medium | **FIXED:** Temporal compatibility delegates to `wire::request::derive` and TC-085 compares exact bytes, usage, the exact output boundary, and the one-over typed refusal. Repair/query compatibility takes only a closed selector, so TL cannot inspect, project, or accidentally retain either foreign artifact. | `src/wire/observation.rs::consume_temporal`; `src/wire/observation.rs::compatibility`; TC-085 |
| FND-5105 | medium | **FIXED:** The accepted temporal-owner landing advanced the crate to Rust 1.98.1 and moved the horizon module, while hosted Kani remained 0.67.0 on a Rust 1.93 nightly and targeted the obsolete harness path. The workflow now pins official Kani 0.68.0, whose release upgrades to the 2026-08-21 nightly, and the gate names the enumerated `future::horizon` harness. | `.github/workflows/ci.yml`; `Makefile`; `make kani-check` |
| FND-5106 | low | **FIXED:** The accepted temporal-owner landing left the live Rust/MSRV, source census, dependency provenance, and Quire coverage populations at pre-landing values. The executable controls now describe the accepted 1.98.1 owner tree and exact current populations without weakening any lane. | `.github/workflows/ci.yml`; `tests/shared_assurance.rs`; `assurance/{pins,change-assurance}.json` |

The workflow diff retains both hosted jobs and the complete `make ci` lane; it
changes only the already-declared Rust 1.98.1 toolchain values. Public items are
documented, closed matches are exhaustive, and the exact foreign contract labels
are centralized behind `Contract::label`.
