---
id: Task-003
title: "Run the retained fuzz baseline"
type: Task
status: blocked
track: B
priority: P1
relationships:
  - target: ix://agent-ix/tl-mltl/Task-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-012
    type: references
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: references
  - target: ix://agent-ix/tl-mltl/TC-055
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-056
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-057
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-058
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-067
    type: verifies
  - target: ix://agent-ix/tl-mltl/TC-068
    type: verifies
---

# Task-003: Run the retained fuzz baseline

## Scope

Implement reviewed fuzz targets and the three-seed, dual-cap baseline with exact
feature identities, replayable crashes, and bounded plateau observations.

## Subtasks

- [ ] Select only catalog-backed Fuzz-kind boundaries and freeze identities.
- [ ] Record snapshots, corpora, stops, raw states, replay, and minimization.
- [ ] Prove count-only, early-stop, reset, and instrumentation-loss cases refuse.

## Deliverables

- `tl-mltl.fuzz-campaign/v1` records and retained artifact manifests.
- Executing requirement-tagged tests for the assigned matrix rows.

## Notes

- Binary retention and actual build profile must be representable by accepted
  Quoin #363/#364 contracts.
