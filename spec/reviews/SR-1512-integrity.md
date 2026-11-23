---
id: SR-1512
title: "Integrity analysis of P-TL-151 specs"
type: SpecReview
analysis: integrity
scope: "ltlf-interop spec: ADR-044..047, FR-083..094, NFR-047, TM-047, PLAN-050; cross-repo ADR-044/045 tl-syntax, ADR-041 tl-parse, ADR-300..302"
review_set: subset
---

## Summary

Counts recomputed independently: 4+5x4+6x16=120 depth-one formulas; 4+16+64+256=340 traces; 40,800 and 40,820 rows; 40x2x340=27,200 closure comparisons; 24 hostile reader cases; 296 export inputs; 116 infinite-profile formulas; tl-parse hostile family 29 entries. All arithmetic holds. Inconsistencies are in the criteria below.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15121 | high | tl-parse FR-061-AC-1 states 3 accepted and 26 refused "under each profile" but the corpus mixes profile-specific entries: bare `X` is refused under LTLf and accepted under the infinite profile, `Xs`/`Xw` the reverse, so per-profile counts differ (LTLf accepts 3 exact-limit plus Xs and Xw cases; infinite accepts 3 plus X). | tl-parse FR-061 lines 24-34 |
| FND-15122 | medium | FR-090-AC-1 says 12 formulas, 4 designed for each of three classes, "one of them carrying two classes"; that is 11 or 13 distinct formulas, not 12. | FR-090 |
| FND-15123 | medium | FR-089 demands upstream repository, path, revision and licence for WEST artifacts, but tl-rewrite corpus/west-v1/manifest.json records upstream, upstreamPath, license, licenseFile and no revision. FR-092 says "upstream revision as recorded by tl-rewrite"; none is recorded. FR-089-AC-1 would refuse the WEST corpus. | FR-089, FR-092, tl-rewrite corpus/west-v1/manifest.json |
| FND-15124 | medium | FR-086 says C2PO grammar "R2U2 4.2" while retained corpus/r2u2-v4.2/manifest.json pins C2PO tool 4.1.0 with R2U2 4.2-release; the past corpus has `O[1,1](p)` twice (past.c2po lines 4 and 7) so 12 formulas are 11 distinct. Fix wording and census rule. | FR-086, corpus/r2u2-v4.2/manifest.json |
| FND-15125 | medium | ADR-047 says `Pending` is unlike inconclusive ("more input may decide it") and then maps Pending to LTL3 inconclusive; and says tl-solve `Inconclusive` has "the same meaning" although it denotes a bounded miss. State mapping as nearest, not same. | ADR-047 |
| FND-15126 | medium | tl-solve ADR-302 verdicts are sat/unsat/unknown/error; FR-303 adds timeout. Align. | ADR-302, FR-303 |
| FND-15127 | low | tl-syntax FR-333 wire tags `StrongNext`/`WeakNext` (CamelCase) contradict the existing wire convention `serde(tag="kind", rename_all="snake_case")` (src/formula/graph.rs:193-197), which yields `strong_next`. | tl-syntax FR-333 |
| FND-15128 | low | ADR-046 heading title differs from frontmatter-level scope (heading adds WEST); minor. | ADR-046 |
