---
id: StR-044
title: Temporal-logic and FRET users need LTLf evaluation and lossless-or-declared interchange
type: StR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-083
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-084
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-086
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-087
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-090
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-091
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-092
    type: satisfied_by
---

# StR-044: Temporal-logic and FRET users need LTLf evaluation and lossless-or-declared interchange

## Stakeholder Need

Users of LTLf, FRET, R2U2 and WEST need to evaluate LTLf formulas, bring their
existing exported files in, and be told exactly what, if anything, an import
could not carry or what upstream mission-bounding may already have changed.

## Rationale

Silent loss in a requirements pipeline turns a verified requirement into a
different one. A typed loss record, a refusal that names the construct, and a
horizon-based note about mission-bounding keep the user in control.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-044-VC-1 | LTLf verdicts agree with an outside reference on a retained corpus. | Test (TC-304, TC-305) |
| StR-044-VC-2 | No import drops, defaults or approximates a construct without a typed record. | Test (TC-320, TC-321) |

## Stakeholders

FRET and R2U2 users, LTLf researchers, regulated-requirements authors.

## Context and Assumptions

Sources: De Giacomo and Vardi IJCAI 2013; De Giacomo, De Masellis, Montali AAAI
2014; Aurandt et al. NFM 2026; R2U2 MLTL semantics page.

## Traceability

Realized by FR-083 through FR-094; constrained by NFR-047.
