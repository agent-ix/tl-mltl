---
id: FR-092
title: "Import and export the WEST formula format and round-trip the tl-rewrite corpus subset"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-046
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-091
    type: depends_on
---

# FR-092: Import and export the WEST formula format and round-trip the tl-rewrite corpus subset

## Description

When a WEST formula line is read or a closed-trace graph is written, tl-mltl
shall convert between them, so that reading then writing the retained WEST corpus
reproduces the source bytes.

## Behavior

Grammar, as observed in the retained corpus: atoms `p<N>` bound to proposition
`N`; `T` true; `!` false; `~` not; `&` and; `v` or; infix `U` and `R` with
`[a:b]`; prefix `G[a:b]` and `F[a:b]`; parentheses around binary forms. Graphs
are under `mltl.closed-trace/v1`. Export refuses, with `UnsupportedOperator`,
`Implies`, `Equivalent`, past nodes, unbounded or non-closed intervals and
`W`/`M`; it does not rewrite them. The retained corpus is the 20-line
`formulas_d1.txt` (MIT, from `zwang271/WEST`, upstream revision as recorded by
tl-rewrite) with the 10 cases `selectedCases` already used by tl-rewrite; bytes
are copied unchanged with the licence file and a SHA-256 that a test compares to
tl-rewrite's copy.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-092-AC-1 | For the 10 tl-rewrite-selected cases, read then write returns the identical line and the graph equals the hand-authored graph in the manifest. | Test (TC-249) |
| FR-092-AC-2 | All 20 lines of `formulas_d1.txt` read then write byte-identically. | Test (TC-250) |
| FR-092-AC-3 | Each of the five unexportable classes (`Implies`, `Equivalent`, past, non-closed interval, `W`/`M`) is refused with a loss record and no text. | Test (TC-251) |
| FR-092-AC-4 | The retained corpus bytes and licence file match their recorded SHA-256 and the tl-rewrite copy. | Test (TC-252) |

## Dependencies

FR-091; tl-rewrite owns `corpus/west-v1` upstream of the copy.
