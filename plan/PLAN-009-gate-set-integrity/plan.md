---
id: PLAN-009
title: "Bind the declared and executed CI gate set"
type: Plan
status: done
relationships:
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
  - target: ix://agent-ix/tl-mltl/FR-006
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-003
    type: references
  - target: ix://agent-ix/tl-mltl/issues/14
    type: depends_on
---

# PLAN-009: Bind the declared and executed CI gate set

## Objective

Remediate Linear TL-65 / GitHub `agent-ix/tl-mltl#14` by implementing
NFR-006: a dedicated Rust CI entry point, invoked in place of a bare
`make ci`, that (1) refuses to run when the Makefile text or invocation
environment carries a state capable of suppressing prerequisite-failure
propagation, and (2) independently reconciles the set of gates that actually
completed against the declared `ci` prerequisite set, so a false pass on any
of the 15 gates is caught regardless of the mechanism that produced it. No
other requirement's gate logic changes; this plan only adds the binding
around it.

This follows the precedent sibling repository tl-rewrite set for the
identical problem (`NFR-004-gate-set-integrity`, Linear TL-64,
`agent-ix/tl-rewrite#11`, merged PR #48) and that tl-parse independently
built for TL-66, rather than the shared upstream control TL-65 originally
cited (`agent-ix/engineering-assurance#11`), which was confirmed in the
ticket thread to be unrelated (producer/runner qualification campaigns, not
gate-set binding) with no matching shared control existing anywhere
upstream. This mechanism is repository-local and Makefile-specific; it is
not a proposal for a shared cross-repository control (NFR-006 Scope).

Both bypass classes discovered after tl-rewrite's and tl-parse's initial
versions shipped — a bare `;` joining a check and the completion-record call
on one recipe line, and a MAKEFLAGS `-i`/`--ignore-errors` check narrow to a
bare `i` token plus an `|| true` detector anchored too tightly — are
specified in NFR-006-AC-1/AC-2 and are implemented in Task-024/Task-025 from
the start, not discovered later as review findings.

## Requirements Summary

### Non-Functional Requirements

- [x] **NFR-006-AC-1**: Static Makefile-text inspection refuses to proceed on
  `SHELL`/`.SHELLFLAGS`/`MAKEFLAGS` assignment, `.ONESHELL:`/`.DEFAULT:`/
  `.IGNORE:`/`.SILENT:`, a `-`-prefixed recipe line, `|| true`, a
  stderr-to-`/dev/null` redirect, a bare `;` command separator (excluding
  `;;` and control-word-prefixed semicolons), `$(eval`, or any of the above
  inside a recursively scanned `include`d file.
- [x] **NFR-006-AC-2**: The entry point does not forward inherited
  `MAKEFLAGS` and invokes Make with an explicit, minimal flag set; an
  `-i`/`-k`/`-S`-equivalent `MAKEFLAGS` in the calling environment, including
  a GNU Make bundled-flag token (e.g. `ik`), is refused before Make runs.
- [x] **NFR-006-AC-3**: Each of the 15 `ci` prerequisites' recipes writes a
  per-gate completion record naming itself and its own recipe's exit status,
  only on that recipe's own successful completion.
- [x] **NFR-006-AC-4**: After Make returns, the entry point reconciles the
  completion-record set against the declared `ci` prerequisite set and
  reports every mismatch in either direction.
- [x] **NFR-006-AC-5**: A missing, removed, or backdated completion record is
  treated as no record, not a pass.
- [x] **NFR-006-AC-6**: The tracked measurement (`.IGNORE:`-prepended
  Makefile copy; skeleton with every recipe replaced by a failing stub; a
  bare-`;` chained check-and-record recipe line) is rejected by the entry
  point.
- [x] **NFR-006-AC-7**: An unmodified Makefile, clean environment, and
  genuinely passing gates — including ordinary shell `for`/`case`
  control-flow semicolons — yield a zero exit with no violation.
- [x] **NFR-006-AC-8**: README, `CLAUDE.md`, and the hosted workflow dispatch
  reference the entry point rather than a bare `make ci`.

## Implementation shape

- **Binary**: a second Cargo binary, `src/bin/ci_guard.rs`, alongside the
  existing default `tl-mltl` binary (`src/main.rs`). Cargo auto-discovers
  every file under `src/bin/` as its own binary target with no explicit
  `[[bin]]` section needed — this repository's `Cargo.toml` currently has
  none for `tl-mltl` either, so no `Cargo.toml` change is required beyond
  the new file existing. This mirrors tl-rewrite's own `src/bin/ci_guard.rs`
  exactly, which this plan follows as precedent rather than inventing a
  different packaging shape.
- **Library module**: `src/ci_guard.rs`, exported as `pub mod ci_guard;`
  from `src/lib.rs` alongside the existing `clock`, `context`,
  `differential`, `future`, `mapping`, `past`, `wire` modules — the checkable
  logic (scanning, reconciliation, record I/O) lives here so it is unit
  tested directly, and `src/bin/ci_guard.rs` stays a thin process/CLI shell,
  matching the existing `lib.rs`/`main.rs` split this repository already
  uses for its own CLI.
- **Dependencies**: `sha2` and `serde`/`serde_json` are already direct
  dependencies (Cargo.toml) — sufficient for a run-id hash and completion
  records without adding a new production dependency. `tempfile = "=3.10.1"`
  is added to `[dev-dependencies]` (not currently present) for the process
  tests, matching the exact version tl-rewrite pins for the same purpose.
- **Completion records**: written under `target/ci-gates/<gate>.json`,
  matching tl-rewrite's convention (a `target/`-relative path, cleaned and
  ignored by git already via the existing `target/` `.gitignore` entry).
