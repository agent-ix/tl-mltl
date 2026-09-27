---
id: SR-071
title: TL-218 gap-analysis review
type: SpecReview
analysis: gap-analysis
scope: agent-ix/tl-mltl@c4cdfc56feb6d8a7b0e2e784b6b301d72ca71db5; spec/requirements/FR-040-infinite-safety-export.md,
  spec/requirements/FR-041-c2po-refusal-partition.md, spec/r2u2-v1-test-matrix.md,
  tests/infinite_export.rs, tests/tc138_feature_boundary.rs, src/infinite/export.rs
review_set: subset
---

## Summary

Focused acceptance-criteria-to-test audit for TC-168 through TC-173; unrelated repository coverage rows are outside this feature review. Ticket: TL-218. Provisional base: 4bcea84bbe3fa7360076ea51edaf7f23a094500f.

## Verdict

**FAIL** — Substantive finding(s) require a fix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-172 claims complete node × interval × context coverage but exercises only [0,0] and [0,) in a single inner context; it misses nonzero closed windows, shifted unbounded windows, outer-context variants and combinations, so the matrix’s complete-census claim is unbacked. | tests/infinite_export.rs:578 |
| FND-002 | medium | TC-170 constructs the target verdict Boolean in the test itself and supplies no retained C2PO step for the newly exported safety expression; it validates provider replay with a synthetic claim, but cannot detect an actual target-vs-provider disagreement. | tests/infinite_export.rs:475 |

## Coverage

Examined: spec/requirements/FR-040-infinite-safety-export.md, spec/requirements/FR-041-c2po-refusal-partition.md, spec/r2u2-v1-test-matrix.md, tests/infinite_export.rs, tests/tc138_feature_boundary.rs, src/infinite/export.rs. Focused cargo test 14/14, cargo fmt and focused Clippy passed; no aggregate gate was run.

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4643c76d4d46730a5de9f50f47ebfc178b8f9e0d — TC-172 adds 171 node/interval/placement and 114 temporal-nesting cells with exact mapped/refused assertions. |
| FND-002 | fixed | 4643c76d4d46730a5de9f50f47ebfc178b8f9e0d — TC-170 binds the exported O[0,1](p) body to SHA-checked retained C2PO source and R2U2 output, replaying three recorded steps. |

## Disposition verdict

**PASS** — no open gap finding on this head. No aggregate gate was run.

## Retargeted-head recheck

**PASS at `a0b3a42ec721217457d9be2a325d5df78d47080b` against landed main `6d8e1ad4c1a86a09d3ed52a4a6df6a98ed0f2596`.** Rechecked FR-040/041 against TM-005 and TC-168–173 tags at the rebased head. The previously missing census and retained target-step evidence remain present, and the focused tests pass. TC-138/171 external-consumer test now pins landed syntax and passes feature on/off. Prior FND-001/002 remain fixed; no new evidence gap.
