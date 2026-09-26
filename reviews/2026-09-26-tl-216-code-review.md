---
id: SR-055
title: TL-216 past C2PO mapping code review
type: SpecReview
analysis: code-review
scope: "agent-ix/tl-mltl@18c1d0bebcc980d114bd3e3a967c01f39409add8; TL-216; src/lib.rs; src/mapping/{mod,legacy,past}.rs; tests/{past_mapping,past_c2po_corpus,c2po_map_fuzz,future_parity}.rs; fuzz/{Cargo.toml,Cargo.lock,README.md,fuzz_targets/c2po_map.rs,corpus/c2po_map/*}; corpus/past-c2po-v1/*; spec/{requirements/FR-038-past-c2po-export.md,requirements/FR-039-past-origin-contract.md,r2u2-v1-test-matrix.md}; .gitignore"
review_set: subset
---

# TL-216 past C2PO mapping code review

## Summary

Rust lane and language-independent code review of PR #98 at the frozen head. No applicable AssuranceProfile was found. The diff does not change CI workflow files. The retained corpus is required by FR-038/039 and is feature evidence, not a TL-217 campaign.

## Verdict

**FAIL.** The mapper admits target-origin cells for which this PR retains no matching target observation or guard proof. Focused tests, formatting, Clippy, and fuzz compilation passed per coder; the aggregate guarded-ci gate remains incomplete and was not retried because it invokes halted assurance work.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | The reviewed target contract admits H[0,0], S[0,1], and T[0,0] from a hardcoded partition, but the retained R2U2 run compares mapper output only for O[0,1] and Y; the claimed broader origin grid is absent. A target mismatch in an admitted cell would export executable C2PO despite FR-039's refusal rule. | src/mapping/past.rs:320; tests/past_c2po_corpus.rs:344; FR-039-AC-1 |

## Rust checks

Reviewed public API, typed refusals, graph validation, bounded renderer, exhaustive node matches, identifier binding, and source-level T duality test. No new unsafe, production panic, CI weakening, or test seam bypass observed. The feature corpus is generated target observation rather than copied upstream source. The high finding concerns the oracle's coverage of admitted behavior.
