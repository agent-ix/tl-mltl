---
id: FR-088
title: "Import FRET's C2PO file, variable map and trace header and bind them to the signal catalog"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-086
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-091
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-007
    type: depends_on
---

# FR-088: Import FRET's C2PO file, variable map and trace header and bind them to the signal catalog

## Description

When a FRET export bundle supplies a C2PO file, a variable map and a trace
header, tl-mltl shall import the C2PO through the per-requirement entry point of FR-086, bind each variable to one
catalog signal, and carry the header fields it understands, with every
unrecognized field recorded.

## Behavior

A variable with no map entry is `UnmappedVariable`; two entries for one variable
or one signal are `DuplicateVariable`; a map entry of non-Boolean type is
`UnsupportedSignalType`; a header field outside the closed recognized set is
`UnknownHeaderField` and does not abort the import. The mission time, when the
header declares one, is passed to FR-090. As in FR-087 the concrete file
layouts are ungrounded here and are fixed by the retrieved fixtures.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-088-AC-1 | Every bundle in the FRET example manifest imports with each variable bound to exactly one catalog signal and the header's recognized fields preserved. | Test (TC-315) |
| FR-088-AC-2 | Unmapped, duplicate, non-Boolean and unknown-header-field cases each yield their loss record and no silently bound signal. | Test (TC-316) |

## Dependencies

FR-086, FR-091, tl-syntax FR-007.
