---
id: FR-043
title: Compare production semantics with an independent tl-oracle
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-031
    type: depends_on
---

# FR-043: Compare production semantics with an independent tl-oracle

## Description

The V1 verification campaign shall compare the production evaluators and
rewrite rules with a dev-only `tl-oracle` that implements the normative
finite, past and infinite semantics independently.

## Behavior

The separate `agent-ix/tl-oracle` repository depends only on tl-syntax public
contracts and no production tl-mltl or tl-rewrite code. It is never released
as a production TL crate. Its explicit recursive finite evaluator and lasso
procedure derive directly from the semantic clauses, with no shared helper
that decides truth. A source import/dependency graph check enforces that seam.
A seeded wrong production rule and a seeded wrong oracle clause each make a
corresponding comparison red; a self-oracle cannot earn coverage. The TL-210
reference scaffold moves into this crate at TL-221 and is extended for nested
past/future loops before it qualifies mixed formulas.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-043-AC-1 | The oracle imports no production evaluator or rewriter code and its result agrees on reviewed finite, past and lasso fixtures. | Test (TC-175) |
| FR-043-AC-2 | Independent seeded faults in each direction are detected, and a self-oracled comparison is refused. | Test (TC-176) |

## Dependencies

TL-35 scaffolds tl-oracle after TL-215; TL-221 completes its lasso procedure.
