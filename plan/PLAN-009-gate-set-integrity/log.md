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
- **2026-09-22** — Independent adversarial review (`reviews/2026-09-22-tl-65-gate-set-integrity-code-review.md`,
  SR-055) found and confirmed end-to-end against the compiled binary two
  further high-severity bypasses missed by the self-review, both fixed
  before merge:
  - **FND-001**: `scan_recipe_line`'s dash-prefix check matched only a
    literal leading `-`, missing GNU Make's `@`/`-`/`+` recipe-prefix
    ordering (`@-false`/`+-false` ignore the recipe's exit status exactly
    like `-false`). Fixed via `recipe_prefix_carries_dash`, which walks the
    full leading run of `@`/`-`/`+` characters in any order and flags if
    `-` appears anywhere in it. Regression tests added at both the unit
    level (`scan_detects_dash_prefixed_recipe_with_at_prefix_first`,
    `..._with_plus_prefix_first`, plus a true-negative for `@+` without
    `-`) and the process level (`at_dash_prefixed_recipe_is_refused`,
    `plus_dash_prefixed_recipe_is_refused`).
  - **FND-002**: `is_assignment` only matched a bare `NAME ...` line, so
    `export MAKEFLAGS := -i` as a Makefile directive was invisible to the
    static scan — and also to the separate calling-environment `MAKEFLAGS`
    check, since it is set from inside the Makefile rather than inherited.
    Fixed via `strip_export_keyword`, applied before the existing
    prefix/operator match, covering all three `ASSIGNED_VARS` (`SHELL`,
    `.SHELLFLAGS`, `MAKEFLAGS`). Regression tests added at both levels
    (`scan_detects_exported_makeflags_assignment`,
    `scan_does_not_misread_a_differently_named_variable_as_export`,
    `exported_makeflags_directive_is_refused_before_make_runs`).

  Both fixes reconfirmed clean against `cargo test --lib` (64 passed),
  `cargo test --test ci_guard` (12 passed), `cargo fmt --check`,
  `cargo clippy --all-targets --all-features -D warnings`, and
  `make test-census` (145 tagged tests). Real-HEAD `make guarded-ci` run
  reconfirmed unchanged: static/environment checks pass cleanly, Make still
  stops at the same pre-existing `kani-check` toolchain mismatch. Findings
  and dispositions posted as a Linear comment on TL-65; the standalone
  review-report file deleted after that (this repository's `spec/reviews/`
  SR convention — SR-053 — is unaffected and kept).
