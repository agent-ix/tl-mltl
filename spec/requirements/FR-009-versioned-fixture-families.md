---
id: FR-009
title: "Version and retain owned corpus fixture families"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-006
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-008
    type: depends_on
---

# FR-009: Version and retain owned corpus fixture families

## Description

When a canonical or observed fixture enters the campaign, the corpus campaign shall
require its owning repository to publish a versioned family manifest with immutable
identity, complete provenance, exact digests, and an explicit lifecycle classification.

## Inputs

- One FR-008 coverage cell and candidate fixture bytes.
- Family owner, schema/profile identities, expected-result provenance, source
  and license provenance, and every consumed dependency revision.
- Lifecycle class `canonical`, `retained-external-observation`, or `generated`.

## Outputs

- An admitted versioned fixture reference or typed refusal.
- A deterministic manifest/corpus digest and owner-routed consumer list.
- A promotion record when generated input becomes a reviewed canonical case.

## Behavior

The campaign manifest identity is `tl-mltl.corpus-manifest/v1`; existing
`tl-syntax-corpus/v1` and `tl-mltl.r2u2-corpus/v1` bytes and meanings remain
unchanged. A v1 fixture record contains at least:

- stable case and family identifiers, lifecycle class, owner repository, and
  schema version;
- exact formula, profile, operator/dialect when applicable, trace/history,
  anchor/clock, proposition/catalog/context, limit, operation, and coverage-cell
  identities or explicit absence;
- expected truth/progress/resource/refusal plus oracle identity/revision and
  derivation class, or external observation state with no reference verdict;
- SHA-256 for every input, expected record, retained output, manifest, and
  vendored artifact; and
- source repository/revision/path, contribution provenance, SPDX license,
  generator/tool/configuration identities, and stated limitations.

Manifest and artifact paths are normalized repository-relative UTF-8 paths.
Absolute paths, empty components, `.` or `..` components, duplicate normalized
paths, symlinks, non-regular files, untracked files, and resolution outside the
declared corpus root refuse before bytes are parsed. The manifest declares
bounded case count, artifact count, path/string length, file size, aggregate
bytes, and nesting depth; values above the implementation hard caps refuse
before allocation or hashing. Digests are verified before semantic decode.

Fixture-family ownership is singular:

| Family | Authoritative owner | Consumers |
|---|---|---|
| canonical formula/profile/schema and proposition-map cases | `tl-syntax` | all TL components |
| evaluation, prefix, horizon/history, result, CLI, and mapping overlays | `tl-mltl` | evaluator and assurance lanes |
| internal text parse/format and span pairs | `tl-parse` | parser and shared replay |
| equivalence pairs and counterexamples | `tl-rewrite` | rewrite and shared replay |
| native predicate and native temporal correspondence | `quire-contract-ir` | native bridge and TL consumers after #63/#64 |
| target-specific loss/output records | the Rust adapter repository that emits the target | interop consumers |

A consumer may vendor or fetch only an exact owner revision plus manifest
digest and must verify byte identity before replay. It cannot restamp copied
bytes, rewrite their expected meaning, or make itself a second owner.

Canonical fixtures are append-only within one corpus identity. Changing bytes,
meaning, expected outcome, ownership, schema/profile, oracle, or provenance
requires a successor corpus identity and compatibility review. Removal remains
visible as a tombstone with replacement or withdrawal rationale; a case cannot
disappear to improve a ratio.

Generated property, fuzz, mutation, model-checking, and minimization inputs live
outside canonical fixture directories and the Git conformance population.
Promotion requires a deterministic minimal reproducer, independent expected
result, full identity/license/provenance, owning requirement and coverage cell,
review, and a successor manifest digest. Seed count or campaign output alone is
not conformance evidence.

Quoin owns evidence retention and receipts. The repository owns domain fixture
bytes and manifests but adds no generic evidence envelope, collector, audit
store, approval mechanism, or retention runtime.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | Every admitted v1 fixture round-trips all required identity, input, expected/observed outcome, digest, source, license, tool/configuration, owner, lifecycle, and limitation fields and refuses omissions, unknown fields, mixed versions, and invalid combinations. | Test (TC-044, TC-045) |
| FR-009-AC-2 | Every family has exactly one authoritative owner; consumers verify exact owner revision and manifest digest, and copied bytes cannot be restamped or reinterpreted. | Test (TC-044, TC-048) |
| FR-009-AC-3 | Changing any canonical byte, meaning, expected outcome, identity, owner, oracle, license, or provenance requires a successor identity; removal leaves a tombstone and cannot silently shrink the denominator. | Test (TC-045, TC-047) |
| FR-009-AC-4 | Generated campaign inputs remain outside the canonical population and can be promoted only with a minimal reproducer, independent oracle, complete provenance, owning cell/requirement, review, and new manifest digest. | Test (TC-045) |
| FR-009-AC-5 | Existing shared and R2U2 corpus bytes, identities, claims, and digest checks remain unchanged, and the new manifest lane introduces no local generic retention/evidence framework. | Test (TC-045, TC-048) |
| FR-009-AC-6 | Fixture loading rejects every absolute, escaping, ambiguous, symlinked, non-regular, untracked, non-UTF-8, duplicate, over-count, over-length, over-size, over-depth, or digest-mismatched artifact before semantic decode. | Test (TC-044, TC-045) |

## Dependencies

Depends on FR-008 for coverage identity and FR-006 for the shared assurance
boundary. Each conditional family also depends on its owning accepted profile
and implementation revision before any fixture can become canonical.
