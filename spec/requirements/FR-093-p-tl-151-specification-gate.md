---
id: FR-093
title: "Record the P-TL-151 specification gate and map every ticket criterion to a requirement"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
---

# FR-093: Record the P-TL-151 specification gate and map every ticket criterion to a requirement

## Description

While the P-TL-151 build tickets are blocked on TL-427,
the specification set shall list each ADR with its acceptance state, map every
acceptance criterion of TL-427 to TL-435 to a requirement and a test row, and
record the review result before any build ticket starts.

## Behavior

The ledger lives in `spec/ltlf-interop-test-matrix.md` (TM-047), which also states
the identifier-scope rule: ids are local to a repository and ids cited from another
repository are qualified with its name; same-numbered ids in different repositories
are homonyms, not collisions (for example this repository's FR-092 and
`filament-specification` FR-092). The open decisions listed in PLAN-050 are
recorded there for Peter and are not resolved by this specification. ADR status is
`proposed` until a human accepts it; no automated step marks an ADR accepted.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-093-AC-1 | The ledger covers every ticket criterion of TL-427 to TL-435 with a requirement and a TC. | Inspection |
| FR-093-AC-2 | Every ADR of this project is recorded as accepted by a named human before the first build ticket starts, and a spec-review result is recorded. | Inspection |

## Dependencies

All other requirements of this project.
