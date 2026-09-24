---
id: FR-045
title: Test semantic laws and strict round-trips
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-043
    type: depends_on
---

# FR-045: Test semantic laws and strict round-trips

## Description

For every new V1 acceptance criterion, the owning crate shall classify the
criterion as property, example or justified exclusion and execute its
applicable semantic law or strict round-trip against real code.

## Behavior

Laws include Boolean and temporal duality, bounded embedding, lasso
unrolling, fairness weakening, partial-information monotonicity,
finite-prefix refutation soundness and rewrite equivalence. Round-trips cover
all versioned syntax, parser and result documents without accepting unknown,
reordered or noncanonical fields. Properties use recorded seeds and shrinking;
zero accepted cases or excessive discards are non-conclusive. A mutant that
breaks a stated law must fail the actual test, not a duplicate test model.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-045-AC-1 | Each criterion has one executable classification and every named law uses admitted real-code inputs with a valid oracle or independent metamorphic premise. | Test (TC-179) |
| FR-045-AC-2 | Strict wire round-trips and malformed-input refusals run for every affected edition; vacuous generators and seeded law faults fail. | Test (TC-180) |

## Dependencies

FR-043 supplies semantic comparison where a law is not independently decisive.
