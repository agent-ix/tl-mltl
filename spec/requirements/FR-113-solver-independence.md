---
id: FR-113
title: "Keep tl-mltl independent of the solver crate"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-074
    type: depends_on
  - target: ix://agent-ix/tl-solve/ADR-004
    type: depends_on
---

# FR-113: Keep tl-mltl independent of the solver crate

## Description

The tl-mltl crate shall not depend on tl-solve or on any SAT solver crate,
under any feature set, so that tl-solve's answers are re-checked by an
evaluator that cannot reach the solver path.

## Behavior

- With default features, with `infinite-trace`, and with all features, the
  resolved normal, build and dev dependency graph of tl-mltl contains no
  `tl-solve`, `rustsat`, `rustsat-batsat`, `batsat` or `varisat` package.
- No tl-mltl source file imports a `tl_solve` path, and `deny.toml`'s `allow-git`
  names no tl-solve repository.
- The public API, wire bytes and dependency closure of tl-mltl are unchanged by
  the existence of tl-solve, apart from the exact pins that already move with
  each release (ADR-003).

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-113-AC-1 | `cargo tree` for the default, `infinite-trace` and all-features sets lists none of `tl-solve`, `rustsat`, `rustsat-batsat`, `batsat` or `varisat`, and a test fails when one is added. | integration-testing (TC-257) |
| FR-113-AC-2 | A source search finds no `tl_solve` import in `src`, `tests`, `fuzz` or `tools`, and `deny.toml`'s `allow-git` list contains no tl-solve entry. | integration-testing (TC-258) |

## Dependencies

Follows ADR-003 and ADR-074; tl-solve ADR-004 records the dependency order.
