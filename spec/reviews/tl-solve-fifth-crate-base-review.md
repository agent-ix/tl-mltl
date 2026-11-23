---
id: SR-1470
title: "Independent base review of ADR-074, FR-113 and TM-076 (TL-393)"
type: SpecReview
analysis: base
scope: "spec/decisions/ADR-074-fifth-production-crate-tl-solve.md, spec/requirements/FR-113-solver-independence.md, spec/solver-independence-test-matrix.md, ADR-003 edit, spec.md edit"
review_set: base
---

# Independent base review: tl-mltl TL-147-spec

## Summary

Reviewed against the tl-solve spec and the tl-mltl manifest. Cited ids resolve (ADR-003, FR-113 target tl-solve/ADR-004, feature `infinite-trace` exists in Cargo.toml). `quire validate` shows no delta against main; coverage adds four planned rows (exempt). Full cross-repo analyses are in tl-solve spec/reviews (SR-1470 to SR-1480).

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | medium | ADR-074 rejects "amend in place" for an accepted ADR, but the same commit edits accepted ADR-003's sentence in place. It also gives a `supersedes` relationship to all of ADR-003 while the body says only one sentence is superseded, so tooling will read ADR-003 as fully superseded and its infinite-trace boundary as void. | ADR-074 Decision, Alternatives; ADR-003 |
| FND-002 | medium | FR-113-AC-2 is a source and manifest text search (`tl_solve` import, `allow-git` entry); the checklist disallows scanning tests. FR-113-AC-1 (`cargo tree`) is the structural check and suffices. | FR-113-AC-2 |
| FND-003 | low | FR-113 forbids the dev-dependency graph from containing tl-solve; true today (dev dependency is tl-oracle only). The criterion names `varisat`, which no spec adds to tl-mltl; harmless. | FR-113 |
| FND-004 | low | Test matrix TM-076 rows are planned and say deferred; no baseline invalidated. Ids ADR-074 and FR-113 skip over the repo's current maximum (ADR-003, FR-042) and the TL-151 branch already allocates FR-091; reserve ids to avoid collision. | TM-076 |
| FND-005 | low | Verification cells carry `TC-` ids (checklist). | FR-113 ACs |
