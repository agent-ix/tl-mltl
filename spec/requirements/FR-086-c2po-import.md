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
checked against the R2U2 4.2 C2PO grammar; the build ticket shall reconcile it
with the grammar and may split but not drop entries.) The refusal emits no
partial graph.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-086-AC-1 | For the 12 retained formulas (6 in `corpus/r2u2-v4.2/formulas.c2po` and 6 in `corpus/past-c2po-v1/target-4.2/past.c2po`) export then import returns the identical graph, map and signal order. | Test (TC-237) |
| FR-086-AC-2 | Each of the 10 construct kinds is refused with its own variant naming the construct and a span, and no graph is emitted. | Test (TC-238) |
| FR-086-AC-3 | A profile mismatch, an absent profile, or a section kind that contradicts the selected profile is refused and the profile is never inferred. | Test (TC-239) |

## Dependencies

FR-038 and FR-041 own the export subset and refusal partition.
