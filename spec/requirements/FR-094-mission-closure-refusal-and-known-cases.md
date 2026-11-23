---
id: FR-094
title: "Refuse implicit mission-bound closure, report the known meaning-changing cases and define the closure API"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-045
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-083
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-091
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-040
    type: depends_on
---

# FR-094: Refuse implicit mission-bound closure, report the known meaning-changing cases and define the closure API

## Description

If any import, closure or evaluation path would turn an unbounded formula into a
bounded one, then tl-mltl shall refuse with `MissionClosureRequired` and emit no
artifact.

## Behavior

**Refusal.** The refusal applies to every path that would return a bounded
formula derived from an unbounded one: the importers (a foreign unbounded
operator is a `LossRecord` of kind `MissionClosureRequired`, FR-091), the closure
API below and any evaluator entry point asked to evaluate an unbounded formula as
bounded. It does not apply to `export_safety_monitor`
(`src/infinite/export.rs`), which keeps its behavior and its own error variants
(ADR-045 item 2): it accepts only `G[0,)psi` with a finite-horizon `psi`, exports
the body as a `refutation_only` expression, and refuses every other unbounded
shape with `UnboundedLiveness`, `UnboundedUntilRelease` or `UnsupportedShape`. No
other code path may emit a closed form of an unbounded formula.

**Closure API (requirement, built regardless of the open decision).**
`mission_closure_report(formula, mission_time)` takes a validated LTLf or
infinite-trace formula and a checked `u32` mission time and returns, per unbounded
temporal node (`F G U R Xs Xw` for LTLf; `F G U R` unbounded for infinite-trace), the
assumed closure (`G` to `G[0,M]`, `F` to `F[0,M]`, `U` to `U[0,M]`, `R` to
`R[0,M]`, `Xs` to `F[1,1]`, `Xw` to `G[1,1]`; next closures do not depend on `M`),
the matching row of the known-case table and the statement that the meaning may
change. For an infinite-trace source the row is `not-computed` (the table compares
LTLf with closed-trace only). It converts nothing, returns no bounded formula and
needs no acknowledgement. `close_mission(formula, mission_time, policy)` exists only
if Peter decides the conversion is built (ADR-045 Open decision); `policy` is
required, has no default and the only converting variant is
`ClosurePolicy::ChangeMeaning { mission_time }` with an equal mission time.

**Known-case table, computed not asserted.** For each depth-one temporal formula over
two atoms (`F`, `G` on the 4 atom and constant operands = 8; `U`, `R` on the 16
ordered operand pairs = 32; 40 formulas), each mission time `M` in `{1, 2}` (80
closures), and each of the 340 traces of length 1 to 4, the LTLf verdict is
compared with the closed-trace verdict of the bounded closure `[0,M]` (27,200
comparisons). For `Xs` and `Xw` on the same 4 operands (8 formulas), closed to
`F[1,1]` and `G[1,1]`, each of the 340 traces (2,720 comparisons, no `M`). In all
29,920 comparisons. The table records per operator and `M` whether the closure
agreed on all, some or no traces, and is the reference for FR-090's citations.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-094-AC-1 | A foreign unbounded operator in each importer and a call to the closure API with a conversion request are refused with `MissionClosureRequired` and no artifact, and `export_safety_monitor` still accepts `G[0,)psi` and still returns its own variant for each other unbounded shape. | Test (TC-327) |
| FR-094-AC-2 | The known-case table over 29,920 comparisons matches an independently written enumerator, at least one operator is classified as changing meaning (`G` on a trace shorter than `M+1`), and both `Xs` and `Xw` are classified as changing meaning at the last position. | Test (TC-328) |
| FR-094-AC-3 | `mission_closure_report` on a formula holding `F`, `G`, `U`, `R`, `Xs` and `Xw` nodes returns one row per node with its assumed closure and table row, returns `not-computed` rows for an infinite-trace source, refuses an overflowing mission time, and returns no bounded formula. | Test (TC-329) |
| FR-094-AC-4 | Conditional on ADR-045 Open decision (conversion built): `close_mission` refuses a missing policy and a policy whose mission time differs from the request with `MissionClosureRequired`, and a matching policy returns the bounded formula with the provenance block of ADR-045 item 4. Withdrawn if the conversion is not built. | Test (TC-333) |

## Dependencies

FR-083 supplies the LTLf side; ADR-045 owns the policy; FR-040 owns the retained
safety exporter this requirement leaves unchanged.
