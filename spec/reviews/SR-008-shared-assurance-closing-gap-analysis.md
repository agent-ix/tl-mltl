---
id: SR-008
title: Closing gap analysis of the tl-mltl shared assurance migration
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, assurance/**, scripts/**, examples/**, tests/**, corpus/**, Makefile, .github/workflows/ci.yml
review_set: all
---

# Closing gap analysis of the tl-mltl shared assurance migration

## Summary

Every finding from SR-005, SR-006 and SR-007 is dispositioned. Every task in
PLAN-002 is done. Every Test Matrix row is backed by a tracking tag in a test
Cargo compiles and runs, and the two deliberately unbacked suite rows are named
in `spec/evidence/suites.md` rather than left to look like an oversight.

| Disposition | Count | Findings |
|---|---|---|
| FIXED | 15 | FND-501…505, FND-701…711, FND-713 |
| ACCEPTED with rationale | 3 | FND-506 / FND-605 (frozen input schema), FND-712 (unrebuilt binary), FND-602 (external monitor not re-executed) |
| DEFERRED with a linked issue | 1 | FND-601 → `agent-ix/tl-mltl#14` |

Three further items are open upstream and are reported rather than worked
around: `agent-ix/engineering-assurance#20`, `agent-ix/engineering-assurance#21`,
and `agent-ix/tl-mltl#11`.

## Measurements at the final head

| Figure | Value | Population |
|---|---|---|
| Declared rows backed | 66 / 68 | every row Quire mints from `spec/`: 35 acceptance criteria, 24 test-matrix rows, 9 suite-registry rows |
| Suite-registry rows backed | 7 / 9 | `spec/evidence/suites.md`; `SUITE-001` and `SUITE-002` deliberately unbacked and named |
| Requirement-tagged Rust tests | 32, none ignored, none configured out | `scripts/rust_test_census.py` compared against `cargo test --list` |
| Rust tests executed by `make ci` | 32 | every test binary in the workspace; every one of them carries a `// Trace:` tag, so the two figures coincide |
| Producer rows | 61 | 14 reference-conformance + 36 R2U2 differential + 11 CLI conformance |
| Chain cases | 21 scenarios, 10 controls, 8 adapter probes | all matched |
| Verification outcomes demonstrated | 12 / 12 | each bound to the named case that owns it |
| Retained evidence files read | 283, 0 moved, 0 uncommitted | `evidence/` |
| Retained envelopes classified | 6 / 6 `incompatible` | the pinned PGM-01 mapping |
| Mutation probes | 36 of 37 detected | the one that is not is the recorded residual |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-801 | medium | The Make execution-control class is open. Measured: one `.IGNORE:` line takes `make ci` from exit 2 to exit 0 with 10 of 14 prerequisites neutered, and the chain's own exit-2 refusal is ignored with the rest. DEFERRED to agent-ix/tl-mltl#14 with a reproduction. | Makefile, spec/requirements/NFR-003-qualification-integrity.md | correct-requirement-no-evidence |
| FND-802 | low | Six of the eight claims the adversarial review was asked to break were attacked; the review names what it did not attack. Those areas — the pin classifier's incompatible path, `deny`, `audit-unsafe`, `fmt-check`, `lint`, `rustdoc`, `check-corpus`, `quire validate`, the `assurance-record` operator target — are unchecked rather than verified, and are recorded as such in SR-007. ACCEPTED. | spec/reviews/SR-007-shared-assurance-closing-code-review.md | correct-requirement-no-evidence |
| FND-803 | low | `agent-ix/tl-mltl#11` is untouched by this migration: `da2c7704` is reachable from no remote branch and this repository has zero tags, so `tl-rewrite`'s pin is still dangling. DEFERRED — resolving it requires creating a tag, which is outside this issue's authority. | — | missing-requirement |
| FND-804 | low | The three compiled example producers are attested with the crate version rather than a probed one. ACCEPTED: the crate version is a real fact about the binary that produced the bytes, and the driver now states that rather than implying every identity is probed. | scripts/assurance_chain.py | correct-requirement-no-evidence |
| FND-805 | low | No ix-flow decision event exists, so the receipt reads `incomplete` with `decision_missing`. ACCEPTED: that is the correct answer, and only the repository owner may create one. | assurance/change-assurance.json | correct-requirement-no-evidence |

## Requirement coverage at the final head

| Requirement | Backed by | What actually runs |
|---|---|---|
| FR-006-AC-1 | TC-018 | four components classified through the packaged matrix; the mirror scan's structural and file branches each seen to fire; collapsing the two tl-syntax revisions seen to be refused |
| FR-006-AC-2 | TC-019 | four independent demonstrations of the producer boundary: PATH stubs with an empty log; a `quoin` stub requiring failure; seven inputs moved aside requiring refusal; and an audit hook inside the driver exercised by injecting `quire coverage` |
| FR-006-AC-3 | TC-020 | impact-snapshot digest compared against the export; eleven requirements required by name; totals pinned with the population stated |
| FR-006-AC-4 | TC-021 | 283 files read, zero moved, Git consulted, five compatibility mutation probes required to be detected |
| FR-006-AC-5 | TC-022 | twelve states, each bound to the named case that owns it, plus the two audit-derived states required to carry their findings; seven negatives each required to have a positive control |
| FR-006-AC-6 | TC-023 | three comparison classifications, four external-monitor states, counts oracled from both corpus manifests, survival into retained bytes, and the unsupported case now performed rather than echoed |
| FR-006-AC-7 | TC-024 | fifteen removed paths absent; three schema digests pinned; a recursive census of the whole repository minus four named trees; a `make -n ci` graph check naming the test runner and the build line by command |
| NFR-002-AC-3 | TC-017 | census compared against `cargo test --list`, and shown able to parse it |
| NFR-003-AC-1..4 | TC-019, TC-021, TC-022 | — |

## Underspecified code

No file under `scripts/` or `examples/` lacks an owning requirement. Every one is
either named by a proof obligation in `assurance/change-assurance.json` or is
`check_unsafe_comments.sh`, which predates this change and is owned by NFR-001's
safety scaffolding.

## Verdict

**ACCEPT.** The plan is complete, the matrix is backed, and — unlike SR-006,
which could not establish this about itself — an independent review has attacked
the gates and the four defects it demonstrated are fixed and re-verified against
its own attacks. What remains open is recorded, linked, and not claimed closed.
