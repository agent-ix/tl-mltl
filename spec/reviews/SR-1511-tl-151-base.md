---
id: SR-1511
title: "Base review of P-TL-151 specification set (TL-427)"
type: SpecReview
analysis: base
scope: "TL-427..435 P-TL-151 specs across tl-syntax, tl-parse, tl-mltl, tl-solve (branch TL-151-spec)"
review_set: all
---

## Summary

Independent base review (checklist, ids, links, semantics) of the unbuilt, proposed P-TL-151 specs. No AssuranceProfile found in these repositories, so no profile governs. Method: the quoin:spec-review skill was invoked for the checklist and SpecReview contract; the seven lens skills were applied manually by the reviewer in the sibling documents (SR-1512 to SR-1518), not through ix-flow runs, and no external service (Jev for criterion-strength) was reachable, so SR-1519 is manual. LTLf rules of tl-syntax ADR-044 item 3 match De Giacomo-Vardi as the reviewer recalls them (strong next needs i<last, weak next true at last, R is the dual of U, CE-1 `G Xs true` and CE-2 `F Xw false` follow by derivation); the paper text itself was not available, so this is unverified against the source. Validate delta vs baseline: no new structural validation errors; new EARS warnings listed in SR-1513; strict coverage rows grow (tl-mltl 265 to 333, tl-solve 204 to 259) all unbacked because specs are unbuilt, and tl-solve strict coverage regresses from 0 to 53 unbacked rows (FND-15111).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-15111 | high | tl-solve new rows use status "🚧 not implemented" where the repo convention is "🚧 planned"; baseline strict coverage had 0 unbacked rows, the branch has 53 (FR-307-AC-3, FR-308-AC-1..3, FR-309-AC-1..3 and others). `make spec` would regress. Use the exempt status vocabulary. | tl-solve spec/ltlf-export-test-matrix.md |
| FND-15112 | high | TC-257 and TC-258 in tl-mltl TM-047 collide with TC-257/TC-258 of branch TL-147-spec (spec/solver-independence-test-matrix.md, FR-113). Renumber the whole TL-151 tl-mltl block TC-227..TC-258 to a free band (proposal TC-300..TC-331; main max 187, TL-149 uses 188-192, TL-150 uses 1570+). No other cross-branch id collision found for ADR-044/045 (tl-mltl, tl-syntax), FR-332.. (tl-syntax), FR-058.. (tl-parse): sibling branches use FR-362, FR-034, FR-302-311 (tl-syntax), FR-028-033 (tl-parse), FR-043, FR-053-060, FR-113 (tl-mltl). | spec/ltlf-interop-test-matrix.md:87-91 |
| FND-15113 | medium | The author's FR-092/TC-121/TC-122 concern: no in-repo collision. TC-121/TC-122 exist in tl-mltl main (spec/verification-effectiveness-test-matrix.md:57-58) and are not reused by TL-151 (new block starts at TC-227). Source cites "FSpec FR-092" (src/infinite/periodic.rs:9, tests/tc149_semantic_laws.rs:242, tests/infinite_trace.rs:325 "FR-092-AC-9") which is filament-specification FR-092, but the new tl-mltl FR-092 has AC-1..4 only, so trace tooling or readers can misattribute. Cross-repo homonyms: tl-parse new TC-121/TC-122 and TC-227..231 in tl-syntax and tl-mltl. Proposal: leave numbers, qualify the source comments as filament-specification FR-092, or move new tl-mltl WEST FR to a free id (FR-095+). | src/infinite/periodic.rs:9 |
| FND-15114 | medium | ADR-047 cites "five FR-341 labels" unqualified; FR-341 is a filament-specification id, not a tl-mltl requirement (spec/requirements/FR-041 relates to it). Qualify. Also `Disposition` lives in the feature-gated `infinite` module (src/infinite/mod.rs:97) so FR-085 totality needs the feature. | ADR-047, FR-085 |
| FND-15115 | medium | ADR-044/045 of tl-syntax are placed in new dir spec/decisions while existing tl-syntax ADRs live in spec/assurance (ADR-001, ADR-002). | tl-syntax spec/decisions |
| FND-15116 | low | Task files for PLAN-050 are boilerplate (identical subtasks); only the Notes differ. | PLAN-050 tasks |
