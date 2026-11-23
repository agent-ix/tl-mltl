---
id: ADR-047
title: "Document verdict names against LTL3 and rename nothing on the wire"
type: ADR
status: proposed
owner: tl-mltl-maintainer
relationships:
  - target: ix://agent-ix/tl-mltl/FR-085
    type: realizes_requirement
---

# ADR-047: Document verdict names against LTL3 and rename nothing on the wire

## Status

Proposed (TL-427). Ready for human acceptance; not accepted.

## Context

LTL3 uses three verdicts: true, false, inconclusive. tl-mltl already exposes
`TruthValue::{True,False,Pending}` (prefix evaluation), the five FR-341
labels `Proved/Refuted/Inconclusive/Unsupported/Failed`, and
`RecordVerdict::{True,False,Pending}`. Renaming any of them would break wire
and corpus bytes.

## Decision

Publish a normative mapping and a total accessor; change no serialized name.

| tl-mltl name | LTL3 verdict |
|---|---|
| `TruthValue::True`, `RecordVerdict::True`, `Disposition::Proved` | true |
| `TruthValue::False`, `RecordVerdict::False`, `Disposition::Refuted` | false |
| `TruthValue::Pending`, `RecordVerdict::Pending`, `Disposition::Inconclusive` | inconclusive |
| `Disposition::Unsupported`, `Disposition::Failed` | none (not a verdict) |

LTLf evaluation of a complete finite trace returns only true or false. A prefix
of a trace is not an LTLf input in v1, so LTLf has no inconclusive result from
the evaluator. tl-solve's `Inconclusive` (bounded miss) is the same
word with the same meaning: no conclusion either way, never unsatisfiable.

## Alternatives rejected

- Rename `Pending` to `Inconclusive`: wire break; and Pending also says "more
  input may decide it", which inconclusive does not.

## Consequences

One documentation table, one accessor per type, one totality test.
