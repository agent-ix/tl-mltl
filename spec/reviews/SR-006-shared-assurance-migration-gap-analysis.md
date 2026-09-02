---
id: SR-006
title: Gap analysis of the tl-mltl shared assurance migration
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, assurance/**, scripts/**, examples/**, tests/**, Makefile, .github/workflows/ci.yml
review_set: all
---

# Gap analysis of the tl-mltl shared assurance migration

## Summary

Every task in PLAN-002 is done, every Test Matrix row is backed by a real
tracking tag in a test Cargo compiles and runs, and the two rows that are
deliberately unbacked are named. This document records what is covered, what is
deliberately not, and what a reader should not mistake for coverage.

Coverage at the migration head, measured with `quire coverage --scope . --json`:

| Figure | Value | Population |
|---|---|---|
| Matrix rows backed | 66 / 68 | every row Quire mints from `spec/`, including requirement acceptance criteria, test-matrix rows and suite-registry rows |
| Suite-registry rows backed | 7 / 9 | `spec/evidence/suites.md` |
| Acceptance criteria classified | 35 | `coverage.criteria` |
| Requirement-tagged Rust tests | 20, none ignored, none configured out | `scripts/rust_test_census.py`, compared against `cargo test --list` |
| Rust tests executed by `make ci` | 30 | every test binary in the workspace |

The two unbacked rows are `SUITE-001` and `SUITE-002`, and `spec/evidence/suites.md`
says why in its own voice: `SUITE-001` is `make ci`, the composite that contains
every other suite, so a test that ran it would be a test the composite runs;
`SUITE-002` is the `quire validate` half of `make spec`, which writes no
structured result for anything to read.

## The registry got smaller and that is the improvement

Before this change, all seven suite rows were backed — by **one** test. A single
`// Trace:` comment in `tests/evidence_contract.rs` named `SUITE-001` through
`SUITE-007` at once. The count was 9/9 and the content was one assertion.

Seven rows are now bound by tests that invoke the suite's own command. Four
bindings were lost and seven real ones gained. A reader comparing 9/9 against
7/9 should read the second number as the larger one.

## Plan completion

| Task | Status | Evidence |
|---|---|---|
| Task-001 inventory, pins, upstream repin | done | `assurance/pins.json`; `check_shared_pins.py` classifies four components through `engineering_assurance.compatibility`; `@agent-ix/quoin` repinned 0.22.5 → 0.23.1 and `ix-flow@0.0.4` added; tl-syntax compiled pin moved to `953ee825` on main with the corpus basis held at `740182f1` and both cross-checked |
| Task-002 producers, adapter, intake | done | three producers under `examples/`; `assurance_chain.py` seals, retains and verifies through the pinned Quoin CLI; `legacy_evidence_view.py` reads 283 evidence files through the pinned mapping |
| Task-003 dual run, deletion, residual | done | dual run recorded in SR-005; 2,769 lines deleted in a separate commit afterwards; the Make residue measured and filed as `agent-ix/tl-mltl#14` |

## Requirement coverage

| Requirement | Backed by | Note |
|---|---|---|
| FR-006-AC-1 | TC-018 | Also exercises the mirror scan's file branch and the collapsed-revision refusal, so both are seen to fire rather than assumed |
| FR-006-AC-2 | TC-019 | Three runs: producers stubbed and the log required empty; `quoin` stubbed and the chain required to fail; each of seven declared inputs moved aside and the driver required to refuse |
| FR-006-AC-3 | TC-020 | Impact snapshot digest compared against the export; export required to name all eleven requirements and to carry the pinned totals |
| FR-006-AC-4 | TC-021 | 283 files read, zero moved, Git asked whether they are the committed bytes, five compatibility mutation probes required to be detected |
| FR-006-AC-5 | TC-022 | Twelve states, each counted only when a case produced it AND matched; seven named negatives each required to have a positive control |
| FR-006-AC-6 | TC-023 | Three comparison classifications and four external-monitor states required; counts oracled from the two corpus manifests |
| FR-006-AC-7 | TC-024 | Fifteen removed paths asserted absent; three schema digests pinned; source census over seven directories plus four root files |
| NFR-002-AC-3 | TC-017 | The census is also shown to parse Cargo's own `--list` output, so it cannot compare an empty set against an empty set and agree |
| NFR-003-AC-1..4 | TC-019, TC-021, TC-022 | — |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-601 | medium | The Make execution-control class is open: one `.IGNORE:` line takes `make ci` from exit 2 to exit 0 with 10 of 14 prerequisites neutered, and the structural replacement covers only the seven proofs that feed the chain. DEFERRED to agent-ix/tl-mltl#14. | Makefile, spec/requirements/NFR-003-qualification-integrity.md | correct-requirement-no-evidence |
| FND-602 | low | The external monitor is never re-executed, so this repository's compatibility claim is about one retained exchange at R2U2 4.2-release rather than a continuously re-verified one. ACCEPTED: a gate that ran the monitor would produce the thing it checks. | corpus/r2u2-v4.2/, assurance/change-assurance.json | correct-requirement-no-evidence |
| FND-603 | low | All six retained envelopes classify `incompatible` under the pinned PGM-01 mapping because their family is `quire.derivation-evidence/v1`. ACCEPTED and reported as it stands; filed upstream as agent-ix/engineering-assurance#21. | scripts/legacy_evidence_view.py | missing-requirement |
| FND-604 | low | The pinned Engineering Assurance release records `pending_human_acceptance` and ships no `human_acceptance_recorded` predicate. ACCEPTED: reported, never gated on, and no branch head pinned. Filed as agent-ix/engineering-assurance#20. | scripts/check_shared_pins.py | missing-requirement |
| FND-605 | medium | The input schema's freeze preserves an identity the retained records name rather than the bytes they name. ACCEPTED with all three digests pinned by TC-024. See SR-005 FND-506. | schemas/README.md | implementation-bug-despite-evidence |
| FND-606 | low | No ix-flow decision event exists, so the verification receipt reads `incomplete` with `decision_missing`. ACCEPTED: that is the correct answer, and only the repository owner may create one. | assurance/change-assurance.json | correct-requirement-no-evidence |

## Gaps, stated rather than closed

**G-1. The Make execution-control class is open.** Measured here: with
`src/lib.rs` made uncompilable, `make -k ci` exits 2 and 10 of the 14 `ci`
prerequisites do not complete; adding one `.IGNORE:` line makes all 10 report
success and `make ci` exits 0, and the run log records the chain's own exit-2
refusal being ignored along with everything else. The structural replacement
covers only the seven proofs re-run inside `assurance-inputs`. Tracked as
`agent-ix/tl-mltl#14`. Not closed, and not claimed to be.

**G-2. The external monitor is not re-executed.** R2U2 4.2 and C2PO 4.1.0 ran
once, out of band. Every claim here is a replay against retained, digest-pinned
bytes. That is deliberate — a gate that ran the external monitor would be
producing the thing it checks — and the consequence is that a change in R2U2
behaviour after `4.2-release` would not be observed. Recorded as
`UNKNOWN-external-monitor-not-re-executed`.

**G-3. The retained records classify `incompatible`.** All six envelopes are
`quire.derivation-evidence/v1`; the pinned PGM-01 mapping covers
`quire.pgm01-evidence` v1 and v2 only. Measured in this repository rather than
inherited: the same question returns a mapped result in `quire-contract-ir` and
`unreadable` in `quire-analyze`. Filed upstream as
`agent-ix/engineering-assurance#21`. No local mapper was written.

**G-4. The acceptance state is reported, not gated on.** `engineering-assurance`
v0.2.0 records `pending_human_acceptance` and ships no
`human_acceptance_recorded` predicate; both landed on that repository's main
after the tag and no v0.2.1 was cut. This repository reports what the installed
release records and reads an absent field as neither approval nor rejection.
Filed as `agent-ix/engineering-assurance#20`. No branch head was pinned.

**G-5. The input schema's freeze is weaker than the manifest schema's.** See
SR-005 FND-506. Accepted with the divergence pinned by TC-024 rather than
smoothed over.

**G-6. No human decision exists.** The receipt this change produces reads
`incomplete` with `decision_missing`, which is the correct answer while no
ix-flow decision event exists. Only the repository owner may create one, and none
was synthesized.

## Underspecified code

`scripts/rust_test_census.py` is the only script in the tree that is neither a
producer named by a proof obligation nor part of the shared-assurance path — it
is both, being `PROOF-test-census`'s producer and a `make` target. Every other
file under `scripts/` and `examples/` is named by a proof obligation in
`assurance/change-assurance.json`. No file was found without an owning
requirement.

## Verdict

CONDITIONAL, pending the independent adversarial review recorded in SR-007. The
plan is complete and the matrix is backed; what this document cannot establish
about itself is whether any of the gates it counts is a false green.
