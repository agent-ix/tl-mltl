---
id: FR-028
title: Route liveness-backend registration to the infinite-trace provider
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-290
    type: references
---

# FR-028: Route liveness-backend registration to the infinite-trace provider

## Description

Where a `tl-syntax.formula-unbounded/v1` document is handed to settlement, the
infinite-trace provider (FR-027) shall register against tl-syntax's
`tl-syntax.liveness/v1` capability (tl-syntax FR-290, tl-syntax
[#73](https://github.com/agent-ix/tl-syntax/issues/73)) and shall return the
proved, refuted, inconclusive, or failed disposition FR-290 routes to a
registered backend. tl-mltl's own tl-syntax capability surface is the bounded
`tl-syntax.formula/v1` consumption FR-001 already specifies, across every
FR-001 through FR-019 entry point.

## Inputs

- The `tl-syntax.liveness/v1` capability identity tl-syntax FR-290 names.
- A `tl-syntax.formula-unbounded/v1` document (tl-syntax FR-289), which the
  infinite-trace provider consumes directly.

## Outputs

- The registered provider's disposition (proved, refuted, inconclusive, or
  failed); before a provider registers, a claim no registered backend can
  discharge settles `unsupported` with a warning naming the required
  capability, `tl-syntax.liveness/v1`, exactly as tl-syntax FR-290 specifies.
- tl-mltl's verdicts, reports, and CLI schemas keep their existing shape.

## Behavior

`tl-syntax.liveness/v1` is a single capability identity; exactly one component
in the tl-syntax/tl-mltl/provider dependency graph registers against it for a
given deployment, and this requirement allocates that role to the
infinite-trace provider. This keeps the FR-290 absence path meaningful: until
a provider registers, every `tl-syntax.formula-unbounded/v1` document settles
`unsupported` with a warning naming `tl-syntax.liveness/v1`, and that absence
is attributable to the provider boundary rather than to tl-mltl's bounded
evaluator, which is never in that path.

tl-mltl's own bounded evaluator is a distinct, already-registered consumer of
`tl-syntax.formula/v1` under FR-001; it has no analogous registration step and
this requirement adds none. A caller that wants both bounded reference
evaluation and infinite-trace liveness analysis composes tl-mltl and the
provider as two independent crates against the same tl-syntax graphs; neither
wraps or re-exports the other. Which crate carries the provider is an owner
decision, recorded as an open question in [AD-002](../assurance/AD-002.md).

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-028-AC-1 | `spec/spec.md` and this requirement allocate the `tl-syntax.liveness/v1` registration to the infinite-trace provider under the `quire.temporal.infinite-trace/v1` facet, with the provider's crate identity recorded as an open owner decision in AD-002. | Inspection (TC-087) |
| FR-028-AC-2 | Every tl-syntax capability tl-mltl consults at a call site is the `tl-syntax.formula/v1` consumption FR-001 specifies. | Test (TC-087) |
| FR-028-AC-3 | Every existing tl-mltl verdict, report, and CLI schema is byte-identical to its pre-existing shape; this requirement adds no field, variant, or code path to any of them. | Test (TC-087) |

## Dependencies

Depends on FR-027's crate-boundary statement. References tl-syntax FR-290's
registration and absence-path contract, which this requirement allocates to
the infinite-trace provider rather than duplicating. FR-029 supplies the
dependency order this registration follows.
