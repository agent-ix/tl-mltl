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
