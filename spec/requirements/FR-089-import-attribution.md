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

When a published FRET or WEST example is imported or retained,
tl-mltl shall carry its upstream repository, path, revision and licence
identifier into the import result and keep its licence file next to the retained
bytes.

## Behavior

The revision is required for FRET examples (the first build task records the
commit it retrieved, FR-087). The retained WEST corpus is the exception that
exists today: tl-rewrite's `corpus/west-v1/manifest.json` records upstream URL,
upstream path, licence identifier, licence file and the formula file, and no
revision. For WEST artifacts the attribution therefore is upstream URL, path,
retained file list, licence identifier and the explicit marker
`revision: UNKNOWN`; the marker is a value, never an absent field, and a revision
is never guessed. Recovering it is DEFERRED. No content hash of a foreign file is
recorded in the manifest or the specification: integrity of a duplicated corpus is
a byte comparison against the owning copy while it exists (FR-092-AC-4), which
avoids a digest that must be re-recorded whenever a file is re-retrieved.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-089-AC-1 | Every imported FRET example result carries upstream repository, path, revision and licence, and an example lacking any of the four is refused before import. | Test (TC-317) |
| FR-089-AC-2 | Every retained foreign fixture directory contains its licence file and a manifest whose retained file list equals the directory census. | Test (TC-317) |
| FR-089-AC-3 | A WEST artifact with upstream URL, path, licence and the marker `revision: UNKNOWN` is accepted, one lacking URL, path, licence or the marker (an absent revision field) is refused, and no revision value is synthesized. | Test (TC-317) |

## Dependencies

NFR-004 for reproducible retained corpora.
