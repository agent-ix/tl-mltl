---
id: FR-047
title: Measure mutation detection on a stable baseline
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-045
    type: depends_on
---

# FR-047: Measure mutation detection on a stable baseline

## Description

When the mutation lane runs, each owning crate shall measure how many viable
semantic mutants its real tests detect from a stable green source revision.

## Behavior

The lane records the discovered, selected, unviable, caught, missed, timed-out
and not-run populations. It uses one fixed test selection and an isolated
source copy per mutant, restores the source and verifies the green control.
Only a completed viable population earns a kill rate; its denominator and
scope accompany the ratio. A missed mutant gets a concrete counterexample,
proof candidate or reviewed limitation, never a silent exclusion. The target
is at least 90% caught among viable mutants in the selected critical semantic
modules; failure leaves the milestone open, not a changed denominator.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-047-AC-1 | Discovery, selection, execution and score populations reconcile on a green fixed baseline and the stated critical scope meets the target. | Test (TC-183) |
| FR-047-AC-2 | Seeded surviving and timed-out mutants remain visible with reviewed disposition; changed test selection or dirty restoration invalidates the run. | Test (TC-184) |

## Dependencies

FR-045 supplies real tests whose fault detection is measured.
