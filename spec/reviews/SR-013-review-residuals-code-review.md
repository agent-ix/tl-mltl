---
id: SR-013
title: Code review of shared-assurance residual fixes
type: SpecReview
analysis: code-review
scope: tests/shared_assurance.rs, assurance/change-assurance.json, spec/requirements/NFR-003-qualification-integrity.md, spec/test-matrix.md
review_set: all
---

# Code review of shared-assurance residual fixes

## Summary

The implementation closes the three medium and eight low mechanisms retained on
issue #20 after PRs #21 and #22. It changes only the shared-assurance test and
the specification/declaration artifacts that describe it. It adds no runner,
Make parser, evidence envelope or temporal-semantics behavior.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1301 | medium | M21-01 FIXED. Both the `requirements-assurance.txt` reader and its mutating negative probe hold the same process-global guard as the existing eight shared-input users. | TC-018, NFR-003 | implementation-bug-despite-evidence |
| FND-1302 | medium | M21-02 FIXED for every shared-state path. A private guard newtype can be constructed only from the process-global input mutex; `chain_report`, `producer_shims`, `run_chain_with_path`, the store-isolation helper, the shared-pin reader, the requirements-file mutation probe, and the census byte scanner all require that token, so deleting acquisition from an existing call site does not compile. | TC-018, TC-019, TC-024, NFR-003 | correct-requirement-no-evidence |
| FND-1303 | medium | M21-03 FIXED. A poisoned mutex now panics with a diagnostic that shared inputs may have been left mutated and names `make assurance-inputs`; it never silently recovers an ordinary measurement. | TC-019, NFR-003 | implementation-bug-despite-evidence |
| FND-1304 | low | M21-04/M21-05 FIXED. Both scratch probes call one containment helper; it canonicalizes an existing real store leaf, falls back only on `NotFound`, and fails on every other resolution error. Both cleanup sites state and implement the unlink-before-recursion boundary. | TC-019, TC-022 | correct-requirement-no-evidence |
| FND-1305 | low | M21-06 FIXED. Stale producer-shim cleanup accepts only success or `NotFound`; every other error is named and fatal. | TC-019 | implementation-bug-despite-evidence |
| FND-1306 | low | M21-07 FIXED. NFR-003-AC-2 and TC-019 now require the unmodified driver to succeed in the same owned scratch and require its Quoin store to lie outside the repository store. | NFR-003-AC-2, TC-019 | correct-requirement-no-evidence |
| FND-1307 | low | M22-01/M22-04 FIXED. The compatibility fixture contains only `.PHONY: compat-view`; a narrowed `compat-view:` needle was mutation-tested red. A hostile `GIT_TEMPLATE_DIR` and staged `core.excludesFile` each hide the preferred makefile until the explicit isolation control is present. | FR-006-AC-7, TC-024 | correct-requirement-no-evidence |
| FND-1308 | low | M22-02/M22-03 FIXED. Structured proof IDs containing `legacy-compat` are rejected, including a mutation with a `-v2` suffix; the declaration and SR-011 now state that top-level `census_controls` is an unsealed authorial cross-check. | FR-006-AC-7, TC-024 | implementation-bug-despite-evidence |

## Falsifiability checks

- Removing `--template=` while supplying the hostile fixture template makes
  TC-024 fail at the empty-template assertion.
- Narrowing both declared and executable needles to `compat-view:` makes TC-024
  fail because the phony-only preferred makefile no longer matches.
- Renaming an active proof to `PROOF-legacy-compatibility-v2` makes TC-024 fail
  at the structured proof-ID assertion.
- The focused shared-assurance binary passes 12/12 under normal parallel
  scheduling after the shared reader/writer pair is serialized.

## Verdict

READY FOR INDEPENDENT REVIEW. The known Make execution-control class remains on
#14 and is not weakened, widened or represented as closed here.

## 2026-09-06 post-merge amendment — PR #23 round three

The final independent review cleared PR #23 and left one medium and six low
follow-ups on issue #20. The repository-local findings are dispositioned here
without treating the original author review as merge authority.

| ID | Severity | Disposition | Evidence |
|---|---|---|---|
| M23R3-01 | medium | FIXED | `census_matches` and `deleted_names_in` now require the private guard token, making the census acquisition compile-time load-bearing. |
| M23R3-02 | low | FIXED | TC-024 compares exact tracked per-area cardinalities and drives a net-zero cross-area substitution that the total equality cannot detect. |
| M23R3-03 | low | FIXED | Both TC-018 Rust tests and TC-024 carry `NFR-003-AC-1`; the criterion and matrix reciprocate all three test IDs. |
| M23R3-04 | low | DEFERRED | The preferred-`GNUmakefile` execution-control bypass remains on tl-mltl#14 and Engineering Assurance #11; this change adds no Make parser. |
| M23R3-05 | low | FIXED | Exact population and per-area assertions run before guard acquisition, so ordinary source growth reports its own error without poisoning the shared-input mutex. |
| M23R3-06 | low | FIXED | This amendment records the floor-to-equality change, census guard, token enforcement, and per-area successor control in the review artifact SR-011 points readers toward. |
| M23R3-07 | low | ACCEPTED | NFR-002 now discloses the pre-existing pre-stable `NFR-002-AC-3` reuse and preserves the meaning of historical references rather than silently claiming an unbroken identity history. |

The load-bearing checks are explicit: removing the `inputs` argument from the
real census does not compile; replacing one tracked `tests/` path with one
synthetic `spec/` path preserves the total but changes the area map; and both
population assertions execute before the guard is constructed. Hosted CI
remains manual-only and was not dispatched.
