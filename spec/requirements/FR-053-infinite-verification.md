---
id: FR-053
title: Verify lasso, fairness and partial semantics independently
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-043
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-033
    type: depends_on
---

# FR-053: Verify lasso, fairness and partial semantics independently

## Description

When the V1 infinite-trace lane runs, it shall compare the feature-gated
provider with tl-oracle on complete and partial lassos, fairness premises,
finite prefixes and all admitted temporal operators.

## Behavior

Generated cases include empty/nonempty prefixes, one/multiple loop
positions, lower-bound boundaries, nested past/future, missing/conflicting
atoms, fair/unfair completions and resource limits. The oracle handles past
state across repeated loops, not just the first materialized lap. Metamorphic
laws check lasso unrolling, duality, fairness weakening, monotonicity under
added information and refutation soundness. A provider result is never its
own oracle. A failed or resource-incomplete result stays on FR-341's failed
axis and earns no semantic coverage.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-053-AC-1 | Every operator and valuation/fairness class agrees with the independent lasso oracle, including mixed past/future over repeated loops. | Test (TC-193) |
| FR-053-AC-2 | Seeded wrong fixed points, fairness filters, prefix proofs and result-axis mappings are caught by real tests. | Test (TC-194) |

## Dependencies

FR-043 supplies the oracle; FR-033 supplies settlement rules.
