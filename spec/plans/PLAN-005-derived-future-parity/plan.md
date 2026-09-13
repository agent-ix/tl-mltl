---
id: PLAN-005
title: Derived future-operator parity plan
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-mltl/FR-016
    type: references
  - target: ix://agent-ix/tl-mltl/StR-001
    type: references
  - target: ix://agent-ix/tl-syntax/FR-008
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-010
    type: depends_on
---

# PLAN-005: Derived future-operator parity plan

## Objective

Implement `agent-ix/tl-mltl#47`: show that bounded W/M evaluation, prefix
progress, horizon analysis, and resource outcomes computed on the tl-syntax
canonical lowering equal an independent direct reference under both semantic
profiles, with mutation controls, and without adding any derived evaluator
branch.

## Base and landing

Work starts from tl-mltl main `4bff387`. Every predecessor has landed:
tl-syntax#40 as main `8dc18eec5af227f484170362c9e8894b8531a27d`, tl-syntax#32
closed, and tl-parse#31 as `9ca856b`. tl-rewrite#35 proceeds independently and
is not touched here. The change lands through one reviewed PR that references,
and does not close, #47.

## Dependency DAG

```text
tl-syntax#40 landed at 8dc18eec
  -> exact tl-syntax pin, TL_SYNTAX_REVISION, lockfile and pin guard
    -> FR-016, TC-076..TC-080, PLAN-005
      -> tests/future_parity.rs direct reference and parity sweeps
        -> mutation and no-derived-branch controls
          -> exact-head make ci
            -> PR-time rust review and gap analysis
              -> fixes, exact-head make ci, mergeable comment
```

## Task File Mapping

| Task | Scope | Exit evidence |
|---|---|---|
| Task-001 | Pin advance, FR-016, matrix rows, direct reference, closed/prefix/progress/horizon/resource parity, mutation and no-branch controls | TC-076 through TC-080 run under `make test` and `make test-census` |
| Task-002 | Exact-head local gate, PR, PR-time review, remediation, re-gate | `make ci` exit 0 at the pushed head and resolved review findings |

## Implementation shape

- Consume `tl_syntax::FutureLoweringRequest` directly in an integration test.
  The library, CLI, producers, and wire forms gain no W/M vocabulary.
- The direct reference is test-only and written in first-occurrence form, so it
  shares no structure with the `Or(U, G)` / `And(R, F)` expansion it checks.
- Open-prefix parity uses the exact continuation verdict for negation-free
  operands, computed from the all-false and all-true continuations. Closed
  parity additionally covers negated and correlated operands.
- Directly constructed canonical graphs anchor horizon and resource parity;
  hand-built mutant expansions anchor the mutation controls.

## Verification method

`tests/future_parity.rs` sweeps every window within `[0,3]`, every
two-proposition trace up to length 5, and verdict times 0 through 2 for both
kinds and profiles; follows every prefix of pseudo-random length-7 traces for
progress; checks maximum `u32` windows and nesting for horizon; drives
work/recursion/span/time/node-budget boundaries; and requires each of eight
wrong expansions to disagree with the reference. A temporary edit of the direct
reference was confirmed to turn TC-076, TC-077 and TC-080 red before landing.

## Exit Criteria

1. The compiled tl-syntax dependency is the reviewed main revision that owns
   the lowering; the corpus basis does not move.
2. Every FR-016 criterion has a named executing trace symbol.
3. No `src/` file names derived future vocabulary.
4. The exact-head local gate passes and PR-time review findings are fixed.
