---
id: Task-028
title: "Documentation and workflow wiring"
type: Task
status: done
track: C
priority: P1
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: inspection
github_issue: ix://agent-ix/tl-mltl/issues/14
relationships:
  - target: ix://agent-ix/tl-mltl/issues/14
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-024
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
  - target: ix://agent-ix/tl-mltl/TC-137
    type: verifies
---

# Task-028: Documentation and workflow wiring

## Scope

Update every documented or hosted reference to running the full local gate
set so it names the CI entry point rather than a bare `make ci`
(NFR-006-AC-8):

- `README.md` — check for any `make ci` reference.
- `CLAUDE.md`'s `## Commands` block, which currently lists `make ci
  # complete local gate` as the last command, and its `## The Makefile is
  not a trust root` section, which currently ends with "Tracked as
  `agent-ix/tl-mltl#14`" and should be updated once this closes the
  tracked gap it describes.
- `.github/workflows/ci.yml`, whose single manual-dispatch job currently
  runs `run: make ci` (confirmed at the file's `run: make ci` step) — the
  hosted dispatch is one of the "documented or hosted invocation paths"
  AC-8 names explicitly.

Add a `make guarded-ci` Makefile convenience target that runs `cargo run
--quiet --bin ci_guard -- ci`, matching tl-rewrite's own `make guarded-ci`
alias, so the documented command stays a `make` invocation rather than
requiring every caller to spell out `cargo run --bin ci_guard` directly.
`make ci` itself remains directly invocable (NFR-006 Scope: "`make ci`
remains directly invocable ... for local convenience; it is not itself the
assured gate") — this task does not remove or rename the existing `ci`
target, only adds `guarded-ci` alongside it and repoints documentation.

## Subtasks

- [ ] Add `.PHONY: guarded-ci` / `guarded-ci:` target to `Makefile`
  (`$(CARGO) run --quiet --bin ci_guard -- ci`).
- [ ] Update `CLAUDE.md`'s `## Commands` block to add `make guarded-ci` and
  mark it as the invocation every local and hosted caller should use, per
  `make ci`'s existing convenience-only framing.
- [ ] Update `CLAUDE.md`'s "The Makefile is not a trust root" section to
  record NFR-006/PLAN-009 as the remediation of the tracked
  `agent-ix/tl-mltl#14` gap, matching how tl-rewrite's own `CLAUDE.md`
  documents its `NFR-004`/`guarded-ci`.
- [ ] Grep `README.md` for `make ci` and update any hit found.
- [ ] Update `.github/workflows/ci.yml`'s `run: make ci` step to `run: make
  guarded-ci`.

## Deliverables

- Updated `Makefile`, `CLAUDE.md`, `README.md` (if it references `make ci`),
  `.github/workflows/ci.yml`.
- Inspection record (TC-137) confirming no remaining documented or hosted
  reference to a bare `make ci` as the full-gate-set invocation.

## Notes

- No code dependency on Task-025/026/027; can start as soon as Task-024's
  entry point exists to name, but should not land a literally concurrent
  edit to `Makefile` against Task-026 without rebasing serially
  (Coordination Rules in `plan.md`).