- **Gate list**: the 15 declared `ci` prerequisites in this repository's
  actual Makefile, in order: `fmt-check`, `lint`, `kani-check`, `test`,
  `check-corpus`, `conformance`, `differential`, `cli-conformance`,
  `test-census`, `deny`, `audit-unsafe`, `spec`, `msrv`, `rustdoc`,
  `assurance`.

## Dependency Graph

- `Task-024 -> Task-025, Task-026`
  Reason: both the environment-control check and the completion-record work
  build on the entry-point scaffold and the completion-record file contract
  Task-024 defines; neither needs the other to start.
- `Task-024 + Task-026 -> Task-027`
  Reason: reconciliation (AC-4/AC-5) needs both the entry point to reconcile
  from and real completion records to reconcile against.
- `Task-024 -> Task-028`
  Reason: AC-8's documentation/workflow references need the entry point to
  name; content and behavior don't depend on Task-025/026/027.
- `Task-025 + Task-027 + Task-028 -> Task-029`
  Reason: the integration gate exercises every mechanism together
  (static+env pre-flight, reconciliation, and the documented invocation
  path) and cannot pass partially — the tracked measurement this plan
  reproduces is specifically about mechanisms interacting.
- `Task-029 -> Task-030`
  Reason: review and matrix closure happen against a working, gate-passed
  implementation, not against work still in flight.

### Shared dependencies

