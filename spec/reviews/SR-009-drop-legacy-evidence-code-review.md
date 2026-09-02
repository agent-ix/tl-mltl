---
id: SR-009
title: Code review of the legacy-evidence deletion
type: SpecReview
analysis: code-review
scope: assurance/**, scripts/**, tests/**, spec/**/*.md, Makefile, .github/workflows/ci.yml, requirements-assurance.txt, CLAUDE.md
review_set: all
---

# Code review of the legacy-evidence deletion

## Summary

Reviewed the deletion under `agent-ix/tl-mltl#16` at `a4a2d05`. The change
removes 295 files — 7,575 deleted lines against 161 inserted — and the deletion
is irreversible in the working tree, so the review put its weight on one
question: **does anything still need what was removed?**

Six findings: **two medium, four low. Zero FIXED-after-the-fact — the two
mediums are reductions in detection that are accepted and recorded rather than
repaired**, because repairing either would mean keeping machinery for material
that no longer exists.

`make ci` exits 0 at the reviewed head with 31 Rust tests, `quire coverage`
reports 62 of 64 rows backed with the two long-standing deliberate exceptions,
and a whole-tree grep finds no live reference to the deleted material.

## Authority

`agent-ix/engineering-assurance#7`, section "Preservation constraint released for
the pre-stable phase". The repository owner decided on 2026-09-02; an agent
transcribed it. The epic's completion criterion and its mandatory control were
amended before this work, so no live constraint was violated. The constraint
re-applies unchanged at the move toward stable releases.

## What was removed, measured

| Item | Measure |
|---|---|
| `evidence/` | 283 tracked files, 842,668 bytes, 6 records |
| Retained envelope family | `quire.derivation-evidence/v1`, 6 of 6 |
| Mapping outcome at the pinned release | `incompatible`, 6 of 6, reason `unknown PGM-01 schema version` |
| True PGM-01 records held here | 0 |
| `scripts/legacy_evidence_view.py` | 465 lines |
| `tests/fixtures/legacy-compat/` | 8 files including `expectations.json` |
| `schemas/` | 3 files: 2 frozen schemas + README |
| Chain obligations | 7 → 6 |
| Acceptance criteria | 35 → 33 (`FR-006-AC-4`, `NFR-003-AC-4`) |
| Test-matrix rows | 24 → 23 (`TC-021`) |
| Suite-registry rows | 9 → 8 (`SUITE-007`) |
| Rust tests | 32 → 31 |

## Did anything still need it? — the checks that were run, not read

- **The two schemas were dead.** `grep -rn` for both filenames across the whole
  tree returned exactly four kinds of hit: `schemas/README.md` documenting the
  freeze, `assurance/change-assurance.json` stating the preservation constraint,
  `tests/shared_assurance.rs` pinning the digests, and the schemas' own `$id`.
  `build.rs` and `src/**` contain no `include_str!`, `include_bytes!` or
  validation call, and nothing in the repository imports `jsonschema`. The freeze
  list was derived from this tree, not inherited: the sibling that keeps a schema
  as a live output contract (`quire-contract-codegen`) has one this repository
  does not have.
- **The 12-state vocabulary survived the loss of the compatibility lane.**
  `TC-022` previously merged the census's mapped states into the demonstrated
  set. Every one of the twelve is bound to a *named* chain scenario or adapter
  probe, and `states_demonstrated` is built from the same matched cases, so the
  census contributed nothing the chain did not already own. Verified by the test
  passing with the census removed rather than by reading the code.
- **The chain still refuses a producer that did not run.** The "move each
  declared input aside in turn" probe covers all six remaining inputs; the
  seventh entry was the compatibility census.
- **The corpus basis is not evidence machinery.** `TL_SYNTAX_CORPUS_BASIS` names
  the revision whose bytes `corpus/tl-syntax-v1` retains, is verified by that
  corpus's own `SHA256SUMS`, is cross-checked against `Cargo.toml`,
  `Cargo.lock`, `src/lib.rs` and `corpus/README.md` on every run, and collapsing
  it into the compiled pin is refused. Its only evidence-side consumer was the
  deleted input schema's `tlSyntaxRevision` constant. **It stays.**
- **No orphan rows.** `quire coverage --scope . --strict` exits 0; `totals` reads
  62 backed of 64, and the two unbacked rows are `SUITE-001` and `SUITE-002`,
  which were already deliberate and are named in `spec/evidence/suites.md`.

## Mutation probes on the new assertions

`TC-024` grew four names and a whole-tree reference census. A deletion gate that
cannot fail is a comment, so each was measured:

