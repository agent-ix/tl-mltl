---
id: Task-026
title: "Per-gate completion records"
type: Task
status: done
track: B
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
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
  - target: ix://agent-ix/tl-mltl/TC-132
    type: verifies
---

# Task-026: Per-gate completion records

## Scope

Touch all 15 `ci` prerequisite recipes in the Makefile (`fmt-check`, `lint`,
`kani-check`, `test`, `check-corpus`, `conformance`, `differential`,
`cli-conformance`, `test-census`, `deny`, `audit-unsafe`, `spec`, `msrv`,
`rustdoc`, `assurance`) so each writes a completion record, matching
Task-024's `GateRecord` contract, only on its own successful completion.

Each recipe's final line becomes a call to the compiled `ci_guard record
<gate>` binary, appended as its own recipe line — **never joined to the
preceding command with a bare `;`**, which is exactly the bypass Task-024's
static scan (`RecipeChainsCommands`/`has_bare_command_separator`) exists to
catch. Use one shared, mechanical pattern across all 15 recipes rather than
15 independently hand-varied edits, so the diff stays reviewable and no
gate's actual command changes — only a record-write line is appended.

`ci_guard record GATE` reads the `CI_GUARD_RUN_ID` environment variable
(set by the entry point's `ci` subcommand once Task-027 adds the Make
invocation); outside a `ci_guard ci` run (e.g. a developer running `make
lint` alone for local iteration) the variable is absent and `record` is a
deliberate no-op, not a failure — this must not turn an ordinary local
`make lint` into a build break.

## Subtasks

- [ ] Implement `src/bin/ci_guard.rs`'s `record GATE` subcommand: no-op
  success if `CI_GUARD_RUN_ID` is unset; otherwise call `ci_guard::write_record`.
- [ ] Add one new recipe line to each of the 15 `ci` prerequisite recipes in
  `Makefile`, appended after that recipe's existing command(s), never
  combined onto the same line with `;`.
- [ ] Confirm no recipe's actual command text changed — diff review must
  show only additive lines.
- [ ] Add `tempfile = "=3.10.1"` to `[dev-dependencies]` in `Cargo.toml`
  (not currently present; needed for the process-level tests this task and
  Task-027/029 add).

## Deliverables

- Updated `Makefile` (15 recipes, each with an appended `record` call).
- `src/bin/ci_guard.rs`'s `record` subcommand.
- Process-level test (TC-132, in `tests/ci_guard.rs`, spawning the real
  compiled binary against a small fixture Makefile with two prerequisites —
  one succeeding, one failing — matching tl-rewrite's own
  `successful_recipe_writes_its_own_record_only` test shape): the
  succeeding gate has a record, the failing gate does not.

## Notes

- Widest blast radius in this plan (touches every `ci` prerequisite recipe),
  but mechanically the simplest task — correctness of any individual gate's
  own command stays out of NFR-006's ownership (NFR-006 Scope); this task
  only appends a record-write line.
- `Makefile` is shared mutable state with Task-028 (Coordination Rules in
  `plan.md`) — rebase serially rather than landing concurrent unreviewed
  edits to the same file.
