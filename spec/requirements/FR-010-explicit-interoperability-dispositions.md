---
id: FR-010
title: "Report agree, disagree, or not-comparable for monitor interoperability"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-005
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: depends_on
---

# FR-010: Report agree, disagree, or not-comparable for monitor interoperability

## Description

When a corpus case is mapped to an external monitor or compared with a retained
external observation, tl-mltl shall report agreement, disagreement, or
not-comparable with a reason, and shall never turn target acceptance, a refusal,
or a missing result into agreement or qualification.

## Inputs

- The source formula, profile, and context, and the case's fixture identity.
- The Rust mapping and its emitted bytes, or the typed mapping refusal.
- Optionally a retained external observation with the tool identity and version,
  the formula and trace identities, and its status and value.

## Outputs

- One comparison outcome: `agreement`, `mismatch` (disagree), or `non_conclusive`
  (not comparable), with a deterministic reason.
- For a mapping that cannot be emitted, a typed refusal with the reason.

## Behavior

A comparison is `agreement` only when the observation is conclusive and its truth
value and verdict time equal the reference's. It is `mismatch` when both are
conclusive and differ. It is `non_conclusive` when the observation is pending,
unsupported, a tool error, malformed, or has no comparable verdict, and the
record keeps the cause. The contextual form also reports identity mismatch when
the formula, trace, signal catalog, or caller context in the observation differs
from the reference's, and emits no comparison in that case.

A formula the mapping cannot represent is refused at export with a typed reason
and emits no target bytes. A refusal is not agreement and says nothing about the
target: the live past-time evidence records how many cases were admitted to the
tool and how many were refused, and claims agreement for the admitted cases
only.

The retained 4.2/C2PO 4.1 exchange is an exact eight-case observation, and the
live R2U2 evidence is bounded to its recorded cases and tool identities.
Agreement covers those exercised cases, not the target language, the monitor,
the source profile, or unexercised cells.

Build, test, CI, and qualification paths do not execute Java, Node, Electron,
C2PO, R2U2, or another foreign runtime; they replay retained output. A manually
produced observation is contextual evidence and does not discharge mapping
correctness. Mapping correctness is checked against the canonical TL semantics,
the exact Rust emitter output, and an independent expected result. Parser
acceptance is at most an observation and never a semantic oracle.

A target emitted by an agent-ix/Quire repository is not a row here: under
ADR-002 its dispositions are `quire-mltl`'s.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-010-AC-1 | Every comparison reports agreement, disagreement, or not-comparable with a reason, and a pending, unsupported, tool-error, malformed, or verdict-absent observation is never reported as agreement. | Test (TC-100, TC-101) |
| FR-010-AC-2 | Claims stay bounded to the retained cases and tool identities, a mapping refusal is never counted as agreement, and no build, test, CI, or qualification path executes Java, Node, Electron, C2PO, or R2U2. | Test (TC-100) |
| FR-010-AC-3 | An observation whose tool identity, formula, trace, signal catalog, or context does not match the reference is refused before comparison, and a conclusive observation that disagrees is reported as disagreement. | Test (TC-100) |
| FR-010-AC-4 | Rust mapping correctness is tested against the canonical TL semantics, exact expected output, and typed refusal reasons without using parser acceptance or external execution as a semantic oracle. | Test (TC-093, TC-100) |

## Dependencies

Depends on FR-004 and FR-005 for mapping and comparison semantics, FR-007 for
contextual identity, and FR-009 for fixture identity.
