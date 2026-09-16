---
id: SR-049
title: Failure-domain review of QObs C00 consumer compatibility
type: SpecReview
analysis: failure-domain
scope: "FR-019; TC-085"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-mltl/FR-019
    type: reviews
---

# Failure-domain review of QObs C00 consumer compatibility

## Summary

The review probed revision substitution, foreign contract coercion, partial
output, implicit semantic reconstruction, and resource-limit exhaustion. Each
failure is either the existing FR-018 typed error or a closed unsupported
outcome that carries no value-like payload.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4901 | low | No failure-domain omission found: invalid temporal inputs fail through the bounded owner reader, and repair/query inputs cannot become Boolean, aggregate, repair, or partial output. | FR-019-AC-1, FR-019-AC-2, FR-019-CON-2 |
