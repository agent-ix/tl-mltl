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

## Status

OPEN/BLOCKED. No retained numeric R2U2 exchange exists, nothing in this repository can
produce one, and no numeric target profile is pinned, so TL-423 AC1 (R2U2 differential
on the numeric corpus with per-step verdicts) is not met by any work specified here.
The only buildable deliverable of TL-423 is the refusal-until-pinned path (FR-056,
FR-057-AC-2); the replay machinery of this requirement is built and tested only against
synthetic fixtures that are labelled synthetic, or not at all, and never claimed as
evidence. What unblocks it: a human runs the pinned R2U2 checkout locally, pins the
profile from that run and retains the exchange (tl-syntax matrix, "Open and blocked
acceptance criteria").

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
  for every step, tl-mltl's verdict, R2U2's verdict or `none`, and one of `agree`,
  `disagree`, `refused-at-export` (with the loss cause) or `no-r2u2-verdict`. A monitor
  that must wait for a window does not necessarily emit a verdict at the last positions
  of `F[0,1] p` or `G[0,1] p` on a 4-step trace, and which steps these are is observed,
  not predicted (the retained Boolean corpus does not show it for numeric
  specifications, UNGROUNDED). `no-r2u2-verdict` is recorded for every step at which
  R2U2 emitted none, whatever tl-mltl's verdict is (including a padded closed-trace
  verdict), and the comparison is made only where both sides have a verdict. Totals
  count the four dispositions separately; `refused-at-export` and `no-r2u2-verdict` are
  never counted as agreement.
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
  below are `planned` or blocked; no claim of agreement is made. The digest retention
  in this requirement is item 10 of the tl-syntax/ADR-019 register (retained,
  justified, subject to the human exemption decision).

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-058-AC-1 | Replay recomputes the comparison from retained bytes, verifies every digest, and fails on a one-byte alteration of any retained file. | integration-testing (TC-1589) |
| FR-058-AC-2 | The report records per-step verdicts for every case row with the four dispositions counted separately, and neither refusal nor an absent R2U2 verdict is ever an agreement; the sum of the four counts equals the 6,912 per-step records. | integration-testing (TC-1590) |
| FR-058-AC-3 | No test, example or ci recipe spawns R2U2 or C2PO; the campaign inventory lists the live target as planned. | inspection (TC-1591) |
| FR-058-AC-4 | The retained exchange for the declared population exists with the pinned profile recorded. OPEN/BLOCKED: generation needs a human R2U2 run; the AC cannot pass until the bytes exist. | demonstration (TC-1592) |

## Dependencies

NFR-004 owns retention; ADR-014 records the policy.
