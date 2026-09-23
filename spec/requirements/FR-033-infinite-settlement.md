---
id: FR-033
title: Settle infinite claims without promoting finite prefixes
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-028
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-030
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-032
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-341
    type: references
---

# FR-033: Settle infinite claims without promoting finite prefixes

## Description

For an admitted infinite request, the provider shall map the semantic
possibility set and evaluation outcome to one of FR-341's five dispositions,
with a typed detail that states its actual evidence and limits.

## Behavior

An admitted, fully evaluated infinite trace yields `proved` only when every
admitted possibility satisfies the claim, and `refuted` only when every
admitted possibility falsifies it. Both possible truth values yield
`inconclusive`. A finite prefix alone may produce a refutation only for a
property whose violation is decisive under all infinite continuations; it
cannot prove liveness. A finite prefix with no decisive counterexample is
`inconclusive`, not `proved`. Unsupported syntax/profile/clock/capability
requests are `unsupported`; an internal provider fault is `failed` with
execution disposition `failed`; an exhausted configured resource or timeout
is `failed` with execution disposition `resource-incomplete`. These two
causes remain distinct. No outcome silently falls back to bounded
closure-as-false semantics.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-033-AC-1 | Proved, refuted and inconclusive are distinguished by independent complete and partial infinite cases, with sound witness/counterexample details. | Test (TC-155, TC-156) |
| FR-033-AC-2 | A finite prefix cannot prove liveness and refutes only a continuation-invariant safety violation. | Test (TC-157) |
| FR-033-AC-3 | Unsupported and both failed execution dispositions keep their exact FR-341 axis combinations and never become Boolean verdicts. | Test (TC-158) |

## Dependencies

FR-028 owns registration; FR-030/032 own possibility and fair-admission
semantics; QSpec FR-341 owns the result vocabulary.
