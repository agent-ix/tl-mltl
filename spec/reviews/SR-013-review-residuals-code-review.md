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
| FND-1302 | medium | M21-02 FIXED for every shared-state path. A private guard newtype can be constructed only from the process-global input mutex; `chain_report`, `producer_shims`, `run_chain_with_path`, the store-isolation helper, the shared-pin reader and the requirements-file mutation probe all require that token, so deleting acquisition from an existing call site does not compile. | TC-018, TC-019, NFR-003 | correct-requirement-no-evidence |
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
