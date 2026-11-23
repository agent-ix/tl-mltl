---
id: FR-089
title: "Attach upstream attribution to every imported FRET and WEST artifact"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-046
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-004
    type: depends_on
---

# FR-089: Attach upstream attribution to every imported FRET and WEST artifact

## Description

When a published FRET or WEST example is imported or retained, tl-mltl shall
carry its upstream repository, path, revision and licence identifier into the
import result and keep its licence file next to the retained bytes.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-089-AC-1 | Every imported FRET example result carries upstream repository, path, revision and licence, and an example lacking any of the four is refused before import. | Test (TC-244) |
| FR-089-AC-2 | Every retained foreign fixture directory contains its licence file and a manifest digest that matches the bytes. | Test (TC-244) |

## Dependencies

NFR-004 for reproducible retained corpora.
