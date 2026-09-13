---
id: FR-016
title: Verify derived future operators through canonical lowering
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-001
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-008
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-010
    type: depends_on
---

# FR-016: Verify derived future operators through canonical lowering

## Description

Where a bounded weak-until (`W[a,b]`) or strong-release (`M[a,b]`) operator
reaches evaluation, tl-mltl shall evaluate, analyze, and bound only the
canonical primitive graph returned by the tl-syntax
`tl-syntax.future-operators/v1` lowering, and shall demonstrate that the
results equal an independent direct reference for W/M finite-trace semantics
under both semantic profiles.

## Inputs

- W/M lowerings produced by `tl_syntax::FutureLoweringRequest::lower` at the
  pinned tl-syntax revision: `p W[a,b] q` as `Or(p U[a,b] q, G[a,b] p)` and
  `p M[a,b] q` as `And(p R[a,b] q, F[a,b] p)`, appended to a validated graph.
- The same canonical graph constructed directly, node by node, without the
  lowering request.
- A test-only direct reference in first-occurrence form: `p W[a,b] q` holds at
  `t` iff `p` holds on all of `[t+a,t+b]` or `q` holds at an offset no later
  than the first offset where `p` fails; `p M[a,b] q` holds iff `p` holds at an
  offset in the window and `q` holds at every offset up to and including the
  first such offset.
- Both `mltl.closed-trace/v1` and `mltl.online-prefix/v1`, empty and short
  traces, open and explicitly closed prefixes, verdict times, and
  `EvaluationLimits`.

## Behavior

- The evaluator, horizon analysis, and prefix semantics acquire no W/M node
  kind, match arm, operator-profile identity, or lowering call. The derived
  operators exist only as tl-syntax lowering and the test-only reference.
- Closed-trace verdicts of the lowered graph equal the direct reference, with
  instants after the trace false.
- Online-prefix verdicts of the lowered graph equal the exact verdict over
  every continuation of the prefix. For negation-free operands the exact
  verdict is decided by the all-false and all-true continuations; the prefix
  parity sweep is restricted to that class because Kleene evaluation is
  intentionally not continuation-exact for correlated negated operands such as
  `p W q` with `q = !p`, and this requirement adds no alternative prefix
  semantics. Closed-profile parity covers negated and correlated operands.
- Prefix progress never retracts a decisive verdict and is decisive, equal to
  the closed verdict, once the prefix covers the verdict time plus the lookahead.
- Horizon lookahead of the lowered graph equals `b` plus the deeper operand
  lookahead, computed without wrapping at `[u32::MAX,u32::MAX]` and under
  nesting.
- Work, recursion-depth, temporal-span, time-arithmetic, and formula-v1 node
  budget outcomes of the lowered graph equal direct canonical construction.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-016-AC-1 | For W and M, every inclusive window `0 <= a <= b <= 3`, every two-proposition trace of length 0 through 5, verdict times 0 through 2, and plain, constant, repeated, nested, negated, and correlated operands, closed-trace verdicts of the lowered graph equal the direct reference. | Test (TC-076) |
| FR-016-AC-2 | Over the same windows and traces, open-prefix verdicts for negation-free operands equal the exact continuation verdict and exercise pending, explicitly closed prefixes for every operand shape equal the closed direct reference, and along every prefix of longer traces verdicts never retract and are decisive and equal to the closed verdict beyond the verdict time plus lookahead. | Test (TC-077) |
| FR-016-AC-3 | Under both profiles, horizon reports for lowered and directly constructed graphs are equal, their lookahead equals the direct `b + max(child)` value for `[0,0]`, `[0,u32::MAX]`, `[u32::MAX,u32::MAX]` and nested lowerings, nested maximum windows report `2 * u32::MAX` without wrapping, and the evaluation record carries the same horizon. | Test (TC-078) |
| FR-016-AC-4 | Lowered nodes, spans, and report fields equal direct construction; lowered and directly constructed graphs share exact work-limit, recursion-depth, temporal-span, and time-overflow outcomes under both profiles; a lowering landing exactly on the formula-v1 node limit evaluates and one more base node is refused. | Test (TC-079) |
| FR-016-AC-5 | Exchanging Until and Release, Globally and Future, Or and And, or W and M, exchanging binary operands, binding the unary node to `q`, widening the end, or raising the start each produces a graph that disagrees with the direct reference, while the tl-syntax lowering and unmutated direct construction disagree nowhere. | Test (TC-080) |
| FR-016-AC-6 | Every Rust source file under `src/`, at any depth, is free of the derived future vocabulary, and lowered graphs contain only the twelve canonical node kinds named by a wildcard-free match. | Test (TC-080) |

## Dependencies

Depends on tl-syntax FR-008 as implemented by `tl-syntax` main
`8dc18eec5af227f484170362c9e8894b8531a27d`, and implements the tl-mltl portion
of tl-syntax FR-010-AC-1 and FR-010-AC-4 routed by
[agent-ix/tl-mltl#47](https://github.com/agent-ix/tl-mltl/issues/47). Rewrite
equivalence belongs to tl-rewrite#35; lowered-graph export belongs to
tl-mltl#48.
