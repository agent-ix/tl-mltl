---
id: NFR-001
title: Preserve determinism and bounded resource claims
type: NFR
---

# NFR-001: Preserve determinism and bounded resource claims

## Statement

All library results and serialized records shall depend only on identified
inputs. Algorithms shall check arithmetic, configured evaluation recursion and
work limits, and the mapping recursion boundary before performing impractical
temporal expansion.

## Scope

All public library functions, CLI records, mapping manifests, context identities
and digests, and retained differential artifacts are in scope.

## Rationale

Reference-oracle and resource-selection evidence must be reproducible and must
not understate bounds after arithmetic or work-budget failure.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Nondeterministic output mismatches | 0 | 0 | Test |
| Wrapped or partial resource claims | 0 | 0 | Test |

## Verification

Requirement-tagged unit and integration tests repeat semantic and serialized
operations and exercise checked arithmetic and configured work limits.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-001-AC-1 | Repeated evaluation, analysis, mapping, context binding, digesting, comparison, and serialization are byte-for-byte stable. | Test (TC-003, TC-013, TC-014, TC-025, TC-028, TC-029) |
| NFR-001-AC-2 | Overflow and work-limit exhaustion are explicit errors with no partial success claim. | Test (TC-004, TC-006) |

## Dependencies

Constrains FR-001 through FR-005 and FR-007.
