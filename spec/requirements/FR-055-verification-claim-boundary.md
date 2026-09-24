---
id: FR-055
title: Bind V1 verification milestones to actual evidence
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/FR-054
    type: depends_on
---

# FR-055: Bind V1 verification milestones to actual evidence

## Description

When the campaign reports a V1–V11 milestone, it shall mark the milestone
complete only when its declared executable gate ran successfully at the
reported revisions and all required populations reconciled.

## Behavior

The local gate runs deterministic unit, corpus, property and bounded
exhaustive partitions. Nightly fuzz, mutation, deeper enumeration, Miri,
coverage, performance and Kani lanes retain their own actual run status.
The Campaign config refuses a member's selected machine toolchain map that
names no supported language (`node`, `rust`, or `python`) or gives one an empty
identity before starting a measurement. Per-member overrides allow distinct
stable and nightly toolchain identities in one Campaign.
Live R2U2 runs in a deliberately invoked target lane. A not-run lane is
not green. A failed mutation target, incomplete exhaustive population,
unknown proof, stale corpus, unclassified unsupported feature graph or unreviewed
semantic mismatch in a required population leaves its milestone open. A
reviewed target-origin refusal may satisfy a declared classification check;
it never establishes target parity for the refused cells. The combined TL-215 review is a
human prerequisite for implementation and is never set by a test result.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-055-AC-1 | Each V1–V11 milestone has a named executable gate and precise pass, failure and incomplete states. | Test (TC-197) |
| FR-055-AC-2 | A missing/failed/stale lane cannot make an aggregate green, while one failed lane does not erase sibling measurements. | Test (TC-198) |
| FR-055-AC-3 | Automated evidence does not mark TL-215 accepted, publish a source release, assert native parity or claim certification. | Test (TC-199) |

## Dependencies

FR-054 binds the observed run; human acceptance remains external.
