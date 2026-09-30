---
id: NFR-002
title: Retain governance and qualification boundaries
type: NFR
---

# NFR-002: Retain governance and qualification boundaries

## Statement

Every exchanged record shall use an explicit supported schema, exact source and
corpus pins, and contribution provenance.
Contextual records shall preserve the exact shared signal and caller-context
identities without claiming their truth. Agent results shall remain distinct
from human approval and consuming-project validation.

## Scope

All wire documents, cross-repository pins, evidence records, and release
claims are in scope.

## Rationale

Unidentified schema, source, tool, or corpus drift invalidates differential and
qualification support even when a Boolean result happens to match.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Unversioned exchanged document kinds | 0 | 0 | Test |
| Omitted material provenance identities | 0 | 0 | Inspection |

## Verification

Schema-negative tests reject unknown identities.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | Unknown schema/profile versions and omitted material identities are rejected. | Test (TC-012, TC-014) |
| NFR-002-AC-2 | Exchanged records name exact tl-syntax, corpus, external-tool, dependency, and output identities without recording an automated release decision. | Test (TC-016) |
| NFR-002-AC-4 | Every contextual native record names the exact tl-mltl revision, complete shared catalog identity, and exact optional requirement context without claiming that tl-mltl validated the caller's provenance or a consuming monitor. | Test (TC-025, TC-028, TC-031) |

## Dependencies

Applies these governance and qualification boundaries to FR-004, FR-005, and
the repository release workflow.
