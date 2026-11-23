---
id: FR-084
title: "Agree with an outside LTLf reference on a retained corpus including the counterexample formulas"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-044
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-083
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-335
    type: depends_on
---

# FR-084: Agree with an outside LTLf reference on a retained corpus including the counterexample formulas

## Description

When the retained corpus `tl-mltl.ltlf-reference-corpus/v1` is replayed, tl-mltl
shall produce, for every row, the verdict the outside reference recorded, and
shall detect a stale or tampered corpus.

## Inputs

- The retained corpus: 40,800 rows (120 depth-one formulas x 340 traces of
  length 1 to 4 over two atoms, 120 x (4+16+64+256)), the 20 LTLf rule and
  counterexample rows of FR-335, and 2 infinite-trace rows.
- The tl-syntax ltlf corpus via `tl_syntax::CORPUS_DIR`.

## Behavior

Each row records formula, trace, verdict and the reference identity. The
provenance block records tool name, exact version, command line, input digest,
operator and date. Replay never executes the tool. The two counterexamples
must show LTLf verdicts that differ from their infinite-trace verdicts (CE-1
false versus true; CE-2 true versus false). A direct test-only evaluator
written from the definition table, sharing no code with production, must agree
on the same 40,820 LTLf rows (40,800 plus 20), with a mutation control that
flips one rule and is caught. **Fresh generation of the corpus with the outside
tool is DEFERRED**: no tool is run during specification; until the corpus
exists the criteria below stay unmet.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-084-AC-1 | All 40,820 retained LTLf rows (40,800 population plus 20 rule and counterexample rows) are reproduced by `evaluate_ltlf`. | Test (TC-231) |
| FR-084-AC-2 | The 4 counterexample entries reproduce, with the 2 infinite readings evaluated by the lasso evaluator and each pair differing. | Test (TC-232) |
| FR-084-AC-3 | A corpus whose provenance block is missing a field, or whose rows or digest are altered, is refused before replay. | Test (TC-233) |
| FR-084-AC-4 | The test-only direct evaluator agrees on all 40,820 rows and a single flipped rule is detected. | Test (TC-234) |

## Dependencies

FR-083; ADR-044 owns the reference choice; tl-syntax FR-335 owns the hand cases.
