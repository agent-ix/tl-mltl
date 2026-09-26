---
id: FR-038
title: Export admitted past-time formulas to C2PO
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-004
    type: extends
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
---

# FR-038: Export admitted past-time formulas to C2PO

## Description

When a complete origin-bound past formula has a C2PO representation with the
same per-step meaning, tl-mltl shall export H, O, S and Y, and T by an
explicit dual construction if no direct C2PO form is available.

## Behavior

A distinct past-profile mapping entry point consumes the validated formula-v2
graph, event-position clock, signal catalog and origin contract. It emits a
new versioned manifest and C2PO expression without changing v1/v2 bounded
future mapping bytes. Export is admitted only when every node and interval
has an exact representation, including Y at position zero and any bounded
past interval lower/upper limits. T uses `not (not p S not q)` only when the
selected target's negation and S behavior support that equivalence under the
same origin contract. Unsupported target syntax, profile or origin semantics
produces a typed refusal and no expression or manifest.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-038-AC-1 | Admitted H/O/S/Y and guarded T exports have exact parsed C2PO forms and preserve graph, clock and source identities. | Test (TC-160, TC-161) |
| FR-038-AC-2 | Unsupported node/interval/target combinations refuse without partial expression; legacy future manifests remain byte-identical. | Test (TC-162, TC-163) |
| FR-038-AC-3 | Per-step target observations are compared with the independent tl-mltl past evaluator on the pinned corpus, without treating replay as a target runtime run. | Test (TC-164) |

## Dependencies

FR-004 owns existing future mapping; FR-027 keeps infinite input separate.
