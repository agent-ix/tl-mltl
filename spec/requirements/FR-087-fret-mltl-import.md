---
id: FR-087
title: "Import FRET-exported MLTL formulas"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-046
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-091
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-045
    type: depends_on
---

# FR-087: Import FRET-exported MLTL formulas

## Description

When a FRET-exported MLTL formula file is imported, tl-mltl shall produce one
validated bounded graph per requirement that maps exactly, and one typed loss
record per requirement or construct that does not.

## Behavior

The reader accepts the bounded MLTL operators `G F U R` with closed integer
intervals and the Boolean operators, mapping to `mltl.closed-trace/v1` unless the
caller selects `mltl.online-prefix/v1`; the profile is explicit. A bare next is
`AmbiguousNext`; FRETish text, project files and non-MLTL content are
`ForeignConstruct`. The concrete spellings of FRET's export (operator tokens,
file framing, requirement identifiers) are **ungrounded in this specification**
because no FRET export was available offline; the first build task retrieves the
published FRET examples, records their upstream path, revision and licence in
the fixture manifest, and the reader is written against those bytes, not against
this text. The number of fixtures is whatever the manifest lists, checked by a
directory census at retrieval time.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-087-AC-1 | Every fixture in the FRET example manifest imports to the graphs recorded by hand in the manifest, or to the loss records it declares, and the manifest count equals the directory census. | Test (TC-240) |
| FR-087-AC-2 | A bare next is reported as `AmbiguousNext` and never lowered as strong or weak. | Test (TC-241) |

## Dependencies

FR-091 for loss records; FR-089 for attribution.
