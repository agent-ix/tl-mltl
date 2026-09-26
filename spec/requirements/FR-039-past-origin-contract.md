---
id: FR-039
title: Guard past-time export at the trace origin
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-038
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-016
    type: references
---

# FR-039: Guard past-time export at the trace origin

## Description

When a past-time formula can inspect before trace position zero, tl-mltl shall
bind the source's false-before-origin rule to an equivalent target contract or
refuse the export before producing executable output.

## Behavior

The source profile interprets out-of-origin past atoms as false. The exporter
cannot infer that R2U2/C2PO shares this rule from syntax acceptance alone.
It records the exact target version and reviewed origin behavior for each
admitted operator and interval. If no equivalent target behavior or explicit
safe guard is established, the mapping is unsupported. The pinned origin
corpus includes position zero, one, and the first position where each lower
bound becomes reachable. Any target difference is retained as a classified
mismatch or declared unsupported case; it is never described as parity.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-039-AC-1 | Every admitted past operator has a reviewed target-origin equivalence or guard, checked at and around origin. | Test (TC-165, TC-166) |
| FR-039-AC-2 | A target origin mismatch or missing target evidence refuses with a typed cause and no C2PO artifact. | Test (TC-167) |

## Dependencies

FR-038 owns mapping; FR-016 owns source past semantics.
