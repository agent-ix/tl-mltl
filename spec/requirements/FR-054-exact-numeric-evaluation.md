---
id: FR-054
title: Evaluate numeric predicates exactly by reduction to the existing evaluators
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-053
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-304
    type: depends_on
---

# FR-054: Evaluate numeric predicates exactly by reduction to the existing evaluators

## Description

When a bound formula, predicate table, catalog and numeric trace are supplied,
tl-mltl shall compute each predicate's truth per position with exact integer
arithmetic and shall obtain every verdict from the existing closed, prefix and
past evaluators over those truth values, adding no temporal semantics.

## Inputs

- A `BoundPredicateFormula` (tl-syntax FR-304), the numeric trace, the profile,
  an evaluation time or anchor, and limits.

## Outputs

- The existing report types extended by a `numeric` section: predicate
  proposition ids, per-position truth (`true`, `false`, `unobserved`), counts of
  unobserved cells, and the derived Boolean or three-valued trace itself (no digest of it).
  Existing report schemas are unchanged; the numeric section lives in a new
  `tl-mltl.numeric-evaluation/v1` wrapper that embeds the existing report.
- Typed `NumericEvaluationError`; never a verdict on error.

## Behavior

- Value at a position is a signed `i64` coefficient with its scale; arithmetic
  uses `i128`, so with the tl-syntax static range check no operation overflows; if
  an `i128` operation nevertheless overflows (a table not validated by tl-syntax),
  the evaluation fails with `ArithmeticOverflow` and no verdict. Comparison is
  exact integer comparison at equal scale. No floating-point type appears in
  the numeric path (verified by inspection of the module's types and a
  dependency check).
- The reduction preserves temporal meaning: for a trace with no numeric values
  and no predicates, results, reports and bytes are identical to the existing
  evaluators (TC-1575).
- Reference for agreement: predicate truth tables retained in the tl-syntax
  numeric corpus were computed independently of tl-syntax and tl-mltl with exact
  rational arithmetic; temporal verdicts are confirmed by `tl-oracle` on the Boolean
  trace those tables induce (`tl-oracle` shares no code with tl-mltl and
  evaluates Boolean traces only; it is not extended by this project).
- The expansion-equivalence check of tl-syntax FR-308 executes here: the
  expanded and the hand-written formula are evaluated on every trace of the
  declared population (567 traces in the small-scope declaration) and must agree
  everywhere.
- Evaluation work is bounded by the existing node-evaluation limit plus
  `positions x predicates x expression nodes`, checked before the loop.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-054-AC-1 | On the numeric corpus, every case's per-position predicate truth equals the independently computed table and every verdict equals `tl-oracle` on the induced trace, for closed and prefix profiles. | integration-testing (TC-1573, TC-1574) |
| FR-054-AC-2 | Traces without numeric content give byte-identical results to the existing evaluators. | integration-testing (TC-1575) |
| FR-054-AC-3 | Predicate truth equals an independent exact reference for generated expressions and values within declared domains, including both bounds, and no operation overflows. | property-based-testing (TC-1576) |
| FR-054-AC-4 | Every aggregation case's expanded and hand-written forms agree on all 567 population traces. | integration-testing (TC-1577) |

## Dependencies

FR-001, FR-003 (existing closed and prefix evaluation), FR-042 (past).
