---
id: SR-056
title: TL-216 feature gap analysis
type: SpecReview
analysis: gap-analysis
scope: "agent-ix/tl-mltl@18c1d0bebcc980d114bd3e3a967c01f39409add8; TL-216; FR-038; FR-039; TM-005; src/mapping/past.rs; tests/{past_mapping,past_c2po_corpus,c2po_map_fuzz,future_parity}.rs; fuzz/fuzz_targets/c2po_map.rs; corpus/past-c2po-v1/*"
review_set: subset
---

# TL-216 feature gap analysis

## Summary

Compared FR-038/039 acceptance criteria, TM-005, production admissions, tagged tests, the retained target run, and the committed fuzz target. No targeted plan bundle was supplied; task completion was assessed against TL-216's stated exit criteria. Semantic intent alignment was examined as part of the requested strong-oracle review.

## Verdict

**FAIL.** Admitted target-origin cells lack retained target observations, and TC-163 is unbacked by a tagged test. Repository-wide Quire coverage reports unrelated pre-existing rows; this verdict is restricted to TL-216.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-001 | high | FR-039-AC-1's “every admitted past operator” evidence is not backed for H[0,0], S[0,1], or T[0,0]. The retained corpus compares mapper output to target source only for O[0,1] and Y; the remaining admissible cells are established by a cited but unretained campaign. | FR-039-AC-1; src/mapping/past.rs:324; tests/past_c2po_corpus.rs:344 |
| FND-002 | medium | TC-163 claims byte-identical legacy future manifests through existing fixtures, but no test code carries TC-163 and the added future-parity test only updates a source-count assertion. The requirement's byte-stability criterion has no feature-specific tagged oracle. | spec/r2u2-v1-test-matrix.md:19; FR-038-AC-2; tests/future_parity.rs |

## Coverage

TM-005 lists TC-160 through TC-167 and TC-174. TC-163 has no matching source test tag. `quire coverage --scope . --json` ran; its broad 119/235 rollup includes many unrelated historical matrices, so it is not used as the TL-216 denominator. FR-042 tags in new tests have no local FR-042 document and should be removed from this feature or given a real owning requirement before claiming them.
