---
id: ADR-014
title: "Retain numeric R2U2 exchange bytes and refuse unpinned numeric export"
type: ADR
status: proposed
owner: tl-mltl-maintainer
relationships:
  - target: ix://agent-ix/tl-syntax/ADR-015
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-057
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-058
    type: realizes_requirement
---

# ADR-014: Retain numeric R2U2 exchange bytes and refuse unpinned numeric export

## Status

Proposed (TL-419). Ready for human acceptance.

## Context

`corpus/README.md` and CLAUDE.md state that `make ci` does not execute R2U2 or
C2PO; evidence is a retained exchange replayed from bytes, and past-time
refusals (1791 of 1833 grid cases) are counted and never treated as agreement.
Numeric export adds a risk (float matching) that tl-mltl cannot ground without
the target's numeric behaviour.

## Decision

1. Numeric differential evidence follows the same pattern: retained bytes under
   `corpus/r2u2-numeric-v1/`, replay by recomputation, live generation only by a
   local, non-ci target that requires `R2U2_ROOT`.
2. Numeric export is refused (`TargetNumericProfileUnpinned`) until a target
   numeric profile is pinned from retained evidence.
3. Fresh R2U2 and C2PO output is DEFERRED; this ADR and the requirements claim no
   agreement.

## Alternatives rejected

- Running R2U2 in ci: contradicts the standing policy.
- Assuming a binary32 target with a default tolerance: ungrounded.

## Consequences

Numeric export has no effect until evidence is retained, which is the intended
conservative behaviour.
