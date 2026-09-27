---
id: FR-032
title: Admit lasso traces and fairness premises
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-031
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-021
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-022
    type: depends_on
---

# FR-032: Admit lasso traces and fairness premises

## Description

When given a validated lasso and optional fairness premises, the infinite
provider shall interpret the nonempty loop as repeating forever and restrict
claim settlement to traces satisfying every admitted premise infinitely often.

## Behavior

The finite prefix may be empty. The materialized loop starts at the declared
entry and repeats without inserting a gap or terminal position. Each fairness
root belongs to the same formula graph, profile and clock. Its condition must
hold at infinitely many positions of the resulting trace. For a complete
lasso this reduces to at least one satisfying loop position per premise.
Partial valuations are interpreted through FR-030 over common completions;
fairness filters those completions before claim truth is determined. If no
completion satisfies the premises, the provider records an empty fair-admission
cause and emits no vacuous proof or refutation. Fairness premises do not alter
the denotation of the claim formula on an admitted trace.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-032-AC-1 | Empty and nonempty prefixes, loop-entry boundaries and repeated loops match an independent infinite-word oracle. | Test (TC-150, TC-151) |
| FR-032-AC-2 | Empty premise sets admit all valid lassos; each nonempty premise is visited infinitely often on every admitted completion, including partial cases. | Test (TC-152, TC-153) |
| FR-032-AC-3 | Unfair or empty fair-admission cases do not produce vacuous proof or refutation; malformed or foreign lasso/fairness identities refuse without partial result. | Test (TC-154) |

## Dependencies

FR-031 owns temporal truth; tl-syntax FR-021/022 own validated premise and
lasso inputs.
