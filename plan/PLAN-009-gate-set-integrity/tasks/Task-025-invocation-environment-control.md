---
id: Task-025
title: "Invocation-environment control"
type: Task
status: done
track: B
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: unit-test
github_issue: ix://agent-ix/tl-mltl/issues/14
relationships:
  - target: ix://agent-ix/tl-mltl/issues/14
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-024
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
  - target: ix://agent-ix/tl-mltl/TC-131
    type: verifies
---

# Task-025: Invocation-environment control

## Scope

Add the `MAKEFLAGS`-sanitizing / explicit-flag-set invocation logic to the
entry point built in Task-024: the entry point never forwards an inherited
`MAKEFLAGS`/`MFLAGS` to the Make process it starts, invokes Make with an
explicit minimal flag set, and refuses outright before invoking Make if the
calling environment's `MAKEFLAGS` carries an `-i`/`-k`/`-S`-equivalent value.

**Both known-bypass fixes go in from the start here, not as follow-up
findings:**

1. `dangerous_makeflags` must recognize GNU Make's own flag-bundling: a
   single re-exported token can carry `-i` without an exact bare `i` word
   (e.g. `-ik`, or the space-free bundled form `ik` Make itself uses when
   re-exporting `MAKEFLAGS` to a sub-make). Match on any alphabetic,
   non-`=`-containing, non-`-`-prefixed token that contains `i` or `k`, not
   just an exact `-i`/`-k`/`--ignore-errors`/`--keep-going` token. `-S`/
   `--no-keep-going` (which *cancels* `-k`) must never itself be flagged.
2. The `|| true` detector (Task-024's static scan) already uses a substring
   match rather than an end-of-line/whitespace-anchored one — verify this
   explicitly with a case that would fail a narrow boundary check:
   `cmd || true; more` (trailing semicolon and further commands after the
   `|| true`). Add this as its own regression case in this task rather than
   assuming Task-024's implementation already covers it silently.

## Subtasks

- [ ] Implement `dangerous_makeflags(value: &str) -> Option<String>` with
  the bundled-flag recognition above.
- [ ] Wire `src/bin/ci_guard.rs`'s `ci` subcommand: read `MAKEFLAGS` from the
  environment, refuse via `dangerous_makeflags` before invoking Make.
- [ ] When invoking Make (once Task-027 adds the actual `Command::new("make")`
  call), always `env_remove("MAKEFLAGS")` and `env_remove("MFLAGS")` on the
  child process — defense in depth even though the pre-flight check above
  should already have refused a dangerous one.
- [ ] Add the `cmd || true; more` regression test against Task-024's
  `scan_recipe_line`/`RecipeSwallowsFailure` path.

## Deliverables

- `dangerous_makeflags` with unit tests (TC-131): `-i`, `-k`,
  `--ignore-errors`, `--keep-going` detected; a bundled token (`ik`, `wik`)
  detected; `-S`/`--no-keep-going` explicitly NOT flagged; clean flags
  (`""`, `"w"`, `"--no-print-directory"`) not flagged; the `cmd || true;
  more` wide-boundary regression case passes against Task-024's scan.

## Notes

- SR-053/precedent note: this is the least-precedented mechanism in this
  plan — no existing repository code does anything like reading and vetting
  `MAKEFLAGS` before invoking a child process — so it is isolated in its own
  task and should be tested against real `make -p`/re-exported `MAKEFLAGS`
  behavior, not only synthetic strings, before Task-029's integration gate.
