---
id: SR-004
title: tl-mltl merged PGM-01 reconciliation
type: SpecReview
analysis: base
scope: PGM-01 requirements and the tl-mltl v0.1 candidate
review_set: all
---

# tl-mltl merged PGM-01 reconciliation

## Summary

Merged PGM-01 policy: `agent-ix/quire-contract-ir#12` at
`7dac9d8c19952412b56a0347387666e2ca81e01d`.

Envelope schema: `quire.derivation-evidence/v1`, SHA-256
`0946e235e9e4b0fa79e9b9ec27ae157b303c17de0a9408d3cc04968fb7152256`.

This is the post-merge reconciliation against the immutable policy revision.
The collector architecture is adapted under MIT OR Apache-2.0 from the
same-program tl-syntax collector at immutable revision
`740182f13b84858008d6f176f75136737d405c1b`. The adaptation records tl-mltl
semantic and resource gates, both corpora, exact R2U2/C2PO identities,
differential limitations, complete command failures, canonical envelope
validation, overwrite refusal, and external checksums. Human review remains
pending.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-401 | medium | PGM-01 is merged and exactly reconciled; independent human review and the exact source-release decision remain open. | PGM-01, AP-001, AA-001 |

## Policy mapping

| Policy requirement | tl-mltl disposition | Evidence or remaining gate |
|---|---|---|
| PGM-01-R01 schema compatibility | Commands, traces, semantic reports, mappings, external verdicts, differential records, local evidence input, and manifests use explicit closed v1 identities. | FR-004, FR-005, NFR-002; wire and evidence schemas |
| PGM-01-R02 exact pins | Candidate evidence records source, merged policy, schema, toolchain, syntax dependency, parameter, corpus, input, executable, configuration, and output digests. | Collection input, manifest, canonical envelope, corpus manifests |
| PGM-01-R03 release order | tl-mltl pins the exact reviewed tl-syntax revision; downstream rewrite and consuming repositories must pin the eventual reviewed tl-mltl source revision. | Cargo.toml, Cargo.lock; human source decision remains open |
| PGM-01-R04 licensing and provenance | Crate, local schemas, collector adaptation, and authored corpus records are MIT OR Apache-2.0; canonical R2U2 source provenance and license identity are retained separately. | Cargo.toml, corpus/README.md, evidence/README.md, license audit |
| PGM-01-R05 clean-room boundary | This repository consumes the typed tl-syntax graph and does not add a text grammar, parser table, or imported grammar fixture. | MRS-001 out-of-scope boundary and repository inspection |
| PGM-01-R06 human authority | Agent-produced implementation and evidence remain separate from the independent human source-release decision. | AP-001, AA-001, canonical envelope provenance |
| PGM-01-R07 classification | tl-mltl is a linked reference/analysis component and requires consuming-project verification. | AP-001, CAC-001, README.md |
| PGM-01-R08 common envelope | Revision-scoped evidence emits every canonical core field and is gated by the pinned PGM-01 Draft 7 schema and validator. | Evidence collector and retained PGM-01 validation outputs |
| PGM-01-R09 retention and decision | New runs refuse overwrite, retain stdout/stderr and SHA-256s, preserve limitations, and record no automated release decision. | Evidence collector, external checksum file, AA-001 |
| PGM-01-R10 qualification boundary | Reference, resource, mapping, and differential evidence confer no monitor qualification, project validation, accreditation, or certification. | AP-001, AA-001, README.md, differential report |

The merged policy requires no public semantic API change. Independent review,
protected-branch checks, and the human source-release decision remain external
workflow gates.
