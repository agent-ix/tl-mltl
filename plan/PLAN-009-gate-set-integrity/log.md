---
type: log
title: "PLAN-009 update log"
description: "Lifecycle record for the tl-mltl NFR-006 gate-set-integrity bundle."
---

# PLAN-009 update log

## History

- **2026-09-22** — Bundle created from `spec/requirements/NFR-006-gate-set-integrity.md`
  (reviewed in `spec/reviews/SR-053-nfr-006-gate-set-integrity-spec-review.md`,
  one low-severity finding fixed, no blocking findings) for Linear TL-65 /
  `agent-ix/tl-mltl#14`. Task breakdown follows sibling repository
  tl-rewrite's merged `PLAN-007-gate-set-integrity` bundle (Linear TL-64,
  `agent-ix/tl-rewrite#11`, PR #48) as structural precedent, scoped to this
  repository's own 15-gate `ci:` prerequisite list and adapted file paths.
  Both bypass classes found after tl-rewrite's and tl-parse's initial
  versions shipped (bare-`;` recipe chaining; MAKEFLAGS bundled-flag and
  `|| true` narrow-boundary matching) are specified into Task-024/Task-025
  from the start rather than deferred to a later review finding.
- **2026-09-22** — Implemented and closed. `src/ci_guard.rs` (checkable
  logic, 41 unit tests) plus `src/bin/ci_guard.rs` (the `ci`/`record`
  entry point), `tests/ci_guard.rs` (9 process-level tests spawning the
  real compiled binary), all 15 `ci` prerequisite recipes in `Makefile`
  wired to `$(CI_GUARD) record <gate>`, and a new `guarded-ci` target.
  Both known bypass classes (bare-`;` recipe chaining; MAKEFLAGS
  flag-bundling including a `-`-prefixed bundle like `-ik`; `|| true`/`|| :`
  matched wherever it appears on the line) were built in from the start and
  covered by dedicated regression tests, plus additional realistic-variation
  cases (quoted `MAKEFLAGS` value with a trailing comment, `while`/`if`
  control-flow semicolons) beyond the plan's literal list. A self-review
  found `dangerous_makeflags` did not recognize a dash-prefixed bundle
  (`-ik`, as opposed to the space-free `ik` form) — fixed before closing;
  see the TL-65 Linear comment for the full review disposition. README,
  `CLAUDE.md`, and `.github/workflows/ci.yml` now name `make guarded-ci`.
  `spec/test-matrix.md` carries TC-130 through TC-137. Real-HEAD run: static
  inspection and environment checks pass; Make itself fails at `kani-check`
  (local Kani/rustc toolchain version mismatch) and `quire coverage
  --strict` already fails pre-existing (119 unbacked rows at the
  unmodified base commit) — both confirmed identical on the unmodified
  tree via `git stash`, matching tl-rewrite's Task-006 precedent for
  accepting a pre-existing, environment-specific failure as sufficient
  integration-gate evidence rather than this plan's own defect.
