---
id: SR-012
title: Composite specification review of the shared-assurance residuals
type: SpecReview
analysis: base
scope: spec/requirements/NFR-003-qualification-integrity.md, spec/test-matrix.md, assurance/change-assurance.json, agent-ix/tl-mltl#20
review_set: all
---

# Composite specification review of the shared-assurance residuals

## Summary

Dependency, risk, evidence, integrity, scope, failure-domain and EARS review of
the PR #21/#22 residuals found one controlling requirement and two existing
test-matrix rows sufficient. NFR-003-AC-2 and TC-019 gain the missing positive
scratch and serialization clauses; TC-024 already owns census falsifiability.
No deleted stable acceptance-criterion identity is reused.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1201 | medium | Shared-input serialization was an untyped call-site convention and omitted a live reader/writer pair. Require a guard token at stateful helpers, serialize both omitted tests, and fail distinctly on poison. | NFR-003, TC-018, TC-019 |
| FND-1202 | medium | TC-019 declared the injected-child refusal but not the unmodified-driver run or store-isolation condition needed to prove the scratch itself is valid. Add both to NFR-003-AC-2 and TC-019. | NFR-003-AC-2, TC-019 |
| FND-1203 | low | TC-024's implementation has four falsifiability and accuracy residuals, but its existing acceptance criterion already requires exact deleted-reference controls and fail-closed Git enumeration. Strengthen the controls without creating a new criterion. | FR-006-AC-7, TC-024 |
| FND-1204 | low | NFR-003-AC-4 was deliberately deleted with legacy-evidence preservation and is a stable historical identity. Do not reuse it for the new serialization clause. | SR-009 FND-903, SR-010 FND-1004 |

## Verdict

READY. The plan closes the observed mechanisms inside existing requirement
boundaries and explicitly excludes the common Make-qualification work in #14.
