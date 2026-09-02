---
id: Task-001
title: "Inventory, pins, and the upstream repin"
type: Task
status: done
track: Foundation
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-mltl/FR-006
    type: references
---
# Task-001: Inventory, pins, and the upstream repin

## Scope

Produce the keep/replace/delete/defer inventory, declare the adopted release in
`assurance/pins.json`, delegate every version verdict to the packaged
compatibility matrix, and move the compiled tl-syntax pin onto a revision
reachable from that repository's `main` without moving the retained corpus
basis.

## Completion Evidence

`scripts/check_shared_pins.py` classifies four components through
`engineering_assurance.compatibility` and restates no version rule locally. The
hosted workflow's `@agent-ix/quoin@0.22.5` pin — a version the matrix names
explicitly incompatible — is repinned to 0.23.1 and `ix-flow@0.0.4` is added;
`0.22.5` appeared exactly once in the tree and a whole-tree sweep confirms it.
The compiled tl-syntax revision moves from `740182f1`, reachable only from an
open pull request's branch, to `953ee825` on `main`. The corpus basis stays at
`740182f1` because `corpus/tl-syntax-v1` is a byte-identical copy taken there;
`check_shared_pins.py` cross-checks both revisions in the four files that name
them and refuses a tree in which the two have been collapsed into one string.
