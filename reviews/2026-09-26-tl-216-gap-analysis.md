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

## Dispositions

| FND | Outcome | SHA/reason |
|---|---|---|
| FND-001 | fixed | fixed 6707744abb0d59e6b16115f30b61238db6a83fe5: Zero-width H/S/T lowering removes unevidenced target past operators and S[0,1] refuses. |
| FND-002 | fixed | fixed 6707744abb0d59e6b16115f30b61238db6a83fe5: TC-163 now tags byte-exact v1/v2 manifest fixture assertions. |
| FND-003 | still-open | Changed-head TC-163 v1/v2 assertions fail at 1bf6257 after the landed syntax repin. |

## New findings (disposition pass 2)

At changed head `1bf6257e710654293d0c3370bca668ef7f670e6b`, TC-163 remains tagged, but its two backing assertions fail after the compiled TL-15 repin. TC-160/161/164/165/166/167 and TC-174 focused past tests pass 13/13.

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-003 | high | TC-163 is present but fails in both v1 and v2 future manifest tests because its committed expected `syntaxRevision` is the pre-repin `4a561419` while the actual artifact records landed TL-15 `6aa9b11`. Therefore FR-038-AC-2 does not yet have passing byte-stability evidence on this head. | FR-038-AC-2; TC-163; tests/fixtures/tl-216/legacy-v1.json:1; tests/fixtures/tl-216/legacy-v2.json:1; tests/interop.rs:57; tests/contextual.rs:249 |

## Changed-head verdict

**FAIL at `1bf6257`** with FND-003 open. The earlier FND-001/002 dispositions remain fixed.
