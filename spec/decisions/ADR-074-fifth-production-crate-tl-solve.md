---
id: ADR-074
title: "Record tl-solve as a fifth production TL crate and supersede the four-crate statement of ADR-003"
type: ADR
status: proposed
owner: kreneskyp
relationships:
  - target: ix://agent-ix/tl-mltl/ADR-003
    type: supersedes
  - target: ix://agent-ix/tl-solve/ADR-001
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-113
    type: relates_to
---

# ADR-074: Record tl-solve as a fifth production TL crate and supersede the four-crate statement of ADR-003

## Status

Proposed, 2026-11-23. Ready for human acceptance; not accepted. It supersedes
one sentence of ADR-003 and nothing else. The decision itself is recorded in
[tl-solve ADR-001](ix://agent-ix/tl-solve/ADR-001); this record exists so that
a reader of ADR-003 finds the change.

## Context

ADR-003 states, under Consequences, that "the campaign remains four production TL
crates", and rejects a separate `tl-infinite` crate because it would add "a fifth
production pin, release lane and duplicated public-type routing without an
established need". Project P-TL-147 (TL-393) adds `tl-solve`, a decision
procedure that encodes formulas to CNF, runs a SAT solver, and re-checks every
satisfiable answer with this crate's evaluators.

## Decision

1. The sentence "The campaign remains four production TL crates" in ADR-003 is
   superseded: the production set is tl-syntax, tl-parse, tl-mltl, tl-solve
   and tl-rewrite. "TL-213's release graph keeps those four crates" is read as
   five, and the release gate checks both tl-mltl feature sets as before.
2. The rest of ADR-003 stands: the non-default `infinite-trace` feature, the
   boundary checks, and the rule that tl-rewrite's production tl-mltl dependency
   does not enable the infinite provider.
3. The rejection of `tl-infinite` stands for its own reason (no established need
   for a second evaluator). tl-solve differs: the checker must stay separate from
   the thing it checks, and the solver stack must stay out of tl-mltl's
   resolved graph.
4. tl-mltl does not depend on tl-solve, directly or through a feature (FR-113).
   tl-solve depends on tl-mltl, with `infinite-trace` enabled only by tl-solve's
   own non-default `infinite-trace` feature.

## Consequences

tl-mltl gains no dependency, no feature and no public API from this decision.
Its maintainers gain one downstream consumer whose exact pin on a tl-mltl
revision must move with tl-mltl's releases, and a requirement (FR-113) that
guards the dependency direction.

## Alternatives Considered

- Leave ADR-003 silent: rejected, it would leave a standing contradiction.
- Amend ADR-003 in place: rejected, an accepted ADR keeps its text and is
  superseded by a new record.
