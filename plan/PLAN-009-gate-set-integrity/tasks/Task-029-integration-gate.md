---
id: Task-029
title: "Integration gate"
type: Task
status: done
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: integration-test
github_issue: ix://agent-ix/tl-mltl/issues/14
relationships:
  - target: ix://agent-ix/tl-mltl/issues/14
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-025
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-027
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-028
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
  - target: ix://agent-ix/tl-mltl/TC-135
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-136
    type: verifies
---

# Task-029: Integration gate

## Scope

Exercise the assembled entry point (Task-024 static check + Task-025
environment check + Task-026 real completion records + Task-027
reconciliation), spawning the real compiled `ci_guard` binary rather than
calling library functions directly, against the tracked reproduction and a
positive control, run together rather than unit-by-unit — this is
deliberate: the tracked TL-65 measurement is specifically about mechanisms
interacting, so a partial per-mechanism pass here is not evidence any single
AC is actually done.

## Subtasks

- [ ] **TC-136** positive control: an unmodified fixture Makefile (two
  prerequisites, both `true`, matching tl-rewrite's `clean_makefile_passes`
  shape), clean environment — `ci_guard ci` exits 0, both gates have
  completion records from this run.
- [ ] **TC-135a**: `.IGNORE:`-prepended fixture Makefile — refused before
  Make ever runs, via static inspection (`ignore-directive` named on
  stderr), not via reconciliation; no completion records exist.
- [ ] **TC-135b**: skeleton fixture Makefile, every recipe a failing stub,
  no execution-control directive present — static inspection finds nothing;
  reconciliation catches it (`missing completion record: <gate>` named on
  stderr) because the failing recipe never reached its `record` call.
- [ ] **TC-135c**: a recipe line joining a failing check and the
  `record`call with a bare `;` (`false; ci_guard record gate-a`) — refused
  via static inspection (`recipe-chains-commands` named on stderr) before
  Make ever runs; the direct reproduction of the bypass class this plan
  bakes in from the start rather than discovering later.
- [ ] Real-HEAD run: invoke the entry point against this repository's own
  actual `Makefile` at HEAD. Record the result. If it does not cleanly pass,
  confirm (e.g. via `git stash -u` against unmodified main) whether the same
  gate fails identically without this plan's changes — matching how
  tl-rewrite's own Task-006 accepted a pre-existing, environment-specific
  failure as sufficient evidence rather than treating it as this plan's
  defect. Do not claim a clean pass that was not actually observed.

## Deliverables

- `tests/ci_guard.rs` process-level test suite (TC-135, TC-136) spawning
  `env!("CARGO_BIN_EXE_ci_guard")` against fixture Makefiles in a
  `tempfile::tempdir()`, matching tl-rewrite's `tests/ci_guard.rs` shape.
- A recorded real-HEAD run result (pass, or a named pre-existing unrelated
  failure) in this task's own history / the plan `log.md`.

## Notes

- **Gate, not a task with partial credit.** Do not proceed to Task-030 until
  every sub-case here is green. If a sub-case fails, bisect to Task-024/025/
  026/027 before re-attempting — see `plan.md`'s Gate entry.
