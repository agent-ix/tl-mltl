---
id: FR-090
title: "Report with horizon analysis where mission-bounding may have changed a requirement's meaning"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-044
    type: implements
  - target: ix://agent-ix/tl-mltl/ADR-045
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
---

# FR-090: Report with horizon analysis where mission-bounding may have changed a requirement's meaning

## Description

When a FRET requirement is imported together with a mission time, tl-mltl shall
compute its horizon (FR-002) and shall classify it so that a user can see where
mission-bounding probably altered the requirement.

## Behavior

Mission time `M` comes from the trace header or the caller as a checked `u32`;
absent, the report says `mission-time-absent` and makes no divergence claim.
Per requirement the report records root identity, horizon `H`, `M`, and one
class: `bound-equals-mission` (some temporal node's upper bound equals `M`,
the signature of an unbounded operator closed at `M`), `horizon-exceeds-mission`
(`H > M`, so the verdict is undecided at mission end), or `within-mission`.
Classes are not exclusive; a requirement may carry both of the first two. The
report is a statement about the formula's shape, not a proof of the original:
the original unbounded formula is not available, so a flagged requirement is
`possibly changed upstream`, and the wording never says `changed`. Each flag
cites the known-case row of FR-094. The report has a SHA-256 digest used by
ADR-045 acknowledgements.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-090-AC-1 | For a hand-authored population of 12 formulas (4 designed for each of `bound-equals-mission`, `horizon-exceeds-mission`, `within-mission`, one of them carrying two classes) at `M = 5`, the classes and horizons match the manifest. | Test (TC-245) |
| FR-090-AC-2 | With no mission time the report says `mission-time-absent`, makes no divergence claim, and an overflowing `H` is refused rather than wrapped. | Test (TC-246) |

## Dependencies

FR-002, ADR-045, FR-094.
