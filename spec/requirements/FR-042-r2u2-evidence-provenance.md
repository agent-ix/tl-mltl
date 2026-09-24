---
id: FR-042
title: Retain R2U2 exchange and C2PO fuzz evidence honestly
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-039
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-041
    type: depends_on
---

# FR-042: Retain R2U2 exchange and C2PO fuzz evidence honestly

## Description

When past or infinite-fragment mapping is tested, the campaign shall retain
its canonical corpus, target version, per-step expected verdicts and exact
source/replay provenance, and shall exercise the `c2po_map` fuzz boundary.

## Behavior

The past corpus manifest pins each formula, trace, expected source verdict by
position, origin hazard class, target response when available, and digest.
The retained R2U2 v4.2 exchange is historical evidence only until a new,
separately recorded runtime execution occurs. A fuzz input enters through the
strict formula reader and mapping entry point; panic, hang, invalid output or
accepted unsupported case is a failure. A crash is minimized and promoted to
canonical corpus only with its own provenance, expected oracle and review.
Generated counts never silently enlarge the reviewed corpus denominator.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-042-AC-1 | Every past corpus case has pinned inputs, expected per-step source results, origin classification and a verified manifest digest. | Test (TC-164, TC-174) |
| FR-042-AC-2 | The `c2po_map` fuzz target invokes the real reader and mapper, records seed/tool/version/budget, and retains minimized failures without claiming a historical replay as a fresh R2U2 run. | Test (TC-174) |

## Dependencies

FR-039 owns origin hazards; FR-041 owns refusal classes.
