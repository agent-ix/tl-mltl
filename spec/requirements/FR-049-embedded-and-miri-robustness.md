---
id: FR-049
title: Check actual embedded and Miri support boundaries
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-027
    type: references
---

# FR-049: Check actual embedded and Miri support boundaries

## Description

When the robustness lane runs, it shall build tl-syntax for a named embedded
`no_std` target and run applicable Rust code under Miri and limit-edge tests.

## Behavior

tl-syntax's declared `#![no_std]` surface and feature selection are checked
on the exact target triple. tl-parse, tl-rewrite and tl-mltl remain std-only;
a successful tl-syntax build is never projected onto them. Miri exercises
applicable unsafe and ownership-sensitive paths under recorded toolchain and
flags. Every public operation receives at-limit and one-over-limit inputs and
asserts a typed outcome without panic or integer wrap. Unsupported Miri paths
are counted explicitly, not silently skipped.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-049-AC-1 | The named tl-syntax embedded target builds with `no_std` and the other crates make no unsupported embedded claim. | Test (TC-187) |
| FR-049-AC-2 | Miri and limit/error-variant probes return recorded typed outcomes, including every one-over boundary without panic. | Test (TC-188) |

## Dependencies

FR-027 records the existing std-only tl-mltl boundary.
