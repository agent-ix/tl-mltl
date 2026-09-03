---
id: SR-011
title: Closing review of removal-census hardening
type: SpecReview
analysis: code-review
scope: tests/shared_assurance.rs, assurance/change-assurance.json, spec/requirements/FR-006-shared-assurance-intake.md, spec/test-matrix.md
review_set: all
---

# Closing review of removal-census hardening

## Summary

The independent review of PR #19 raised two high, five medium and five low
findings. Both highs and all five mediums are fixed. Four lows are fixed; L26 is
accepted because the Rust test already fails closed with distinct diagnostics,
while the twelve-state protocol belongs to Quoin intake rather than this static
repository-tree check. The tl-mltl half of tl-rewrite FND-1705 is also fixed.

## Scope and claim

This review dispositions the independent review of PR #19 at `4604314` and the
tl-mltl sibling item filed as FND-1705 on tl-rewrite PR #17. The change hardens
TC-024; it does not reinstate repository-local assurance tooling and it does not
claim to close the Make execution-control class in issue #14.

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1101 | high | H10 FIXED. The extension allow-list is deleted. Git enumeration includes `GNUmakefile`, `makefile`, included `.mk` fragments, extensionless paths and `.yaml`; a retained fixture presents a preferred `GNUmakefile` containing only `compat-view:` and requires that exact path and name to be observed. | TC-024 | implementation-bug-despite-evidence |
| FND-1102 | high | H11 FIXED. Tracked and untracked-not-ignored paths come from the same `git ls-files` helper. The three-path denial set and ten-area set are each compared with independent constants. | TC-024, FR-006-AC-7 | implementation-bug-despite-evidence |
| FND-1103 | medium | M16 FIXED. A temporary Git fixture drives the production enumerator over a preferred makefile and `.yaml`; the production exemption-plus-byte scanner sees every deleted needle after a non-UTF-8 byte; missing-file and non-repository failures are caught and checked by diagnostic. | TC-024 | correct-requirement-no-evidence |
| FND-1104 | medium | M17 FIXED. Area equality catches one-file areas; the tracked-only floor is a second, coarser control. | TC-024 | correct-requirement-no-evidence |
| FND-1105 | medium | M18 FIXED. Exemptions are anchored repository-relative paths: two exact declarations, plus `.md` only under `spec/reviews/` or `spec/plans/`. Positive and hostile path fixtures exercise the same classifier and the observed exact set must equal the two expected paths. | TC-024 | implementation-bug-despite-evidence |
| FND-1106 | medium | M19 FIXED. Counts come only from tracked paths reported by Git. No filesystem recursion follows symlinks and untracked files cannot pad the floor. | TC-024 | implementation-bug-despite-evidence |
| FND-1107 | medium | M20 FIXED within this change's boundary. TC-024 pins two stable clauses of the measured `.IGNORE:` disclosure, rejects live non-comment declarations of the named special controls, and requires exactly one literal `ci` declaration. Full Make qualification remains issue #14. | TC-024, issue #14 | correct-requirement-no-evidence |
| FND-1108 | low | L23 FIXED. The dead second Makefile scan is removed; every makefile and fragment is reached through the one repository census. | TC-024 | implementation-bug-despite-evidence |
| FND-1109 | low | L24 FIXED. The retained `GNUmakefile` control contains `compat-view:` without `legacy_evidence_view`, so the target needle is pinned independently. | TC-024 | correct-requirement-no-evidence |
| FND-1110 | low | L25 FIXED by deletion. `collect_sources` and its stale allow-list comments no longer exist. | TC-024 | implementation-bug-despite-evidence |
| FND-1111 | low | L26 ACCEPTED with rationale. Enumeration/read failures and forbidden-content failures all make the Rust test non-zero, while their diagnostics are distinct and asserted. The twelve-state vocabulary belongs to the Quoin intake scenarios; TC-024 is a test of the repository tree, not an intake producer, so manufacturing a second exit-code protocol here would duplicate the contract rather than strengthen it. | FR-006-AC-5, FR-006-AC-7 | correct-requirement-no-evidence |
| FND-1112 | low | L27 FIXED. FR-006-AC-7, TC-024, the sealed change-assurance declaration and this SpecReview all name the new control surface. No new acceptance criterion is needed: these are non-vacuity controls for the existing criterion, not a new product behaviour. | FR-006-AC-7, TC-024, assurance/change-assurance.json | correct-requirement-no-evidence |
| FND-1113 | medium | tl-rewrite FND-1705's tl-mltl half FIXED. The dangling-scenario and execution-boundary probes no longer symlink the repository `target`; each owns its Quoin store and shares only already-produced `target/assurance` inputs. The dangling fixture also proves the unmutated driver succeeds in the same scratch. | TC-019, TC-022 | implementation-bug-despite-evidence |

## Population and residual

At the final tracked tree, TC-024 expects 124 paths, denies exactly the root
lockfile and two licence texts, and scans 121 tracked paths plus every
untracked-not-ignored path. Ten tracked areas are required. `tests` contains 15
paths; removing it leaves 106, so the independently stated coarse floor is 107.

Issue #14 remains open. This review's line-level Make assertions prevent the
specific disclosure inversion found in review; they are not a substitute for
the later use-specific producer/runner qualification work in
`agent-ix/engineering-assurance#11`.

## Verification record

Exact-head commands and mutation results are recorded in PR #19 after the final
commit. Hosted CI remains manual-dispatch only and is not dispatched by this
change.
