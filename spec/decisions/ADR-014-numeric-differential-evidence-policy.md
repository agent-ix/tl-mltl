---
id: ADR-014
title: "Retain numeric R2U2 exchange bytes and refuse unpinned numeric export"
type: ADR
status: proposed
owner: tl-mltl-maintainer
relationships:
  - target: ix://agent-ix/tl-syntax/ADR-015
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-019
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-057
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-058
    type: realizes_requirement
---

# ADR-014: Retain numeric R2U2 exchange bytes and refuse unpinned numeric export

## Status

Proposed (TL-419). Ready for human acceptance. This is tl-mltl ADR-014; the ADR-014 of
tl-syntax is a different record (numeric expression layer) and every cross-repository
citation in this repository names its repository (`tl-syntax/ADR-015`).

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
4. The R2U2 differential acceptance criteria TL-423 AC1 and TL-425 AC2 are OPEN and
   BLOCKED on an R2U2 run that no specified work performs. The only deliverable of this
   project toward them is the refusal-until-pinned path of decision 2: every numeric
   predicate and every contract containing one is refused with a typed loss record, and
   nothing is exported, compared or counted as agreement. What unblocks them: a human
   runs the pinned R2U2 checkout locally, pins the numeric profile from that run and
   retains the exchange (see tl-syntax `spec/numeric-predicates-test-matrix.md`,
   "Open and blocked acceptance criteria").
5. Grounding. The retained corpus (`corpus/r2u2-v4.2`) shows only Boolean `INPUT`
   declarations and `FTSPEC`/`PTSPEC` sections. Everything numeric about C2PO and
   R2U2 in this repository's specification (typed numeric `INPUT`, arithmetic, `abs`,
   numeric comparison and threshold literals, float tolerance, a contract construct,
   per-step contract output) is UNGROUNDED or DEFERRED and is labelled so where it
   appears.

## Alternatives rejected

- Running R2U2 in ci: contradicts the standing policy.
- Assuming a binary32 target with a default tolerance: ungrounded.

## Consequences

Numeric export has no effect until evidence is retained, which is the intended
conservative behaviour.
