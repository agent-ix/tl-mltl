---
id: FR-013
title: "Measure a deterministic Rust mutation campaign"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-011
    type: depends_on
---

# FR-013: Measure a deterministic Rust mutation campaign

## Description

When an exact green TL candidate enters the mutation pilot, its owning
repository shall freeze the complete discovered mutant population and a
deterministic selected population before execution and shall retain every
mutant outcome and reviewed survivor disposition.

## Inputs

- Exact source revision, locked dependencies, compiler/toolchain, mutation tool
  and configuration, baseline command/result, and timeout/resource limits.
- Exact Quire 0.31 `implements` exports for the selected requirements plus the
  exact owning TestMatrix priorities established by FR-011.
- Closed mutation operator, path, generated-population, exclusion, and
  deterministic selection rules.

## Outputs

- An owner-native mutation-campaign record containing the discovered, excluded,
  selected, unselected, and executed populations; tl-mltl uses
  `tl-mltl.mutation-campaign/v1`.
- Stable mutant identities, isolated execution results, conclusive score when
  permitted, and one disposition for every missed or timed-out mutant.
- Typed refusal when the baseline is not green or the population, isolation,
  restoration, identity, or result state is incomplete.

## Behavior

A mutant identity binds normalized repository-relative path, original source
digest, byte span, original token/AST class, mutation operator, replacement,
tool/configuration identity, and exact candidate revision. Discovery completes
before selection. Exclusions use a closed reviewed reason set and never erase a
discovered identity. If a declared hard cap prevents full execution, selection
uses the stored identity digest order within each reviewed-priority and
operator stratum; all unselected mutants remain listed and outside the score.

The exact baseline command must pass three consecutive no-mutant controls before
any mutant runs. Each selected
mutant executes alone in a clean isolated tree against the same locked test
selection and finite timeout. A no-mutant control runs before and after every
batch of at most 20 mutants. If any control differs, every result in that batch
becomes `suspect`, is excluded from scoring, and must be rerun after the
instability is resolved. After every run, the source digest and repository status
must match the candidate before another mutant starts. A restoration or
enumeration failure aborts the campaign and prevents a score.

Execution states are `caught`, `missed`, `timeout`, `unviable`, `tool_error`,
`not_run`, and `cancelled`. Exclusion is a population state, not an execution
outcome. Timeout remains separately visible but contributes conservatively to
the shared mutation-score denominator; unviable contributes to neither side
because it exercised no test. The shared killed fraction is published only when
every selected mutant has an execution state and the viable denominator is
nonzero; it is `caught / (caught + missed + timeout)`, adjacent to every raw
population and exclusion. No adjusted score or target threshold is inferred
from the first pilot.

The observed collection and score are immutable; a disposition cannot rewrite
an outcome, denominator, or historical ratio. Every missed or timed-out mutant
receives exactly one reviewed disposition:
`test_gap` with a requirement-tagged regression ticket, `spec_gap` returning to
the specification cycle, `equivalent_within_declared_bound` with exact bounded
proof and reviewer,
`duplicate_of` another survivor, `tool_defect` with an upstream issue, or
`accepted_risk` with owner, rationale, expiry, and affected requirement. A
machine guess cannot mark a mutant equivalent. Survivors are prioritized first
by reviewed matrix priority, then deterministic identity; a ratio never hides
the queue.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-013-AC-1 | Discovery records every mutant identity before selection; exclusions and cap-driven unselected rows remain visible, and repeating discovery at the same revision/configuration produces the same ordered populations. | Test (TC-059, TC-060) |
| FR-013-AC-2 | A failed or inconsistent three-run baseline/control, concurrent mutants, changed test selection, timeout ambiguity, dirty restoration, missing source identity, or partial enumeration aborts or makes the affected batch suspect and cannot produce a score. | Test (TC-060, TC-061) |
| FR-013-AC-3 | Caught, missed, timeout, unviable, tool-error, not-run, and cancelled outcomes round-trip separately; the shared score uses caught over caught plus missed plus timeout only for a complete nonzero viable population and retains all adjacent counts. | Test (TC-061, TC-062) |
| FR-013-AC-4 | Every missed or timed-out mutant receives exactly one reviewed closed-set disposition with its required ticket/proof/owner/expiry evidence and is ordered by reviewed matrix priority before identity. | Test (TC-062, TC-063) |
| FR-013-AC-5 | Seeded mutant controls demonstrate that population, execution-state, score-denominator, isolation, restoration, and survivor-routing checks fail red while an unmutated control stays green. | Test (TC-064) |

## Dependencies

Depends on the FR-011 grounded property baseline. FR-012 is a conditional input
only for a mutation selection justified by a fuzz result; a repository with no
applicable Fuzz-kind boundary records that exclusion rather than fabricating
fuzz evidence. Each source repository owns its mutation producer and
remediation tickets. Quire's current criterion facts do not export an
authoritative production-symbol relation through the property records, but
Quire 0.31 separately exports the non-coverage `implements` relation delivered
after `agent-ix/quire-rs#171`. At this specification head it reports six bound
production symbols for FR-001 through FR-005. Before selection, the owning
repository shall complete and review that relation for every applicable
requirement in the mutation population and demonstrate that it does not change
the backed-evidence total. A TL producer shall consume the export rather than
invent a private mapping.
