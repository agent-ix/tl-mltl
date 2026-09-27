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

## Disposition verdict

**FAIL** — FND-002 remains open. Focused infinite-export tests 18/18, formatting, Clippy, and diff check passed; no aggregate gate was run.
