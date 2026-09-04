---
id: SR-014
title: Gap analysis of shared-assurance residual fixes
type: SpecReview
analysis: gap-analysis
scope: spec/**/*.md, assurance/change-assurance.json, tests/shared_assurance.rs, agent-ix/tl-mltl#20
review_set: all
---

# Gap analysis of shared-assurance residual fixes

## Summary

All eleven residuals recorded on issue #20 have a concrete implementation and a
declarative owner. Quire validates the enlarged specification and reports 62/64
rows backed, 23/23 Test Matrix rows backed, all 33 acceptance criteria property-
and specificity-shaped, and 31/31 Rust evidence symbols read, authored and
compiled. The two intentionally unbacked suite rows remain the composite and
the static validator, as documented in the suite registry.

## Plan completion

| Plan item | Status | Evidence |
|---|---|---|
| Guard coverage and poison handling | done | Eleven shared-state tests use the private guard token; the Rust-test census alone is unguarded because it touches neither shared input; poison is a named panic |
| Symmetric scratch isolation | done | Both probes call `assert_probe_store_isolated`; existing real leaf canonicalized, absent leaf handled, other errors fail closed |
| Census falsifiability | done | Phony-only plain-name fixture; hostile template mutation; staged excludes miss/override; substring proof-ID rejection |
| Specification and retained record | done | NFR-003-AC-2, TC-019, declaration purpose and SR-011 agree with the implementation |
| Local verification | done | focused TC-024 pass; three named red mutations; shared-assurance 12/12; `make spec` 62/64 and Rust 31/31; complete local `make ci CARGO_TARGET_DIR=target/cargo-review` pass |

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-1401 | medium | The Make execution-control class remains open and is explicitly outside this repository-local change. DEFERRED to tl-mltl#14 and engineering-assurance#11. | Makefile, NFR-003 | correct-requirement-no-evidence |
| FND-1402 | low | Two mechanisms recur in sibling repositories and are not closed by a tl-mltl-only implementation. DEFERRED to repository-specific follow-ups in tl-parse and tl-rewrite. | agent-ix/tl-mltl#20 | implementation-bug-despite-evidence |
| FND-1403 | low | Hosted CI is manual-only and has not been dispatched. ACCEPTED: the exact-head local gate and independent review are the landing evidence for this build-out phase. | .github/workflows/ci.yml | correct-requirement-no-evidence |

## Remaining gaps

**G-1 — Make execution controls.** Issue #14 remains open by design. This
change neither introduces another repository-local parser nor claims that
`make ci` is a trust root. The later use-specific qualification design remains
`agent-ix/engineering-assurance#11`.

**G-2 — Cross-repository siblings.** `tl-parse` shares the requirements-file
reader/writer race and existing-real-store leaf case; `tl-rewrite` shares the
store-leaf case and narrowed compatibility-target needle. Those are separate
repository changes and are not represented as closed by this PR.

**G-3 — Hosted execution.** Hosted CI remains manual-dispatch only and is not
run by this work. The final exact-head local gate and independent review are
recorded on the PR before landing.

## Verdict

READY FOR INDEPENDENT REVIEW. No local issue-#20 item is knowingly unimplemented;
the remaining gaps have explicit owners outside this change's boundary.
