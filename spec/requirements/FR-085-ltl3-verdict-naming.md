---
id: FR-085
title: "Map every result name to its LTL3 verdict through a documented total accessor"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-047
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-007
    type: depends_on
---

# FR-085: Map every result name to its LTL3 verdict through a documented total accessor

## Description

Where a tl-mltl result type carries a truth or disposition name, tl-mltl shall
document its LTL3 mapping (true, false, inconclusive, or none) and shall expose
an accessor that is total over every variant, without changing any serialized
name.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-085-AC-1 | Every variant of `TruthValue`, `RecordVerdict` and `Disposition` (3 + 3 + 5 = 11 variants) maps as in ADR-047, `Unsupported` and `Failed` map to none, and an added variant fails compilation of the exhaustive test. | Test (TC-309) |
| FR-085-AC-2 | Every serialized name and every retained corpus byte is unchanged. | Test (TC-309) |

## Dependencies

ADR-047.
