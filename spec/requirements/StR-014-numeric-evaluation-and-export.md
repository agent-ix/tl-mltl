---
id: StR-014
title: Monitor users need reproducible numeric and contract verdicts that agree with R2U2 or say why not
type: StR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-053
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-054
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-056
    type: satisfied_by
  - target: ix://agent-ix/tl-mltl/FR-059
    type: satisfied_by
---

# StR-014: Monitor users need reproducible numeric and contract verdicts that agree with R2U2 or say why not

## Stakeholder Need

Runtime-monitoring users and assessors require that tl-mltl evaluate numeric predicates and assume-guarantee contracts reproducibly over traces that carry numbers, export them to C2PO so R2U2 can monitor them, and compare them with R2U2 per step, with every construct that cannot be exported exactly refused and named.

## Rationale

R2U2 and the Lunar Gateway work use arithmetic and contracts. Floating-point
differences between an exact evaluator and a monitor can silently flip a verdict
at a boundary; an unexplained disagreement or an uncounted refusal is not
evidence.

## Validation Criteria

| ID | Criteria | Validation |
|---|---|---|
| StR-014-VC-1 | Two runs of the evaluator on one numeric trace give byte-identical reports on every platform. | Demonstration |
| StR-014-VC-2 | For the numeric corpus a reviewer sees admitted, refused (with typed cause) and disagreeing cases counted separately, and a refusal is never counted as agreement. | Demonstration |
| StR-014-VC-3 | A reviewer sees a contract's inactive, invalid or verified status per step beside R2U2's. | Demonstration |

## Dependencies

tl-syntax FR-302..FR-311 and ADR-014..ADR-018.