| Probe | Result |
|---|---|
| `evidence/` recreated with one file | detected (assertion names `evidence`) |
| `schemas/` recreated with one schema | detected |
| `scripts/legacy_evidence_view.py` restored | detected |
| a `legacy_evidence_view` reference appended to `build.rs` | detected by the census |

`build.rs` is the deliberate target: it executes on every cargo invocation and an
earlier census that walked named directories did not see it (SR-007 FND-705).

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-901 | medium | `assurance/pins.json` declared four digest-bearing consumed artifacts — `verification_semantics.py`, the compatibility-view schema and the two PGM-01 control fixtures — and the deleted view was their only reader. They are removed, so this repository now reads exactly one file from the pinned distribution, `compatibility-matrix.json`, which deliberately carries no local digest. `check_shared_pins.artifact_digest_mismatches` therefore has an empty population and `TC-018`'s `artifact_mismatches` assertion is now vacuous; SR-007's probe P29 can no longer be run. ACCEPTED — the alternative is pinning digests for files nobody reads, which is a false statement about what the repository consumes. Recorded as `UNKNOWN-consumed-artifact-digests-now-vacuous`. | assurance/pins.json, scripts/check_shared_pins.py | correct-requirement-no-evidence |
| FND-902 | medium | The detection surface shrinks. Six of SR-007's 37 mutation probes went with the deleted material: P06 (census `matched=false`), P25 (a retained evidence byte modified), P26 (a derived fixture edited off its derivation), P29 (a consumed-artifact digest changed), P32 (a frozen schema edited) and P33 (a frozen schema referenced by a script). ACCEPTED — each guarded only material that no longer exists — and partially offset by the four new probes above, which are measured rather than asserted. Net: 37 probes become 35, of which 34 detect and the one that does not is the recorded `.IGNORE:` residual, unchanged. P34, "deleted machinery restored under its old name", survives and now covers four more names. | tests/shared_assurance.rs | correct-requirement-no-evidence |
| FND-903 | low | `FR-006` and `NFR-003` now have gaps in their acceptance-criterion numbering (`FR-006-AC-4`, `NFR-003-AC-4`). ACCEPTED: renumbering would silently repoint identities that sealed records, the test matrix and four prior reviews name. Stable IDs with a visible gap beat a tidy sequence that means something different than it did. | spec/requirements/FR-006-shared-assurance-intake.md, spec/requirements/NFR-003-qualification-integrity.md | wrong-requirement |
| FND-904 | low | `NFR-003-AC-4` also carried "no local digest claims external attestation or release authority", which is broader than the retained-evidence claim it was deleted with. ACCEPTED: it is still carried by the `Automatic release decisions: 0` metric in the same requirement, by `NFR-003`'s Qualification Boundary section, and by `AA-001`'s closing paragraph, none of which changed. | spec/requirements/NFR-003-qualification-integrity.md | missing-requirement |
| FND-905 | low | Three prose mentions of `da2c7704` survive, in `PLAN-001`'s log, its Task-006, and SR-008. ACCEPTED: they are historical records of work that happened, not pins, and none requires the commit to be fetchable. No pin, dependency, retained record or gate references it any more. | spec/plans/**, spec/reviews/SR-008-shared-assurance-closing-gap-analysis.md | wrong-requirement |
| FND-906 | low | `quire validate` emits two advisories at this head — `status-column-matches-nothing` on the Functional Requirement Coverage table and `archetype-matches-nothing` for the `Inspections` archetype. ACCEPTED: both reproduce at `f7eb8bd` and are untouched by this change. Reported so a reader does not attribute them to it. | spec/test-matrix.md | correct-requirement-no-evidence |

## What this review did not do

Recorded because an unattacked claim should not read as a verified one. Hosted CI
was not dispatched and remains `workflow_dispatch`-only. Quoin, Quire and
engineering-assurance internals were not audited. The `.IGNORE:` measurement was
not re-run — the `ci` prerequisite list is unchanged at 14 and the residual is
`agent-ix/tl-mltl#14`, whose guard this change deliberately does not re-add. The
domain gates (`deny`, `audit-unsafe`, `fmt-check`, `lint`, `rustdoc`,
`check-corpus`, the three producers) were run but not attacked; this change does
not touch them.

## Verdict

**ACCEPT.** Nothing in the tree still needs the deleted material, and that was
measured four ways rather than argued. Two mediums are accepted reductions in
detection, both recorded in the change-assurance declaration rather than
smoothed over. No record was rewritten, backdated or re-sealed, and no claim that
argued from the retained evidence was restated in a weaker form.
