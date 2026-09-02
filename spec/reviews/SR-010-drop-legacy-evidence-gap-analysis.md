---
id: SR-010
title: Gap analysis of the legacy-evidence deletion
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, assurance/**, scripts/**, tests/**, Makefile, .github/workflows/ci.yml
review_set: all
---

# Gap analysis of the legacy-evidence deletion

## Summary

Every item in `agent-ix/tl-mltl#16` is done, every SR-009 finding is
dispositioned, and no test-matrix row, acceptance criterion or suite-registry row
is left pointing at deleted material.

| Disposition | Count | Findings |
|---|---|---|
| FIXED | 0 | — |
| ACCEPTED with rationale | 6 | FND-901…906 |
| DEFERRED with a linked issue | 0 | — |

Nothing was FIXED because nothing was broken: SR-009's two mediums are
reductions in detection that follow from the deletion itself, and repairing
either would mean retaining machinery for material that no longer exists. Both
are recorded in `assurance/change-assurance.json` rather than argued away.

No plan bundle was created. This is a single-issue deletion with no TDD
decomposition to make — there is no red-to-green cycle in removing a file — and
the issue body carries the checklist. `PLAN-002` remains the migration's plan and
is not rewritten; its Task-002 still describes the compatibility view as it stood
when that task ran, which is what a plan log is for.

## Measurements at the final head

| Figure | Value | Population |
|---|---|---|
| Declared rows backed | 62 / 64 | every row Quire mints from `spec/`: 33 acceptance criteria, 23 test-matrix rows, 8 suite-registry rows |
| Suite-registry rows backed | 6 / 8 | `spec/evidence/suites.md`; `SUITE-001` and `SUITE-002` deliberately unbacked and named |
| Requirement-tagged Rust tests | 31, none ignored, none configured out | `scripts/rust_test_census.py` compared against `cargo test --list` |
| Rust tests executed by `make ci` | 31 | every test binary in the workspace; every one carries a `// Trace:` tag, so the two figures coincide |
| Chain obligations attested | 6 | `assurance/change-assurance.json`; was 7 |
| Chain cases | 21 scenarios, 10 controls, 8 adapter probes | all matched, unchanged by this deletion |
| Verification outcomes demonstrated | 12 / 12 | each bound to the named chain case that owns it, with no contribution from the deleted census |
| Producer rows | 61 | 14 reference-conformance + 36 R2U2 differential + 11 CLI conformance, unchanged |
| Mutation probes | 34 of 35 detected | the one that is not is the recorded `.IGNORE:` residual |
| Files deleted | 295 | 283 `evidence/` + 3 `schemas/` + 1 reader + 8 fixtures |
| Lines | 161 inserted, 7,575 deleted | `git diff --numstat f7eb8bd..HEAD` |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1001 | medium | SR-009 FND-901: `assurance/pins.json` no longer declares a digest-bearing consumed artifact, because the deleted compatibility view was the only reader of all four. `check_shared_pins.artifact_digest_mismatches` still runs but has an empty population, so `TC-018`'s `artifact_mismatches` assertion is vacuous. ACCEPTED — keeping digests for files nobody reads would be a false statement about what this repository consumes. Recorded as `UNKNOWN-consumed-artifact-digests-now-vacuous`. | assurance/pins.json, scripts/check_shared_pins.py | correct-requirement-no-evidence |
| FND-1002 | medium | SR-009 FND-902: six mutation probes (P06, P25, P26, P29, P32, P33) went with the material they guarded, and four new ones replace part of that surface. ACCEPTED, with the net measured rather than asserted: 35 probes, 34 detecting, the exception being the recorded `.IGNORE:` residual. | tests/shared_assurance.rs | correct-requirement-no-evidence |
| FND-1003 | low | SR-009 FND-903: `FR-006` and `NFR-003` now have gaps at `AC-4`. ACCEPTED: identities stay stable, because sealed records, the matrix and four prior reviews name them. | spec/requirements/FR-006-shared-assurance-intake.md, spec/requirements/NFR-003-qualification-integrity.md | wrong-requirement |
| FND-1004 | low | SR-009 FND-904: the broader "no local digest claims release authority" clause left with `NFR-003-AC-4`. ACCEPTED: still carried by the same requirement's metric table, its Qualification Boundary section, and `AA-001`. | spec/requirements/NFR-003-qualification-integrity.md | missing-requirement |
| FND-1005 | low | SR-009 FND-905: three prose mentions of `da2c7704` survive in `PLAN-001`'s log, its Task-006 and SR-008. ACCEPTED: historical records, not pins; none requires the commit to be fetchable. | spec/plans/**, spec/reviews/SR-008-shared-assurance-closing-gap-analysis.md | wrong-requirement |
| FND-1006 | low | SR-009 FND-906: two `quire validate` advisories reproduce at `f7eb8bd` and are unrelated to this change. ACCEPTED and reported so they are not attributed to it. | spec/test-matrix.md | correct-requirement-no-evidence |

## Traceability

Every requirement that argued from the retained evidence lost that argument
rather than a weakened restatement of it:

| Artifact | Before | After |
|---|---|---|
| `FR-006-AC-4` | retained bytes read through the mapping, answer reported uncollapsed | deleted |
| `FR-006-AC-7` | "…and the two retained evidence schemas are frozen and referenced by nothing" | "…and neither the retained legacy evidence nor any reader, fixture set, or schema belonging to it is present or referenced" |
| `NFR-002-AC-2` | named `TC-016, TC-021`, claimed the retained records are read unmodified | names `TC-016`; the retained-record clause is deleted |
| `NFR-002` metrics | carried "Retained evidence bytes modified: 0" | row deleted |
| `NFR-003-AC-4` | retained evidence read without a byte moving | deleted |
| `NFR-003` metrics | carried "Retained evidence bytes modified: 0" | row deleted |
| `TC-021` | matrix row and its test | both deleted |
| `SUITE-007` | registry row | deleted, with the reason stated in the registry rather than left as an unbacked row |
| `AA-001` "Retained Archive" | six records preserved byte-for-byte, all `incompatible` | replaced by a statement of the owner's release and the deletion, making no verification claim |
| `PROOF-legacy-compatibility` | obligation, `INPUTS` entry, `derive_result` branch, tool-identity probe | all deleted |
| `PRESERVE-legacy-bytes`, `PRESERVE-frozen-schemas` | preservation constraints | replaced by one `RELEASE-legacy-evidence` statement naming the authority |
| `UNKNOWN-derivation-evidence-not-mapped` | open, filed as engineering-assurance#21 | deleted; #21 closes as moot, not as fixed |

## Underspecified code

None found. The deletion removes code and its owning criteria together; no
surviving file lost its owner. `check_shared_pins.artifact_digest_mismatches` is
the one function whose population is now empty — it is still owned by
`FR-006-AC-1` and still runs, and the vacuity is SR-009 FND-901, recorded as an
open unknown rather than left for a reader to discover.

## Residual and out of scope

- `agent-ix/tl-mltl#14` — the Make execution-control class. The guard is
  deliberately **not** re-added by this change. The `ci` prerequisite list is
  unchanged at 14, so the recorded measurement still stands.
- `agent-ix/tl-mltl#11` — `da2c7704`. This change removes the last artifacts
  that require that revision to be fetchable. The issue stays open and the
  `retain/tl-mltl-v0.1-da2c7704` branch stays in place; disposing of both is
  handled separately once every repository's deletion has landed.
- `agent-ix/engineering-assurance#21` — the unmapped `quire.derivation-evidence`
  family. Moot here: the records it was filed about no longer exist.
- `agent-ix/engineering-assurance#20` — the pinned release records
  `pending_human_acceptance`. Unaffected and still reported, not gated on.
