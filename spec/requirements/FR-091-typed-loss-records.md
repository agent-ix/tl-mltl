---
id: FR-091
title: "Return a typed loss record for every construct an import or export cannot carry"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-046
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-010
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-017
    type: depends_on
---

# FR-091: Return a typed loss record for every construct an import or export cannot carry

## Description

If an import or export meets a construct it cannot carry exactly,
then tl-mltl shall return a `LossRecord` naming the kind, format, source span and
construct, and never drop, default or approximate it silently.

## Behavior

`LossKind` is closed with 10 members: `UnsupportedOperator`, `AmbiguousNext`,
`UnsupportedSignalType`, `UnsupportedInterval`, `UnmappedVariable`,
`DuplicateVariable`, `UnknownHeaderField`, `MissionClosureRequired`,
`ForeignConstruct`, `ResourceLimit`. A multi-requirement file is imported
requirement by requirement (the C2PO reader's per-requirement entry point, FR-086,
whose file-level refusals abort the file instead): the result lists `imported` and `unimported` entries
and `len(imported) + len(unimported)` equals the requirement count the file
declares (conservation). A record with no span states why. Records serialize to
`tl-mltl.loss-record/v1`. No loss record is a temporal verdict.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-091-AC-1 | One fixture per `LossKind` (10) produces exactly that kind with format, construct and span, and the exhaustive match over `LossKind` fails to compile if a member is added without a fixture. | Test (TC-320) |
| FR-091-AC-2 | For every multi-requirement fixture, imported plus unimported equals the declared requirement count, and no requirement vanishes. | Test (TC-321) |

## Dependencies

FR-010 and FR-017 own existing explicit-disposition and refusal behavior that
this requirement generalizes to the new formats.
