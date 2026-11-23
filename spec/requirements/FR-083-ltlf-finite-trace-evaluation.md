---
id: FR-083
title: "Evaluate LTLf formulas on nonempty finite traces with explicit end-of-trace rules"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-044
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-044
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-332
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-333
    type: depends_on
---

# FR-083: Evaluate LTLf formulas on nonempty finite traces with explicit end-of-trace rules

## Description

When a validated formula under `mltl.ltlf-finite-trace/v1` and a nonempty finite
trace are supplied, tl-mltl shall return the Boolean truth at position 0
according to the end-of-trace rules of tl-syntax ADR-044, with no padding.

## Inputs

- A validated LTLf `Formula` and a trace of `n >= 1` positions of total
  Boolean valuations bound through the FR-007 signal catalog.
- A work limit.

## Outputs

- `Ok(bool)` or a typed refusal: `EmptyTrace`, `ProfileMismatch`,
  `UnboundSignal`, `WorkLimitExceeded`.

## Behavior

The rules are those of tl-syntax ADR-044 item 3. An empty trace is
`EmptyTrace`, never false. A formula under any other profile is
`ProfileMismatch`; `evaluate_closed` and `evaluate_prefix` refuse an LTLf
formula and `evaluate_ltlf` refuses every other profile. The horizon analysis of
an LTLf formula returns the typed result `unbounded-by-trace-length` and never
a wrapped number (FR-002 checked arithmetic). Evaluation is iterative over
positions from the last to the first or otherwise bounded in stack depth, and
cost is bounded by `nodes x n`, charged against the work limit.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-083-AC-1 | The 18 hand-authored rule cases and the LTLf reading of the 2 counterexamples (20 evaluations) return their manifest verdicts. | Test (TC-300) |
| FR-083-AC-2 | An empty trace, a foreign profile and an unbound signal each produce their own typed refusal, and cross-profile evaluator calls refuse an LTLf formula. | Test (TC-301) |
| FR-083-AC-3 | Over all 120 depth-one formulas and all 340 traces of length 1 to 4 on two atoms, `Xw p = not Xs not p`, `G p = not F not p` and `p R q = not(not p U not q)` hold. | Test (TC-302) |
| FR-083-AC-4 | LTLf horizon analysis returns the typed unbounded result, and a work limit one below the charge refuses while the exact charge succeeds. | Test (TC-303) |

## Dependencies

tl-syntax FR-332/FR-333 supply the graphs; FR-084 checks the result against an
outside reference.
