---
id: TM-076
title: Solver-independence test matrix
type: TestMatrix
relationships:
  - target: ix://agent-ix/tl-mltl/FR-113
    type: covers
---

# Solver-independence test matrix

## Functional Requirement Coverage

| Functional Req | Acceptance Criteria | Test Cases | Status |
|---|---|---|---|
| FR-113 | FR-113-AC-1, FR-113-AC-2 | TC-257, TC-258 | 🚧 planned |

## Test Case Summary

| Test ID | Title | Type | Priority | Traces To | Status |
|---|---|---|---|---|---|
| TC-257 | The resolved dependency graph of tl-mltl under default, `infinite-trace` and all features contains no tl-solve or SAT solver package | Integration | P1 | FR-113-AC-1 | 🚧 planned, deferred verification (needs `cargo tree`) |
| TC-258 | No source file imports tl-solve and `deny.toml` names no tl-solve repository | Integration | P1 | FR-113-AC-2 | 🚧 planned, deferred verification (needs a test run) |
