---
id: SR-052
title: "Base review — retire PGM-01 citation and FR-019 (TL-180)"
type: SpecReview
analysis: base
scope: "spec/spec.md; spec/requirements/FR-019-consume-qobs-c00.md; spec/requirements/NFR-002-governance-boundary.md; spec/decisions/ADR-001-retire-pgm01-citation-and-fr-019.md"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-mltl/FR-019
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-002
    type: references
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: references
  - target: ix://agent-ix/quire-mltl/FR-002
    type: references
---

# Base review — retire PGM-01 citation and FR-019 (TL-180)

## Summary

Reviewed the TL-180 spec correction: MRS-001's removed `quire-contract-ir/PGM-01`
`depends_on` edge and PGM-01-citing prose, FR-019's `status: superseded`
transition, NFR-002's removed PGM-01 reference and its reword onto NFR-003,
and the new ADR-001 recording the architect's TL-175 ruling. This is the
reversal record for SR-044 through SR-051, which reviewed and accepted the
original FR-018/FR-019/PGM-01-citing work in PR
[#67](https://github.com/agent-ix/tl-mltl/pull/67). SR-044 through SR-051 are
historical record and are not edited by this review or by TL-180 — this
org's convention, per `quire-contract-ir`'s own PGM-01-R08/R09 withdrawal
notes, is that a frozen SpecReview stands as-is even when the decision it
reviewed later reverses.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-5201 | low | No issues found: MRS-001's `depends_on: ix://agent-ix/quire-contract-ir/PGM-01` edge, its Purpose-section PGM-01 sentence, and its References-section PGM-01 link are all removed; no other PGM-01 citation remains in `spec/spec.md`. | spec/spec.md |
| FND-5202 | low | No issues found: FR-019 is marked `status: superseded`, gains a `references` edge to `ix://agent-ix/quire-mltl/FR-002`, and its new `## Supersession` section names the successor and the architect ruling without deleting or rewriting its Description, Inputs, Outputs, Behavior, Constraints, or Acceptance Criteria. | spec/requirements/FR-019-consume-qobs-c00.md |
| FND-5203 | low | No issues found: NFR-002's `references: quire-contract-ir/PGM-01` frontmatter edge and its "canonical PGM-01 evidence boundaries" / "Applies PGM-01 to FR-004, FR-005..." prose are both replaced with a `references` edge and prose pointing at NFR-003, which already owns the shared-assurance intake path this repository actually runs; the reword introduces no duplication of NFR-003's own content. | spec/requirements/NFR-002-governance-boundary.md |
| FND-5204 | low | No issues found: ADR-001 cites the correct Linear link (`https://linear.app/agent-ix/issue/TL-175`), not `agent-ix/tl-mltl#7` — the exact mistake this org caught and corrected during TL-177's review — and cites PR #67, FR-018, FR-019, and SR-044 through SR-051 as the work being reversed. | spec/decisions/ADR-001-retire-pgm01-citation-and-fr-019.md |
| FND-5205 | medium | Accepted, not a defect: `tl-mltl`'s code (`Cargo.toml`, `src/wire/{request,observation,report}.rs`, `src/mapping/contract_ir.rs`) still depends on `quire-observation` and still implements FR-019's boundary; TC-085 still exercises it and `spec/test-matrix.md`'s FR-019 row is left `✅ covered` because that remains true today. This spec/code divergence is expected of spec-first work and is closed by TL-179's PR B, tracked separately and out of this ticket's scope. | Cargo.toml; src/wire/observation.rs; spec/test-matrix.md |
| FND-5206 | low | No issues found: SR-044 through SR-051 are byte-unchanged by this review and by TL-180; this document is the new review that records the reversal, consistent with the frozen-SpecReview convention. | spec/reviews/SR-044-temporal-owner-code-review.md through spec/reviews/SR-051-qobs-c00-rust-review.md |

## Verdict

**PASS.** The spec now states tl-mltl's target/retired posture toward PGM-01
and FR-019 without erasing the historical record of either, and without
touching the four wire/mapping source files, `Cargo.toml`, or README.md's
AGPL paragraph — all out of this ticket's scope. `quoin review`'s full
`ix-flow` workflow could not be run in this environment (a known, previously
confirmed issue: a dev build of `quoin` pointing at a deleted worktree path);
this review instead used `quire validate --scope . "spec/**/*.md"` and `quire
coverage --scope . --strict` directly, both of which ran standalone.
