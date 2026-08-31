---
id: NFR-002
title: Retain governance and qualification boundaries
type: NFR
relationships:
  - target: ix://agent-ix/quire-contract-ir/PGM-01
    type: references
---

# NFR-002: Retain governance and qualification boundaries

## Statement

Every exchanged record shall use an explicit v1 schema, exact source and corpus
pins, contribution provenance, and canonical PGM-01 evidence boundaries. Agent
results shall remain distinct from human approval and consuming-project
validation.

## Scope

All wire documents, cross-repository pins, evidence records, and release claims
are in scope.

## Rationale

Unidentified schema, source, tool, or corpus drift invalidates differential and
qualification support even when a Boolean result happens to match.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Unversioned exchanged document kinds | 0 | 0 | Test |
| Omitted material provenance identities | 0 | 0 | Inspection |

## Verification

Schema-negative tests reject unknown identities, while evidence inspection and
checksums verify exact source, tool, corpus, and output pins.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | Unknown schema/profile versions and omitted material identities are rejected. | Test (TC-012, TC-014) |
| NFR-002-AC-2 | Evidence names exact tl-syntax, PGM-01, corpus, tool, dependency, and output identities without recording an automated release decision. | Test (TC-016) |

## Dependencies

Applies PGM-01 to FR-004, FR-005, and the repository release workflow.
