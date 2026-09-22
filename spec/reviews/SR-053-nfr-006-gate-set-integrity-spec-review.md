---
id: SR-053
title: Spec review of NFR-006 gate-set integrity
type: SpecReview
analysis: base
scope: "agent-ix/tl-mltl#14; TL-65; NFR-006; FR-006; NFR-003"
review_set: subset
relationships:
  - target: ix://agent-ix/tl-mltl/NFR-006
    type: reviews
---

# SR-053: Spec review of NFR-006 gate-set integrity

## Summary

Base checklist plus EARS-conformance and dependency review of the newly
authored `spec/requirements/NFR-006-gate-set-integrity.md` (Linear TL-65,
`agent-ix/tl-mltl#14`), which follows the precedent tl-rewrite established for
the identical problem (`NFR-004-gate-set-integrity`, TL-64, merged PR #48)
rather than the shared upstream control TL-65 originally cited — that
citation (`agent-ix/engineering-assurance#11`) was corrected in the ticket
thread as unrelated (producer-qualification campaigns, not gate-set binding),
and no matching shared control exists upstream.

One inconsistency was found and fixed during this review (FND-1). No blocking
findings remain. `quire validate` and `quire lint` are clean against the
file, aside from pre-existing module-registration duplicate-archetype
warnings unrelated to this artifact (confirmed present against an unrelated
baseline file — see Notes).

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-1 | low | The Measurement and Evaluation table's "Execution-control surfaces recognized" metric counted 12/12 while the prose and AC-1 list 13 named surfaces (the 13th being the bare-`;` command-separator check this requirement deliberately bakes in from the start, unlike tl-rewrite's own NFR-004.md, whose measurement table was never updated after that check was added in a later review pass there). Fixed: target/threshold corrected to 13/13. | NFR-006 Measurement and Evaluation table |

## Dispositions

| Finding | Disposition | Evidence |
|---|---|---|
| FND-1 | **FIXED** | Table row now reads `13/13 \| 13/13`. |

## Base checklist

- **ID format**: `NFR-006` matches `^[A-Z]{2,4}-[0-9]+$`; AC ids `NFR-006-AC-1`
  through `NFR-006-AC-8` are sequential with no gaps. No duplicate ID exists
  in `spec/requirements/` (`NFR-006` was the next free NFR id after
  `NFR-005-truthful-effectiveness-evidence.md`).
- **Frontmatter**: `type: NFR`, `quality_attribute: reliability` (matches the
  sibling `NFR-003`, `NFR-005`, and tl-rewrite's own `NFR-004`, all
  reliability-attribute gate/qualification requirements). `relationships`
  targets verified to exist: `ix://agent-ix/tl-mltl/FR-006` and
  `ix://agent-ix/tl-mltl/NFR-003` both resolve to real files in
  `spec/requirements/`.
- **Required sections present**: Statement, Measurement and Evaluation (table
  with the exact `Metric | Target | Threshold | Method` header), Verification
  — all present. Optional Scope, Rationale, and Acceptance Criteria (table
  with the exact `ID | Criteria | Verification` header) are also present and
  well-formed.
- **Test coverage rules**: every AC names a `Verification` method (`Test` x7,
  `Inspection` x1 for AC-8, which is a documentation-reference check rather
  than an executable one — matches how tl-rewrite's NFR-004-AC-8 and this
  repo's own NFR-003 "Automatic release decisions" row both use Inspection
  for non-executable verification facts).
- **Consistency against the real Makefile**: the 15-item declared `ci:`
  prerequisite list this requirement's Scope and AC-3/AC-4 name was checked
  against `Makefile` lines 253-254 directly rather than trusted from the
  draft — `ci: fmt-check lint kani-check test check-corpus conformance
  differential cli-conformance test-census deny audit-unsafe spec msrv
  rustdoc assurance`. Order and count (15) match exactly.

## EARS-conformance analysis

The Statement uses two EARS forms, both applicable here (state-driven,
event-driven) rather than a single ubiquitous form, matching tl-rewrite's own
proven NFR-004 statement shape:

- "If any prerequisite declared on the `ci` target does not both run its own
  recipe to completion and report success from that recipe's own outcome,
  then the CI entry point shall report `ci` as failed, naming every such
  prerequisite." — state-driven (`If <state> then <system> shall <response>`).
- "While invoking Make, the CI entry point shall refuse to proceed if the
  Makefile text or the invocation environment ... carries an
  execution-control state capable of suppressing prerequisite-failure
  propagation." — state-driven with an explicit precondition clause
  (`While <precondition>, <system> shall <response> if <trigger>`).

Both name a concrete subject (the CI entry point) and a concrete response
(report failed / refuse to proceed) rather than a vague quality claim. No
"fast"/"user-friendly"-class vagueness was found. A ubiquitous-form
alternative was considered and rejected: the behavior is genuinely
conditional on Make's own execution-control state, so collapsing it to an
unconditional "shall" would misstate the requirement.

## Dependency analysis

- `depends_on ix://agent-ix/tl-mltl/FR-006` (Adopt the shared assurance
  intake path): correct direction — NFR-006 is a control that protects the
  gate set FR-006's assurance intake depends on running honestly; FR-006
  does not depend back on NFR-006, avoiding a cycle.
- `extends ix://agent-ix/tl-mltl/NFR-003` (Make qualification controls
  explicit and fail closed): correct — NFR-003's own "What removing the Make
  guard actually costs, measured here" section explicitly states the
  residual this requirement closes and names the tracking issue
  (`agent-ix/tl-mltl#14`), so `extends` is the accurate relationship rather
  than an unrelated `depends_on` or `constrains`.
- No relationship to a StR was added. `StR-001` (reference semantics),
  `StR-002` (monitor interoperability), and `StR-003` (context-bound temporal
  results) were each read and none states a stakeholder need this control
  serves directly — they are about MLTL evaluation correctness and
  interoperability, not build/CI trustworthiness. `NFR-003`, the closest
  sibling requirement in this repository, likewise carries no `traces_to`
  StR relationship in its own frontmatter, so omitting one here follows
  existing repository convention rather than deviating from it.
- No dependency cycle introduced: `FR-006 -> (implements) -> StR-002`,
  `NFR-006 -> (depends_on) -> FR-006`, `NFR-006 -> (extends) -> NFR-003`, all
  acyclic.

## Notes

`quire validate --scope /Users/peter/dev/tl-mltl "spec/**/*.md"` and `quire
lint --module <spec-artifacts-iso> spec/requirements/NFR-006-gate-set-integrity.md`
both exit 0. Both commands print `DuplicateArchetype`/`DuplicateInverseEdge`
warnings that are pre-existing module-registration artifacts of this
environment's `spec-artifacts-process` module being contributed twice — the
same warnings print for an unrelated pre-existing file
(`spec/requirements/NFR-005-truthful-effectiveness-evidence.md`) and are not
attributable to this change.
