---
id: FR-058
title: Retain and replay numeric R2U2 differential evidence without executing R2U2
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-056
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-004
    type: depends_on
  - target: ix://agent-ix/tl-mltl/ADR-014
    type: depends_on
---

# FR-058: Retain and replay numeric R2U2 differential evidence without executing R2U2

## Description

When numeric R2U2 evidence is needed, tl-mltl shall consume retained exchange
bytes under `corpus/r2u2-numeric-v1/` and compare them per step with its own
verdicts, and nothing in `make ci` or any test shall execute R2U2 or C2PO.

## Behavior

- **Retained bytes** (same pattern as `corpus/r2u2-v4.2/` and
  `corpus/r2u2-live-v1/`): `manifest.json` (identity `tl-mltl.r2u2-numeric-corpus/v1`,
  tool identities and pinned revision from `campaign/tool-versions.json`, the
  pinned `TargetNumericProfile` and its evidence), `spec.c2po`, `signals.map`,
  `trace.csv`, raw `r2u2.stdout`, `report.json` (tool versions, input and output
  digests), and the source-bound `replay-report.json`.
- **Replay** (`make numeric-differential`, a `ci` prerequisite): recompute the
  comparison from the retained bytes only, verifying digests; R2U2's per-range
  output is expanded over covered positions as in the future grid.
- **Per-step verdicts recorded**: for each case (formula, trace) the record keeps,
  for every step, tl-mltl's verdict, R2U2's verdict, and one of `agree`,
  `disagree`, `refused-at-export` (with the loss cause). Totals count the three
  separately; `refused-at-export` is never counted as agreement.
- **Declared population** (to be generated in the deferred step): 36 Integer
  predicates `x op c` (6 operators by 6 constants) and 36 Decimal predicates
  `d op k`, each under three wrappers (`p`, `F[0,1] p`, `G[0,1] p`) = 216
  formulas; 8 fixed traces of 4 steps; 216 x 8 = 1,728 case rows and 6,912
  per-step records. The admitted/refused split is recorded after generation and
  is not predicted here.
- **DEFERRED**: producing fresh R2U2 and C2PO output (running the pinned
  checkout) and pinning the target numeric profile. A live target
  `make r2u2-numeric-live` mirrors `r2u2-past-differential` (local-only, requires
  `R2U2_ROOT`, exits 2 if absent), is not a `ci` prerequisite, and is registered
  as a `planned` member in `campaign/members.json` with ticket TL-423 until
  executed (`make campaign-inventory` would otherwise fail).
- Until generated, `make numeric-differential` is not added to `ci` and the rows
  below are `planned`; no claim of agreement is made.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-058-AC-1 | Replay recomputes the comparison from retained bytes, verifies every digest, and fails on a one-byte alteration of any retained file. | integration-testing (TC-1589) |
| FR-058-AC-2 | The report records per-step verdicts for every case row with the three dispositions counted separately, and refusal is never an agreement. | integration-testing (TC-1590) |
| FR-058-AC-3 | No test, example or ci recipe spawns R2U2 or C2PO; the campaign inventory lists the live target as planned. | inspection (TC-1591) |
| FR-058-AC-4 | The retained exchange for the declared population exists with the pinned profile recorded (DEFERRED generation). | demonstration (TC-1592) |

## Dependencies

NFR-004 owns retention; ADR-014 records the policy.
