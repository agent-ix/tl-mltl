---
id: SR-070
title: TL-218 code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/tl-mltl@c4cdfc56feb6d8a7b0e2e784b6b301d72ca71db5; src/infinite/export.rs,
  src/infinite/mod.rs, src/mapping/mod.rs, tests/infinite_export.rs, tests/tc138_feature_boundary.rs,
  src/mapping/past.rs
review_set: subset
---

## Summary

Rust/code review of the TL-218 export and its TL-216 target-origin seam. Ticket: TL-218. Provisional base: 4bcea84bbe3fa7360076ea51edaf7f23a094500f.

## Verdict

**FAIL** — Substantive finding(s) require a fix.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | The exporter admits nested past expressions beyond the reviewed target-origin shape: it checks individual intervals but never applies the TL-216 homogeneous/depth guard, so a mixed O/Y or deep O graph gets a C2PO manifest under evidence that covers only homogeneous bounded nesting. | src/infinite/export.rs:543 |

## Coverage

Examined: src/infinite/export.rs, src/infinite/mod.rs, src/mapping/mod.rs, tests/infinite_export.rs, tests/tc138_feature_boundary.rs, src/mapping/past.rs. Focused cargo test 14/14, cargo fmt and focused Clippy passed; no aggregate gate was run.

## New findings (disposition pass 1)

Reviewed at `4643c76d4d46730a5de9f50f47ebfc178b8f9e0d` against provisional base `4bcea84bbe3fa7360076ea51edaf7f23a094500f`.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | medium | The fix reimplements TL-216's homogeneous operator/interval and depth admission algorithm in `validate_target_origin_shape` while `mapping::past::validate_origin_shape` remains a separate authority; changing the reviewed partition in one exporter can silently leave the other admitting a different target shape. Share the policy transition or one typed authority across both syntax graph adapters. | src/infinite/export.rs:180; src/mapping/past.rs:337 |

## Dispositions

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 4643c76d4d46730a5de9f50f47ebfc178b8f9e0d — `validate_target_origin_shape` now rejects mixed O/Y and depth-four O before rendering; focused tests cover both. |
| FND-002 | still-open | Round 1 new finding: the same target-origin admission rule is implemented twice in the same crate; no shared authority yet. |
| FND-002 | fixed | 28255988abeebb660058e0b58c6426184f33b727 — both finite past mapping and infinite safety export now call one `OriginShapeGuard::push` policy in `src/mapping/past.rs`; focused past and infinite export suites preserve behavior. |

## Disposition verdict

**FAIL** — FND-002 remains open. Focused infinite-export tests 18/18, formatting, Clippy, and diff check passed; no aggregate gate was run.

## Disposition verdict — round 2

Reviewed at `28255988abeebb660058e0b58c6426184f33b727` against provisional base `4bcea84bbe3fa7360076ea51edaf7f23a094500f`. The previously fixed findings remain fixed; this round checked the sole still-open finding and both callers of the extracted guard.

**PASS** — zero open TL-218 review findings at this head. Focused `infinite_export` 18/18 and `past_mapping` 11/11, formatting, focused Clippy and diff check passed; no aggregate gate was run.

## Retargeted-head recheck

**PASS at `a0b3a42ec721217457d9be2a325d5df78d47080b` against landed main `6d8e1ad4c1a86a09d3ed52a4a6df6a98ed0f2596`.** Code review with the Rust lane rechecked the 16-file feature diff, four-commit range-diff, `OriginShapeGuard::push` shared by finite mapping and infinite export, fail-closed shape/refusal paths, and the one changed external-consumer syntax pin. The replayed feature patch is equivalent to the prior reviewed head; the only changed feature line pins the consumer to landed syntax `6aa9b11`. Focused infinite export 18/18, infinite trace 16/16, past mapping 11/11 with feature on/off, interop 4/4, contextual 5/5, and external consumer 1/1 with feature on/off passed. Formatting, all-target/all-feature Clippy and diff check passed. Prior FND-001/002 remain fixed; no new finding.
