---
type: log
title: "PLAN-008 update log"
description: "What changed in the verification-effectiveness implementation plan, and when."
---

# PLAN-008 update log

## History

- **2026-09-10** - Created the blocked implementation plan from reviewed
  MRS-003, FR-020 through FR-024, NFR-005, MP-003 through MP-006, and TM-003.
  Decomposed the work into seven tasks across admission, critical-path, fuzz,
  mutation, and bounded-proof tracks. Recorded M0/M4, semantic-profile,
  local production-binding completeness, attachment, build-profile, and
  local-plan gates. First authored on branch
  `issue/39-verification-effectiveness-spec` (PR #45), stacked on the
  never-merged PR #44, and never merged itself.
- **2026-09-21** - Re-derived against current `main`. The plan identity is
  `PLAN-008` (`PLAN-004` was never on `main`, and `PLAN-006` and `PLAN-007` are
  now taken), its tasks are `Task-017` through `Task-023` continuing after
  PLAN-007's `Task-016`, and its requirements are `FR-020` through `FR-024`.
  `FR-011` through `FR-013` — the identities PR #45 used — are live dangling
  trace targets in `tests/past_history.rs`, where they name `tl-syntax`
  requirements; declaring them here would silently capture those tags and bind
  past-history tests to the wrong owner. For the same reason the test cases are
  `TC-104` through `TC-129` rather than PR #45's `TC-050` through `TC-075`,
  which collide with `main`'s allocations. PR #45's `SR-046`..`SR-071` review
  artifacts are not carried over: those identities are taken on `main` and they
  are claims about content that has since changed.
- **2026-09-21** - Removed the campaign's `quire-contract-ir` #63/#64 resume
  condition and the native-predicate/temporal-bridge obligations it gated.
  Under the TL-175 architect ruling the native-correspondence bridge is
  `quire-mltl`'s, recorded as ADR-002. Nothing else in the campaign changed:
  the property ledger, fuzz, mutation, Kani, and shared-intake obligations were
  already TL-internal. The campaign's only remaining external resume condition
  is tl-syntax #38 plus its routed evaluator support.
