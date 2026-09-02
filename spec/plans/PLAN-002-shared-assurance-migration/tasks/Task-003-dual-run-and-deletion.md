---
id: Task-003
title: "Dual run, deletion, and residual"
type: Task
status: done
track: Landing
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-mltl/NFR-003
    type: references
---
# Task-003: Dual run, deletion, and residual

## Scope

Run the old and new paths against the same candidate revision, record the result
as observed rather than as parity, delete the local evidence framework in a
separate commit afterwards, measure what removing the Make execution-control
guard costs, and file the residual.

## Completion Evidence

Both paths were run at one revision with both present. The old path's result is
recorded as observed: it fails, and the reason is recorded rather than worked
around.

The deletion is a separate final commit, so both paths coexist in history up to
the revision the dual run names.

The Make execution-control residue is measured in this repository — not inherited
from a sibling — and recorded in four places: `NFR-003`, the `AA-001` challenge
list, the `UNKNOWN-make-failure-propagation-guard-removed` entry in
`assurance/change-assurance.json`, and a header comment in the `Makefile`. It is
tracked as `agent-ix/tl-mltl#14` and is not claimed to be closed by the
structural replacement.
