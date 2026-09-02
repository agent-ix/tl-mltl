---
id: Task-002
title: "Producers, adapter, and the shared intake path"
type: Task
status: done
track: Intake
priority: P0
relationships:
  - target: ix://agent-ix/tl-mltl/PLAN-002
    type: part_of
  - target: ix://agent-ix/tl-mltl/FR-006
    type: references
---
# Task-002: Producers, adapter, and the shared intake path

## Scope

Write the three domain producers, the native adapter, and the driver that seals
through Quoin; demonstrate the Quire static export and Quoin intake, audit and
receipt without either tool executing a producer; and read the retained evidence
through the pinned Engineering Assurance mapping.

## Completion Evidence

`examples/reference_conformance.rs` replays the shared temporal corpus,
`examples/r2u2_differential.rs` replays the retained R2U2 exchange through the
C2PO mapping and the comparison layer, and `examples/cli_conformance.rs` drives
the built CLI over its declared request documents twice and compares bytes. Each
emits declared structured rows and exits non-zero on a failing row.

`scripts/assurance_chain.py` reads those bytes, derives every attested result
from a field the producer wrote, and seals, retains and verifies through the
pinned Quoin CLI. `tests/shared_assurance.rs` asserts the producer boundary with
two runs — every producer replaced by a logging stub with the log required to be
empty, and a control that stubs `quoin` and requires the chain to fail.

`scripts/legacy_evidence_view.py` reads all 283 files under `evidence/` through
`map_pgm01_bytes` from the pinned release, measures that no byte moved, asks Git
whether the retained bytes are the committed bytes, and reports the mapping's
answer as it stands.
