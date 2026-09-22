---
id: ADR-001
title: "Retire tl-mltl's PGM-01 citation and FR-019; the QObs C00 boundary moved to quire-mltl"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-019
    type: relates_to
  - target: ix://agent-ix/tl-mltl/NFR-002
    type: relates_to
  - target: ix://agent-ix/quire-mltl/FR-002
    type: relates_to
---
# ADR-001: Retire tl-mltl's PGM-01 citation and FR-019; the QObs C00 boundary moved to quire-mltl

## Status

**Accepted** — architect ruling recorded against Linear epic
[TL-175](https://linear.app/agent-ix/issue/TL-175). This document is the
mirror image of `quire-mltl`'s own
[ADR-001](https://github.com/agent-ix/quire-mltl/blob/main/spec/decisions/ADR-001-tl-crates-stay-quire-independent-quire-mltl-bridges.md):
that document records why `quire-mltl` is the one deliberate exception that
*keeps* the `quire-observation` dependency and the PGM-01 citation; this
document records why `tl-mltl` *loses* both. Supersedes nothing — no prior ADR
existed in this repository's `spec/decisions/` before TL-180, because this
directory did not exist before this ticket.

## Context

`tl-mltl` merged PR [#67](https://github.com/agent-ix/tl-mltl/pull/67)
("Implement complete temporal evaluator owner boundary"), which added FR-018
(the strict request/result/mapping owner boundary) and FR-019 (pinning that
boundary to an accepted `quire-observation` C00 revision and publishing
explicit QObs compatibility dispatch). That work was reviewed by SR-044
through SR-051 and accepted. It also gave `spec/spec.md` (MRS-001) a
`depends_on` edge to `ix://agent-ix/quire-contract-ir/PGM-01`, Purpose-section
prose citing PGM-01 as governing "compatibility, provenance, evidence, human
authority, and qualification boundaries," and a References-section link to
the PGM-01 governance document. `NFR-002` picked up a parallel `references`
edge to PGM-01 and prose citing "canonical PGM-01 evidence boundaries."

That arrangement directly contradicted the architect's intent for the TL-*
crate family. Peter, as architect, ruled on epic TL-175:

> "i was expecting the TL-\* crates to be ENTIRELY independent of Quire
> ecosystem... the crates themselves were supposed to be independent"
>
> "we will use quire-observation and other things FOR THE OUTPUT FROM tl-\*
> crates. BUT TL DOES NOT USE ANYTHING FROM AGENT-IX TO DO ITS THING"
>
> "now it is possible we need a quire-mltl integration crate, but that is it.
> deps are still TL flowing IN to Quire ecosystem"
>
> "I AM THE ARCHITECT. MY INTENT IS WHAT YOU ARE BUILDING. MY WORD OVERRULES
> ANY MISTAKEN DESIGN IN A SPEC"

MRS-001's own `depends_on: ix://agent-ix/quire-contract-ir/PGM-01` edge was
the spec-root evidence of the contradiction: a crate whose Purpose section
says it "owns closed-trace evaluation, decision horizons, pending-aware prefix
verdicts, external-monitor mapping, and differential evidence" — a
self-contained temporal evaluator — should not itself declare a `depends_on`
edge to a Quire governance program. That a master requirements spec can carry
a dependency edge its own architect never intended is exactly the kind of
mistaken design the ruling above overrules.

## Decision

`tl-mltl` retires its PGM-01 citation and the FR-019 requirement that
motivated it, while keeping the historical record of both intact rather than
erasing them:

- **MRS-001** (`spec/spec.md`) drops the `depends_on:
  ix://agent-ix/quire-contract-ir/PGM-01` frontmatter edge, the Purpose-section
  sentence citing PGM-01's governance scope, and the References-section link
  to the PGM-01 governance document (TL-180).
- **FR-019** is marked `status: superseded` rather than deleted. Its
  Description, Inputs, Outputs, Behavior, Constraints, and Acceptance Criteria
  are retained unmodified as the historical record of what it specified while
  active. A new `## Supersession` section names
  [quire-mltl FR-002](https://github.com/agent-ix/quire-mltl/blob/main/spec/requirements/FR-002-dispatch-qobs-c00-compatibility.md)
  as the successor that carries this boundary forward, ported unchanged in
  behavior by TL-178.
- **NFR-002** drops its `references: quire-contract-ir/PGM-01` edge and its
  PGM-01-citing prose, and is reworded to lean on
  [NFR-003](../requirements/NFR-003-qualification-integrity.md) — which
  already owns the shared-assurance intake path this repository actually
  runs — instead of a governance program `tl-mltl` no longer cites.
- **SR-044 through SR-051**, the frozen SpecReviews that covered PR #67 /
  FR-018 / FR-019, are not edited. This repository's convention, confirmed by
  precedent in `quire-contract-ir`'s own PGM-01-R08/R09 withdrawal notes (which
  amend policy in a dated, signed paragraph rather than rewriting the withdrawn
  rule's original text), is that a frozen SpecReview is a historical record of
  what was reviewed and accepted at the time, not a live document that tracks
  the current decision. [SR-052](../reviews/SR-052-retire-pgm01-citation-and-fr-019.md)
  is the new review that documents this reversal.

This ticket (TL-180) is spec-only. It does not touch `Cargo.toml`,
`src/wire/{request,observation,report}.rs`, `src/mapping/contract_ir.rs`, or
README.md's AGPL paragraph — those files still implement FR-019's boundary
today, and TC-085 still exercises it. Dropping the `quire-observation`
dependency and deleting those four files is separate, tracked follow-up work
(TL-179's PR B), picked up by a different agent. FR-019 is marked superseded
now so the spec stops asserting a governance dependency the architect ruled
out, without waiting on that code removal to land first.

## Consequences

- `tl-mltl`'s spec no longer asserts a PGM-01 dependency or governance
  citation anywhere, closing the spec-root contradiction the ruling
  identified.
- `tl-mltl` remains, for the moment, still coupled to `quire-observation` in
  code (`Cargo.toml`, the four wire/mapping files) until TL-179's PR B lands.
  The spec and the code are briefly out of sync on this one axis — the spec
  states the target/retired state, the code hasn't caught up yet — which is
  expected of spec-first work and is exactly what TL-179's PR B closes.
- A future reader of FR-019 sees both what it specified and why it stopped
  being current, without losing either.
- `quire-contract-ir`'s own PGM-01-governance document and schema catalog
  still reference `tl-mltl` types directly; repointing those to `quire-mltl`
  is TL-181's scope, not this ticket's.

## Alternatives Considered

- **Delete FR-019 outright.** Rejected: erases the historical record of what
  `tl-mltl` actually implemented and shipped in PR #67, and of what SR-044
  through SR-051 reviewed. Marking it `superseded` keeps that record readable
  while making clear it no longer governs current or target behavior.
- **Edit SR-044 through SR-051 to reflect the reversal.** Rejected: this
  org's convention, per the `quire-contract-ir` PGM-01-R08/R09 precedent, is
  that frozen SpecReviews are never edited after the fact, even when the
  underlying decision reverses. A new SpecReview (SR-052) documents the
  reversal instead.
- **Wait for TL-179's PR B to land before touching the spec.** Rejected: the
  spec correction stands on its own — MRS-001's PGM-01 `depends_on` edge is
  wrong regardless of when the code catches up, and leaving it in place would
  keep asserting a dependency the architect has already ruled out.
