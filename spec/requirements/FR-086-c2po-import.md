---
id: FR-086
title: "Import C2PO specifications so that export then import is the identity on the supported subset"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-046
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-038
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-041
    type: depends_on
---

# FR-086: Import C2PO specifications so that export then import is the identity on the supported subset

## Description

When C2PO text is imported with a caller-selected semantic profile, tl-mltl shall
reconstruct the validated graph and proposition map that the export mapping
would have rendered it from, or refuse with a typed error naming the construct
outside the subset.

## Inputs

- C2PO text, the signal map, an explicit profile (`mltl.online-prefix/v1` for
  `FTSPEC`, `mltl.origin-complete-history/v1` for `PTSPEC`), limits.

## Behavior

The supported subset is the closed set of forms the FR-038/FR-041 mapped cells
render: `INPUT` declarations of Boolean signals, `FTSPEC` and `PTSPEC` sections,
Boolean operators `!`, `&&`, `||`, `->`, `<->`, bounded `F G U R` and past
`O H Y S T` with closed intervals. Identity means: for every formula `g` and
map in the mapped cells, `import(export(g)) = g` by canonical graph identity,
proposition map and signal order. The profile is never inferred from the
section name alone; a mismatch with the selection is refused. The following
constructs are refused by the typed error
`C2poImportError::UnsupportedConstruct { construct, span }`, one variant each
(10): struct declaration, DEFINE macro, arithmetic operator, relational
operator, function call, set or array aggregation, non-Boolean input type,
CONTRACT section, ORDER section, unrecognized token. (This construct list is not
checked against the C2PO grammar, which the retained corpus records as C2PO tool
version 4.1.0 shipped with R2U2 4.2-release
(`corpus/r2u2-v4.2/manifest.json`); the build ticket shall reconcile it with that
grammar and may split but not drop entries.) The refusal emits no partial graph.

**Refusal model (one model, two entry points).** `import_c2po` is the strict entry
point used for export-then-import identity: a construct outside the subset anywhere
in the text aborts the whole import with `UnsupportedConstruct`. The per-requirement
entry point, used by the FRET bundle import (FR-088) and by FR-091, imports each
requirement (one `;`-terminated formula of a spec section) independently. There the
10 constructs split by scope. File-level constructs (struct declaration, DEFINE
macro, non-Boolean input type, CONTRACT section, ORDER section, and an unrecognized
token that stops requirement boundaries from being found) abort the file with
`UnsupportedConstruct`, because they can change the meaning of any requirement.
Expression-level constructs (arithmetic operator, relational operator, function
call, set or array aggregation, and an unrecognized token inside one expression)
yield a `LossRecord` for that requirement only, mapped to the `LossKind` of FR-091
(`UnsupportedOperator` for the first four, `ForeignConstruct` for the rest), while
the other requirements are imported; `len(imported) + len(unimported)` equals the
declared requirement count (FR-091 conservation).

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-086-AC-1 | For the 11 distinct retained formulas (6 in `corpus/r2u2-v4.2/formulas.c2po` and 5 distinct among the 6 lines of `corpus/past-c2po-v1/target-4.2/past.c2po`, where `O[1,1](p);` appears twice) export then import returns the identical graph, map and signal order; the duplicate line is imported as two requirements and compared once. | Test (TC-310) |
| FR-086-AC-2 | Each of the 10 construct kinds is refused with its own variant naming the construct and a span, and no graph is emitted. | Test (TC-311) |
| FR-086-AC-3 | A profile mismatch, an absent profile, or a section kind that contradicts the selected profile is refused and the profile is never inferred. | Test (TC-312) |
| FR-086-AC-4 | In the per-requirement entry point, a text with one expression-level unsupported construct among several requirements imports the others, returns one loss record for that requirement, and conserves the requirement count; a file-level construct aborts the file with `UnsupportedConstruct` and no graph. | Test (TC-332) |

## Dependencies

FR-038 and FR-041 own the export subset and refusal partition.
