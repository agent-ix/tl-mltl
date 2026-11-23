---
id: ADR-045
title: "Refuse implicit mission-bound closure and allow it only with a bound acknowledgement"
type: ADR
status: proposed
owner: tl-mltl-maintainer
relationships:
  - target: ix://agent-ix/tl-mltl/FR-090
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-094
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-044
    type: depends_on
---

# ADR-045: Refuse implicit mission-bound closure and allow it only with a bound acknowledgement

## Status

Proposed (TL-427). Ready for human acceptance; not accepted.

## Context

Converting an unbounded formula to a bounded one with a mission time `M`
(`G p` to `G[0,M] p`, `p U q` to `p U[0,M] q`) changes meaning in known
cases. Over a finite trace closed-trace padding makes `G[0,M] p` false on a
short trace where LTLf `G p` is true; `F[0,M] p` can miss a witness an
unbounded `F p` finds. FRET exports bounded MLTL after applying mission time, so
meaning may already have changed before this repository sees the formula.

## Decision

Options considered: (A) refuse; (B) warn and proceed; (C) allow with
acknowledgement. **Recommendation: A by default with C as the only override.**

1. No importer, exporter or evaluator converts an unbounded formula to a
   bounded one implicitly. The attempt is the typed refusal
   `MissionClosureRequired`, and no artifact is emitted.
2. A closure request carries a mission time and an `Acknowledgement` holding the
   SHA-256 of the closure report (FR-090) for that exact formula and mission
   time. A missing or mismatched acknowledgement is refused. A matching one
   emits the bounded formula with the report attached.
3. There is no warning-only outcome: a warning is not machine-actionable and in
   a regulated pipeline is discarded; B is rejected.
4. Importing an already-bounded formula is not a closure by this library and is
   not refused. It is reported: where the horizon analysis shows the signature
   of a mission bound (FR-090), the report says the requirement was possibly
   changed upstream. The report never claims it knows the original meaning.

## Alternatives rejected

- B (warn): see item 3.
- Allow silently with a documentation caveat: the failure this ADR exists to stop.
- Prove preservation per formula and allow when proven: valuable but beyond
  P-TL-151; the known-case classification of FR-094 is the interim aid.

## Consequences

The Rust API gains one refusal, one acknowledgement type and a report digest.
The decision whether to build the closure conversion itself in P-TL-151, or only
the refusal and the report, is left to Peter (recommended: refusal and report
only).
