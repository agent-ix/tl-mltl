---
id: Task-030
title: "Review and matrix closure"
type: Task
status: done
track: A
priority: P0
owner_repository: agent-ix/tl-mltl
consumer_repositories: [agent-ix/tl-mltl]
evidence_method: inspection
github_issue: ix://agent-ix/tl-mltl/issues/14
relationships:
  - target: ix://agent-ix/tl-mltl/issues/14
    type: depends_on
  - target: ix://agent-ix/tl-mltl/Task-029
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: references
---

# Task-030: Review and matrix closure

## Scope

Run this repository's spec-driven review step (code-review + gap-analysis)
against the finished PLAN-009 implementation, fix findings directly, re-run
the Task-029 integration gate to confirm still green, then close the loop:

- Add TC-130 through TC-137 to `spec/test-matrix.md`, marked implemented,
  with their owning FR/NFR AC references.
- Mark this plan's NFR-006 Acceptance Criteria checklist (`plan.md`
  Requirements Summary) fully checked.
- Update `plan.md`'s frontmatter `status` and each task's `status` to
  `done`.
- Post a Linear comment on TL-65 recording the review findings and their
  dispositions (and delete any standalone review-report `.md` file used to
  draft that comment — the `spec/reviews/SR-XXX` files this repository's
  own spec-review convention commits are kept; only a throwaway
  drafting file is removed).

## Subtasks

- [ ] Run code-review (Rust-idiom, mock-boundary, and repo-convention
  checks) against `src/ci_guard.rs`, `src/bin/ci_guard.rs`,
  `tests/ci_guard.rs`, and the Makefile diff.
- [ ] Run gap-analysis: every NFR-006-AC backed by a real tracking tag in a
  real test; no code introduced with no owning requirement.
- [ ] Fix every real finding; re-run `make guarded-ci` (or the equivalent
  direct `cargo run --bin ci_guard -- ci` invocation before the Makefile
  target lands) to confirm still green after fixes.
- [ ] Add `spec/test-matrix.md` rows for TC-130 through TC-137.
- [ ] Update `plan.md` and every `Task-024`..`Task-030` frontmatter `status`.

## Deliverables

- `spec/test-matrix.md` rows for TC-130–TC-137, marked implemented.
- `plan.md` with every AC checked and `status: done`.
- Linear TL-65 comment recording the review pass.

## Notes

- Do not mark this task (or the plan) done while any Task-029 sub-case is
  red, or while any code-review/gap-analysis finding remains unfixed with a
  real functional impact.
