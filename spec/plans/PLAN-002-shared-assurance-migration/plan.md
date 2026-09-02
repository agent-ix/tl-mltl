---
id: PLAN-002
title: "Shared assurance migration"
type: Plan
status: in_progress
relationships:
  - target: ix://agent-ix/tl-mltl/FR-006
    type: references
---
# PLAN-002: Shared assurance migration

## Objective

Move tl-mltl from its repository-local evidence framework onto the released
Engineering Assurance, Quire, Quoin and ix-flow contracts, preserving every
MLTL-domain behaviour, the retained R2U2 interoperability exchange, and every
retained evidence byte.

## Approach

The reference semantics are not the subject of this change. Bounded closed-trace
evaluation, checked horizon analysis, pending-aware open prefixes, the C2PO
mapping, the comparison layer, the shared temporal corpus and the CLI are
carried across unchanged, and the migration only changes how their results are
declared, transcribed, retained and verified.

Four properties shape the design.

**The driver never produces.** One target, `make assurance-inputs`, runs the
producers. Everything downstream consumes those files, and an absent input is an
error naming that target rather than a step the driver quietly performs itself.

**Every attested result is read from producer bytes.** No verdict is inferred
from an exit code alone or recovered from a transcript. This is the failure that
cost Wave 1 a high finding — a chain that sealed `passed` for every proof without
reading what the producer wrote — and it is designed against here rather than
discovered later.

**The differential is never a boolean.** This repository's external-compatibility
claim is a comparison against a retained R2U2 4.2 exchange. Agreement, mismatch
and non-conclusive stay three answers; conclusive, pending, unsupported and
tool-error stay four external states; the reference verdict and time stay
separate from the external ones. Reducing any of that to one bit would keep every
gate green while destroying the claim.

**Nothing executes the external monitor.** R2U2 and C2PO ran once, out of band.
Their exact exchanged artifacts are retained and pinned by digest, and every
later claim is a replay against those bytes. A gate that re-ran the monitor
would be producing the thing it checks.

## Scope

In scope: the pin declaration, the change-assurance declaration, the driver, the
compatibility view, the three domain producers, the Makefile, the hosted
workflow, the specification, and the deletion of the local evidence framework.

Out of scope: evaluation, horizon, prefix, mapping or comparison behaviour; the
corpus contents; the retained R2U2 artifacts; and any release or publication
decision.

## Landing constraints

- Hosted CI is manual-only and is not dispatched by this change.
- Retained evidence bytes are immutable.
- The old generic path is deleted only after both paths have been run against
  the same candidate revision and the result recorded as observed.
- What removing the Make execution-control guard costs is measured in this
  repository and recorded, not argued away.
