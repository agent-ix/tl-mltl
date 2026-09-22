---
id: Task-027
title: "Declared/executed reconciliation"
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
  - target: ix://agent-ix/tl-mltl/Task-024
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-026
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
  - target: ix://agent-ix/tl-mltl/TC-133
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-134
    type: verifies
---

# Task-027: Declared/executed reconciliation

## Scope

Wire the entry point's `ci` subcommand end to end: after the static scan
(Task-024) and the environment check (Task-025) both pass, reset
`target/ci-gates/` (so no prior run's records leak into this run — AC-5
defense in depth), generate a fresh run id, invoke
`Command::new("make").arg("ci")` with `MAKEFLAGS`/`MFLAGS` removed and the
run id exported as `CI_GUARD_RUN_ID`, then read the completion-record set
(Task-026's writer) and reconcile it against the declared `ci:` prerequisite
set (Task-024's `parse_prerequisites`).

A record only counts if its `run_id` matches the run under evaluation — a
record left over from, replayed from, or backdated to a different run is
treated as no record at all (AC-5), not a stale-but-valid pass.

Report every mismatch by name — declared gates with no valid record for this
run, and records for undeclared gates — and exit non-zero if either set is
non-empty, or if Make itself exited non-zero.

## Subtasks

- [ ] Implement `Reconciliation { missing: Vec<String>, unexpected:
  Vec<String> }` and `reconcile(declared, records, run_id) -> Reconciliation`.
- [ ] Wire `src/bin/ci_guard.rs`'s `ci` subcommand: reset gates dir, generate
  run id, invoke Make with sanitized environment, read records, reconcile,
  report, and set the final exit code from the reconciliation result (not
  from Make's raw exit code alone — a reconciliation violation must fail
  even if Make's own exit code looked clean, and vice versa is impossible
  since a genuinely failing gate never writes its record).
- [ ] Confirm the entry point checks reconciliation *before* trusting Make's
  raw exit status, so a stale/backdated record cannot mask a real failure.

## Deliverables

- `reconcile` with unit tests (TC-133): missing+unexpected reported
  together; exact match reports neither.
- Run-id freshness unit tests (TC-134): a record with a different `run_id`
  from a prior invocation is treated as absent; a record deleted after
  being written is treated as absent.
- Process-level test (in `tests/ci_guard.rs`): a stale record file present
  in `target/ci-gates/` from an unrelated prior run does not leak into a
  pass for the current run.

## Notes

- This task and Task-025 are the last two mechanisms the entry point needs
  before Task-029's integration gate; Task-028 (docs) has no code
  dependency on this task and can land independently.
