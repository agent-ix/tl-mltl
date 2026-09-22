---
id: ADR-002
title: "The native-predicate/native-temporal corpus dimension belongs to quire-mltl, not tl-mltl"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-008
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-009
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-010
    type: relates_to
  - target: ix://agent-ix/tl-mltl/decisions/ADR-001
    type: relates_to
---
# ADR-002: The native-predicate/native-temporal corpus dimension belongs to quire-mltl, not tl-mltl

## Status

**Accepted** — owner decision recorded 2026-09-22 against Linear epic
[TL-175](https://linear.app/agent-ix/issue/TL-175), applying the same architect
ruling [ADR-001](./ADR-001-retire-pgm01-citation-and-fr-019.md) records: every
TL-* crate (`tl-syntax`, `tl-parse`, `tl-mltl`, `tl-rewrite`) stays independent
of the agent-ix/Quire ecosystem, and only `quire-mltl` may bridge a TL-* crate
to Quire types.

## Context

The M4 corpus-coverage campaign (MRS-002, FR-008 through FR-010, NFR-004,
MP-002, TM-002, PLAN-007) was first authored in September 2026 on branch
`issue/38-corpus-interop-spec` (PR #44, never merged). That revision predates
the TL-175 ruling and declared a native-predicate/native-temporal
correspondence coverage dimension owned here, with:

- a plan task (`Task-023`, GitHub tl-mltl#55) whose frontmatter read
  `owner_repository: agent-ix/tl-mltl` and
  `producer_repository: agent-ix/quire-contract-ir`, blocked on
  `quire-contract-ir` #63 and #64;
- an FR-009 fixture-family row whose authoritative owner was
  `quire-contract-ir`;
- an FR-010 target-catalog row for a FRETish mapping emitted by
  `quire-contract-ir`, a native-source-authority paragraph, and an
  `FR-010-AC-5` requiring native-bridge cells to preserve #63/#64 identities;
- an FR-008 dependency clause blocking native cells on #63/#64.

Every one of those makes `tl-mltl` a *consumer* of `quire-contract-ir` output.
That is Quire→TL, the direction the ruling prohibits: dependencies flow TL→Quire
only, because Quire consumes TL's output and not the reverse.

The coupling is not incidental to the campaign. The rest of M4 — the closed
cell-obligation registry, the `tl-syntax`/`tl-parse`/`tl-rewrite`/`tl-mltl`
fixture families, the C2PO/R2U2 dispositions, the bounded manifest and
lifecycle rules, the reproducible census — is entirely TL-owned and stands
without it.

## Decision

The native-predicate and native-temporal correspondence coverage dimension is
removed from `tl-mltl`'s M4 campaign and specified in `quire-mltl` instead.

- `tl-mltl` keeps the TL-owned campaign: MRS-002's closed coverage model,
  FR-008's dimension catalog, FR-009's TL-owned and Rust-adapter fixture
  families, FR-010's C2PO/R2U2 dispositions, NFR-004, MP-002, TM-002, and
  PLAN-007's seven TL-owned tasks.
- FR-008's dimension catalog admits no native-predicate or native-temporal
  class; a cell naming one is an unknown-dimension refusal.
- FR-009's fixture-family ownership table is closed to TL-owned families and
  the Rust adapter repositories that emit a mapping target. A family whose
  authoritative owner is an agent-ix/Quire repository is out of scope here.
- FR-010's target catalog is closed to targets `tl-mltl` itself emits. A target
  emitted by an agent-ix/Quire repository — FRETish being the concrete case —
  is `unsupported` in this catalog rather than a row in it.
- `quire-mltl` specifies the dimension over the contracts it already owns.

## Why quire-mltl and not quire-contract-ir

`quire-contract-ir` owns the native Quire grammar and the predicate/temporal
correspondence vocabulary, and it is a *consumer* of `quire-mltl`: under
[ADR-001](./ADR-001-retire-pgm01-citation-and-fr-019.md) and `quire-mltl`'s own
ADR-001, TL-181 repoints `quire-contract-ir`'s imports from `tl_mltl::wire::*`
and `tl_mltl::mapping::contract_ir::*` to `quire_mltl::*`. So `quire-mltl`
cannot take a crate dependency on `quire-contract-ir` either — that would
close a cycle.

It does not need one. `quire-mltl` already owns the three documents the
dimension is about: the `tl-mltl.temporal-assessment-request/v1` that binds a
native clause's identity, predicate/catalog and capture/clock/history
alongside the TL formula and profile; the
`tl-mltl.temporal-assessment-result/v1` that carries the evaluated outcome and
every owner assertion; and the `tl-mltl.contract-ir-result-map/v1` that derives
the Contract-IR value-or-typed-non-value outcome. The correspondence identities
arrive as *data inside a request quire-mltl already strict-reads*, not as a
compiled dependency. The coverage dimension therefore states which
correspondence classes those three contracts must carry losslessly — which is
exactly what Task-023 was trying to say, from the wrong side of the boundary.

## Consequences

- GitHub tl-mltl#55 ("Consume native temporal corpus families after bridge
  acceptance", Linear TL-32) no longer describes work `tl-mltl` will do. It is
  closed as moved, and the equivalent obligation is tracked against
  `quire-mltl`.
- The M4 campaign loses one of eight plan tasks and gains no TL-side
  replacement. Nothing in the remaining seven depended on it: PR #44's own
  dependency graph placed `Task-023` on a conditional Track C, off the
  future/W-M critical path.
- `FR-010-AC-5` is deleted rather than reworded. `TC-097` keeps only its
  past/history obligation and moves from `Task-023` to `Task-014`, which owns
  the evaluator overlays those rows bind.
- The M4 campaign no longer has a cross-repository resume condition on
  `quire-contract-ir`. Its remaining external conditions are `tl-syntax` #38
  and its routed evaluator implementation — TL-internal.
- A future decision to give `tl-mltl` any native-source awareness reopens this
  ADR; it is not an editorial choice available to a campaign task.

## Alternatives Considered

- **Keep `Task-023` and reword it as "consume an artifact, not a crate".**
  Rejected: the direction of the dependency is what the ruling is about, and a
  campaign row that cannot become applicable until another repository's issues
  land is a cross-repository dependency however it is spelled. It also keeps
  `quire-contract-ir` in three `consumer_repositories` lists and in FR-009's
  ownership table.
- **Move the whole M4 campaign to `quire-mltl`.** Rejected: measured against
  PR #44's content, seven of eight tasks, three of three fixture-family owner
  groups, and both interoperability targets that `tl-mltl` actually emits are
  TL-owned. Relocating the campaign would strand the corpus model away from the
  crates whose bytes it governs.
- **Leave the rows `blocked` in `tl-mltl` indefinitely as documentation.**
  Rejected: a blocked row is still a declared obligation with an owner, a
  producer repository and a resume condition. "Blocked" is a lifecycle state,
  not an exemption from the ownership rule — and `quire-contract-ir` #63 and
  #64 have both since closed, so the rows would not stay blocked.