- The completion-record file format (path convention, gate-name field,
  exit-status field, run-id field) is a shared contract between the writer
  (Task-026, the 15 Makefile recipes) and the reader (Task-027, the entry
  point's reconciliation). Task-024 fixes this contract in its deliverable
  so Task-025/026 can proceed in parallel against a stable shape.

### Cross-cutting constraints

- NFR-003 applies to every gate whose result the Quoin-bound
  `assurance-inputs` chain already reads (`conformance`, `differential`,
  `cli-conformance`, `test-census`, `spec`'s `quire coverage` half, `msrv`);
  this plan does not duplicate that coverage, and Task-027's reconciliation
  treats those gates identically to the remaining ones the chain cannot
  see — the completion record, not the chain, is what NFR-006 reconciles
  against.
- Every new production line in this plan is Rust, matching the owner
  directive recorded on TL-65: no task introduces a new Python/shell
  evidence framework; `scripts/*.py` stays as-is and out of this plan's
  scope.
- No task in this plan touches `scripts/*.py`, `assurance/`, or any gate's
  own domain correctness (`src/clock.rs`, `src/context.rs`,
  `src/differential.rs`, `src/future/`, `src/mapping/`, `src/past/`,
  `src/wire/`); if a task's implementation seems to need that, stop and
  re-check against NFR-006's Scope before proceeding.

## Test Plan

### Unit Tests

- [x] **TC-130** (NFR-006-AC-1): Static inspector rejects each of `SHELL`,
  `.SHELLFLAGS`, `MAKEFLAGS`, `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`,
  `.SILENT:`, a `-`-prefixed recipe line, `|| true`, a stderr-to-`/dev/null`
  redirect, and `$(eval` individually, each as its own case, plus a bare `;`
  command-separator case and a case placing one of the surfaces inside an
  `include`d file; a control Makefile with none present is accepted, and a
  `for`/`case` recipe using `;`/`;;` control-flow semicolons is accepted
  without a false positive.
- [x] **TC-131** (NFR-006-AC-2): Entry point invoked with `MAKEFLAGS` set to
  an `-i`/`-k`/`-S`-equivalent value, including a bundled short-flag token
  (e.g. `ik`), in the environment is refused before Make starts; invoked
  with a clean environment, it is not; `|| true` is detected wherever it
  appears on a recipe line, including immediately before a trailing `;` and
  further commands.

### Integration Tests

- [x] **TC-132** (NFR-006-AC-3): Running each of the 15 `ci` prerequisites
  individually against its real recipe writes exactly one completion record
  naming that gate and its exit status; a recipe forced to fail (stubbed
  tool) writes no record for that gate.
- [x] **TC-133** (NFR-006-AC-4): Given a completion-record set that is
  missing one declared gate and carries one record for an undeclared gate,
  the entry point reports both mismatches by name; given an exact match, it
  reports none.
- [x] **TC-134** (NFR-006-AC-5): A completion record deleted after being
  written, and one rewritten with a run-id preceding the evaluated run, are
  each treated as absent by reconciliation.
- [x] **TC-135** (NFR-006-AC-6): The entry point run against a
  `.IGNORE:`-prepended copy of the real Makefile, a skeleton Makefile with
  every recipe replaced by a failing stub, and a recipe line joining a
  failing check and the completion-record call with a bare `;`, each exits
  non-zero with a named violation — the direct reproduction of the tracked
  measurement in TL-65/`agent-ix/tl-mltl#14`.
- [x] **TC-136** (NFR-006-AC-7): The entry point run against the real,
  unmodified Makefile with a clean environment and every real gate passing
  exits zero with no violation reported.

### Verification (NFRs)

- [x] **TC-137** (NFR-006-AC-8): Inspect README, `CLAUDE.md`, and
  `.github/workflows/*.yml` for every reference to running the full local
  gate set; each must name the entry point, not a bare `make ci`.
  Inspection, not a test.

## Remaining Work

### Remaining Dependency Graph

```text
Task-024 (entry-point scaffold + static inspection, AC-1)
   |-- Task-025 (env control, AC-2)              [Track B]
   |-- Task-026 (15 completion records, AC-3)     [Track B]
   |         \
   |          Task-027 (reconciliation, AC-4/5)   [Track A]
   |-- Task-028 (docs/workflow wiring, AC-8)      [Track C]
   \
    (Task-025, Task-027, Task-028) -> Task-029 (integration gate, AC-6/7)
                                          -> Task-030 (review + matrix close)
```

### Track A: Critical Path (serial)

#### A1: Task-024 — entry-point scaffold + static Makefile inspection
- **Scope:** New `src/ci_guard.rs` library module plus `src/bin/ci_guard.rs`
  binary. Parses CLI args for a target (`ci` initially), parses the Makefile
  text (and any `include`d file, recursively) for the twelve
  execution-control surfaces in AC-1 — including the bare-`;`
  command-separator check with its `;;`/control-word exclusions, built in
  from the start rather than added after a later review — and refuses with a
  named violation if any is present. Also fixes the completion-record file
  contract (path, gate-name field, exit-status field, run-id field) as a
  documented deliverable so Task-026/Task-027 can build against it
  independently.
- **Difficulty:** Medium — Makefile text is regular enough for a line/token
  scan; the `include` recursion, `$(eval` ban, and the bare-`;` scan (which
  must not flag `for ...; do ... done` or `case ...; esac` shell idiom) need
  care but remain textual.
- **Exit criteria:** TC-130 green; the record-contract note exists and is
  referenced by Task-026 and Task-027.

#### A2: Task-027 — declared/executed reconciliation
- **Scope:** After the entry point invokes Make (once Task-025's environment
  control and Task-026's completion records exist), read the
  completion-record set, compare it to the declared `ci:` prerequisite list
  parsed from the Makefile (15 entries), and report every mismatch by name.
  Treat a missing/backdated record as absent (AC-5).
- **Difficulty:** Medium — the comparison itself is a set-difference; the
  care is in record-freshness (AC-5) and in parsing the declared list once,
  the same way Task-024 already parses the Makefile for AC-1.
- **Exit criteria:** TC-133, TC-134 green.

#### Gate: Task-029 — integration
- **Measures:** The assembled entry point (Task-024 static check + Task-025
  environment check + Task-026 real completion records + Task-027
  reconciliation) against the tracked reproduction and the clean-pass
  control, run together rather than unit-by-unit.
- **Pass criteria:** TC-135 and TC-136 both green; a real `make ci` run
  through the entry point on the unmodified repository at HEAD passes with
  no violation (or, if it does not, fails at the exact same pre-existing gate
  a bare `make ci` independently fails at, for a reason unrelated to this
  plan's own code — matching how tl-rewrite's own Task-006 accepted a
  pre-existing environment-specific failure as sufficient evidence rather
  than blocking on an unrelated defect).
- **If fails:** Do not proceed to Task-030. Bisect which of Task-024/025/026/
  027 the failure traces to before re-attempting the gate.

#### A3: Task-030 — review and matrix closure
- **Scope:** Run this repository's spec-driven review step
  (code-review/gap-analysis) against the finished implementation, fix
  findings, re-run the Task-029 gate to confirm still green, then add
  TC-130 through TC-137 to `spec/test-matrix.md` marked implemented and mark
  this plan's NFR-006 checklist complete.
- **Difficulty:** Easy-Medium, contingent on review findings.
- **Exit criteria:** the CI entry point exits 0 (or fails only at a named
  pre-existing, unrelated gate as above) at the merge revision;
  `spec/test-matrix.md` carries the new rows; NFR-006's Acceptance Criteria
  checklist above is fully checked.

### Track B: Parallel (independent agent, can start once Task-024 lands)

#### B1: Task-025 — invocation-environment control
- **Scope:** Add the `MAKEFLAGS`-sanitizing/explicit-flag-set invocation
  logic to the entry point built in Task-024, including the bundled
  short-flag and wide-boundary `|| true` recognition specified in AC-2 from
  the start.
- **Difficulty:** Medium — least-precedented mechanism in this plan; no
  existing repository code does anything like it. Isolated in its own task
  for exactly that reason.
- **Exit criteria:** TC-131 green.

#### B2: Task-026 — per-gate completion records
- **Scope:** Touch all 15 `ci` prerequisite recipes in the Makefile so each
  writes a completion record, matching Task-024's contract, only on its own
  successful completion. Use one shared, mechanical pattern (each recipe
  appends a call to the compiled `ci_guard record <gate>` binary as its
  final recipe line, never joined to the preceding check with a bare `;`)
  rather than 15 independently hand-varied edits, to keep the diff
  reviewable and avoid accidentally changing a gate's actual behavior while
  only intending to add a record.
- **Difficulty:** Medium — mechanically simple per recipe, but widest blast
  radius in this plan (touches every gate's own recipe line, though not its
  own command). Correctness of each gate's own recipe stays out of NFR-006's
  ownership; this task only adds a record write, and must not change any
  recipe's actual command.
- **Exit criteria:** TC-132 green.

### Track C: Post-Task-024, independent of Track B

#### C1: Task-028 — documentation and workflow wiring
- **Scope:** Update README.md's and CLAUDE.md's `make ci` references, and
  `.github/workflows/ci.yml`'s `run: make ci` dispatch step, to name the
  compiled entry point instead (`cargo run --quiet --bin ci_guard -- ci`, or
  a `make guarded-ci` convenience wrapper around the same invocation,
  matching tl-rewrite's own `make guarded-ci` alias — decide and document
  the exact invocation form during this task).
- **Difficulty:** Easy.
- **Exit criteria:** TC-137 inspection passes (AC-8).

## Parallel Execution Summary

```text
Task-024 ─┬─ Task-025 ─────────┐
          ├─ Task-026 ─ Task-027┤
          └─ Task-028 ──────────┴─ Task-029 (gate) ─ Task-030
```
Task-025, Task-026, and Task-028 can run on independent agents once
Task-024 merges; Task-027 needs Task-026's real completion records to test
against, so it starts after Task-026 (not purely parallel to it) even though
neither blocks Task-025 or Task-028.

## Task File Mapping

| Task | Track | Requirement(s) | Depends on | Status |
|---|---|---|---|---|
| Task-024 | A | NFR-006-AC-1 | — | done |
| Task-025 | B | NFR-006-AC-2 | Task-024 | done |
| Task-026 | B | NFR-006-AC-3 | Task-024 | done |
| Task-027 | A | NFR-006-AC-4, AC-5 | Task-024, Task-026 | done |
| Task-028 | C | NFR-006-AC-8 | Task-024 | done |
| Task-029 | A (Gate) | NFR-006-AC-6, AC-7 | Task-025, Task-027, Task-028 | done |
| Task-030 | A | NFR-006 (closure) | Task-029 | done |

## Coordination Rules

- Task-024 is a hard single-writer prerequisite for everything else in this
  plan: no other task starts until Task-024's completion-record contract is
  fixed, to avoid Task-026 and Task-027 diverging on record shape.
- `Makefile` is shared mutable state between Task-026 (adds record writes to
  all 15 recipes) and Task-028 (edits header/doc references) and, upstream,
  every other requirement's own gate work. Task-026 and Task-028 should not
  run as literally concurrent edits to `Makefile`/`README.md`/`CLAUDE.md`
  without rebasing serially; they are marked as different tracks for
  *review* independence, not for simultaneous unreviewed edits to the same
  files.
- Do not start Task-029 until Task-025, Task-027, and Task-028 are each
  individually green — the gate is a joint property, not something any one
  of them can pass alone.
- No task in this plan touches `scripts/*.py`, `assurance/`, or `NFR-003`'s
  owned gates; if a task's implementation seems to need that, stop and
  re-check against NFR-006's Scope section before proceeding.
- Hosted CI remains manual-only (`workflow_dispatch`) and is not dispatched
  by this plan.
