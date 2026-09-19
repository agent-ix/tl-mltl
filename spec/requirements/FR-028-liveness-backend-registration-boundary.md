---
id: FR-028
title: Route liveness-backend registration to the provider crate
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-290
    type: references
---

# FR-028: Route liveness-backend registration to the provider crate

## Description

Where a `tl-syntax.formula-unbounded/v1` document is handed to settlement,
`tl-live` (FR-027), not tl-mltl, shall register against tl-syntax's
`tl-syntax.liveness/v1` capability (tl-syntax
[FR-290](https://github.com/agent-ix/tl-syntax/blob/main/spec/requirements/FR-290-liveness-capability-registration.md))
and shall return the proved, refuted, inconclusive, or failed disposition FR-290
routes to a registered backend. tl-mltl shall register for no capability under
`tl-syntax.liveness/v1` and shall not consult it from any FR-001 through FR-019
entry point.

## Inputs

- The `tl-syntax.liveness/v1` capability identity tl-syntax FR-290 names.
- A `tl-syntax.formula-unbounded/v1` document (tl-syntax FR-289), which reaches
  `tl-live` directly and never passes through a tl-mltl entry point.

## Outputs

- `tl-live`'s registered disposition (proved, refuted, inconclusive, or
  failed) once it registers, or the FR-290 `unsupported` absence settlement
  before it does; tl-mltl produces neither, since it never registers and never
  receives the document.
- No tl-mltl output, verdict, or CLI report changes shape, gains a field, or
  gains a code path as a result of this requirement.

## Behavior

`tl-syntax.liveness/v1` is a single capability identity; exactly one component
in the tl-syntax/tl-mltl/tl-live dependency graph registers against it for a
given deployment, and this requirement names that component as `tl-live`, not
tl-mltl. This keeps the FR-290 absence path meaningful: until `tl-live`
registers, every `tl-syntax.formula-unbounded/v1` document settles
`unsupported` with a warning naming `tl-syntax.liveness/v1`, exactly as FR-290
already specifies, and that absence is never mistaken for a tl-mltl gap because
tl-mltl was never in that path.

tl-mltl's own bounded evaluator is a distinct, already-registered consumer of
`tl-syntax.formula/v1` under FR-001; it has no analogous registration step and
this requirement adds none. A caller that wants both bounded reference
evaluation and infinite-trace liveness analysis composes tl-mltl and `tl-live`
as two independent crates against the same `tl-syntax.formula-unbounded/v1`
graph; neither wraps or re-exports the other.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-028-AC-1 | `spec/spec.md` and this requirement name `tl-live`, not tl-mltl, as the `tl-syntax.liveness/v1` registrant. | Inspection (TC-087) |
| FR-028-AC-2 | No tl-mltl source registers against, imports a registration type for, or otherwise references `tl-syntax.liveness/v1` at a call site. | Test (TC-087) |
| FR-028-AC-3 | Every existing tl-mltl verdict, report, and CLI schema is byte-identical to its pre-existing shape; this requirement adds no field, variant, or code path to any of them. | Test (TC-087) |

## Dependencies

Depends on FR-027's crate-boundary statement. References tl-syntax FR-290's
registration and absence-path contract, which this requirement routes to
`tl-live` rather than duplicating. FR-029 supplies the dependency order this
registration follows.
