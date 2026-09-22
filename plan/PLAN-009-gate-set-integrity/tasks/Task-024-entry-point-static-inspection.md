---
id: Task-024
title: "Entry-point scaffold + static Makefile inspection"
type: Task
status: done
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: unit-test
github_issue: ix://agent-ix/tl-mltl/issues/14
relationships:
  - target: ix://agent-ix/tl-mltl/issues/14
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
  - target: ix://agent-ix/tl-mltl/TC-130
    type: verifies
---

# Task-024: Entry-point scaffold + static Makefile inspection

## Scope

New `src/ci_guard.rs` library module (exported `pub mod ci_guard;` from
`src/lib.rs`) plus `src/bin/ci_guard.rs` binary — the CI entry point invoked
in place of a bare `make ci`.

- Parse CLI args for a `ci` command (and a `record GATE` command whose
  implementation Task-026 depends on, though its call sites land in
  Task-026).
- Parse the Makefile text, and recursively any `include`d file, for every
  execution-control surface NFR-006-AC-1 names: `SHELL`, `.SHELLFLAGS`,
  `MAKEFLAGS` assignment; `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`, `.SILENT:`
  as special targets; a `-`-prefixed recipe line; a recipe containing
  `|| true`, a stderr-to-`/dev/null` redirect; `$(eval` anywhere; **and,
  from the start, not as a later fix**: a recipe line joining two shell
  commands with a bare `;` — Make's exit status for a recipe line is the
  *last* shell command's, so `false; ci_guard record gate-a` reports success
  because `ci_guard record` always exits 0. The bare-`;` check must not flag
  a `;;` case-statement terminator, or a `;` immediately preceding `do`,
  `done`, `then`, `else`, `elif`, `fi`, or `esac` (ordinary shell control
  flow inside a recipe).
- Fix the completion-record file contract as a documented deliverable:
  `GateRecord { gate: String, run_id: String }`, JSON, one file per gate
  under `target/ci-gates/<gate>.json`. Task-026 (writer) and Task-027
  (reader) both build against this contract independently once it lands
  here.
- Track whether a recipe line is a `\`-continuation of the previous one — a
  `-`/`@`/`+` prefix, and the bare-`;` scan, only mean anything at the start
  of a fresh recipe command, never on a continuation line (e.g. a wrapped
  `--flag` on its own line is not the Make error-ignoring `-` prefix).

## Subtasks

- [ ] Add `sha2`-based run-id generation (dependency already present in
  `Cargo.toml`; no new production dependency).
- [ ] Implement `scan_makefile`/`scan_file`/`scan_recipe_line`/
  `scan_directive_line`, matching the eleven-plus-bare-`;` surface set.
- [ ] Implement `has_bare_command_separator` with the `;;`/control-word
  exclusion list (`do`, `done`, `then`, `else`, `elif`, `fi`, `esac`).
- [ ] Implement `parse_prerequisites` for the declared `ci:` rule
  (`\`-continuation aware).
- [ ] Implement `GateRecord`, `write_record` (with `is_valid_gate_name`
  path-traversal rejection — a fixed literal gate name is the only real
  caller today, but the check is fail-closed anyway, matching the same
  defense-in-depth this repository already applies elsewhere), `read_records`,
  `reset_gates_dir`.
- [ ] Wire `src/bin/ci_guard.rs`'s `ci` subcommand to call `scan_makefile`
  and refuse with every named violation on stderr before invoking Make.
- [ ] Add `#[derive(Serialize, Deserialize)]` with `#[serde(deny_unknown_fields)]`
  on `GateRecord`, matching this repository's `deserialize_contextual_record!`
  strict-decoding convention elsewhere in `src/lib.rs`.

## Deliverables

- `src/ci_guard.rs` with unit tests (TC-130): one clean-control case, one
  case per named surface (twelve total including bare-`;`), one `include`
  recursion case, one unresolvable-`include` case, one continuation-line
  false-positive-avoidance case, one comment-only false-positive-avoidance
  case, one `for`/`do`/`done` control-flow-semicolon acceptance case, one
  `;;` case-terminator acceptance case.
- `src/bin/ci_guard.rs` scaffold: `ci` subcommand performs the static scan
  and exits non-zero with named violations on any hit; does not yet invoke
  Make (Task-025/Task-027 add that).

## Notes

- This is the hard single-writer prerequisite for the rest of the plan
  (Coordination Rules in `plan.md`): the completion-record contract fixed
  here must not change once Task-025/026/027 start building against it.
- Bake the bare-`;` check in here, in the same commit as the other eleven
  surfaces — do not land AC-1 first and add the bare-`;` case in a follow-up
  task or review finding, per this repository's explicit direction for
  TL-65.
