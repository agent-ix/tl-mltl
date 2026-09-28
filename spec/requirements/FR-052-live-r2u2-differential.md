---
id: FR-052
title: Compare a fresh R2U2 run with TL semantics
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-038
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-040
    type: depends_on
---

# FR-052: Compare a fresh R2U2 run with TL semantics

## Description

When the live R2U2 lane runs, it shall execute a pinned C2PO/R2U2 toolchain
against reviewed bounded, past and admissible infinite safety cases and
compare per-step output with the independent TL oracle.

## Behavior

The lane records source/tag/commit, binary digests, license, command,
environment, C2PO input, compiled monitor, trace, raw output, exit state and
parser revision. Existing v4.2 retained bytes are historical and cannot
satisfy this fresh-run requirement. Every case is classified agreement,
semantic mismatch, unsupported mapping, unavailable target or non-conclusive
output. Origin hazards and refutation-only infinite cases remain distinct;
a target pass never proves liveness. Neither CI nor the oracle silently
executes an unpinned foreign runtime.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-052-AC-1 | A fresh exact-pin target run retains raw artifacts and per-step comparison classes on reviewed bounded and past cases. | Test (TC-191) |
| FR-052-AC-2 | Infinite safety violations replay as bad prefixes; target pass, origin mismatch and unavailable runs cannot become proved parity. | Test (TC-192) |

## Dependencies

FR-038/040 own mapped target forms.
