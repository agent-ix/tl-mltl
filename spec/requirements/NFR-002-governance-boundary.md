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
| Retained evidence bytes modified | 0 | 0 | Test |
| Requirement-tagged tests Cargo does not compile and run | 0 | 0 | Test |

## Verification

Schema-negative tests reject unknown identities. Retained evidence is read
through the compatibility mapping packaged with the pinned Engineering Assurance
release, never through a local mapper, and the read is measured to have moved no
byte. The compiled Rust test census re-derives which requirement-tagged tests
Cargo actually runs.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-002-AC-1 | Unknown schema/profile versions and omitted material identities are rejected. | Test (TC-012, TC-014) |
| NFR-002-AC-2 | Exchanged and retained records name exact tl-syntax, corpus, external-tool, dependency, and output identities without recording an automated release decision, and the retained records are read without being modified. | Test (TC-016, TC-021) |
| NFR-002-AC-3 | Every requirement-tagged Rust test is a test Cargo actually compiles and runs, and no compiled requirement-tagged test is ignored or configured out, so a matrix row cannot be backed by a tag above a test that never executes. | Test (TC-017) |

## Dependencies

Applies PGM-01 to FR-004, FR-005, and the repository release workflow. The
generic evidence-collection controls this requirement formerly carried as
NFR-002-AC-3 and NFR-002-AC-4 — host-scoped executable census, allowlisted
collection environment, corroborated positive outputs, envelope self-attestation
refusal, and Make execution-control policing — were removed with the local
evidence framework. What survives of that intent, and what does not, is stated in
NFR-003, which owns the shared-assurance intake path and records the measured
cost of the removal.
