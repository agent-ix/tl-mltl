---
id: FR-013
title: "Measure a deterministic Rust mutation campaign"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
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

A mutant path obeys FR-009's `/`-only exact UTF-8 path rules.
`mutantSha256` is lowercase hexadecimal SHA-256 over the UTF-8 bytes of
`tl-mltl.mutant/v1`, one zero byte, and the compact JSON array
`[repository,candidateRevision,path,originalSourceSha256,byteStart,byteEnd,originalClass,mutationOperator,replacementSha256,toolConfigurationSha256]`.
Offsets are checked `u64` UTF-8 byte offsets into the exact original source and
the half-open span must be in bounds and align with the tool's retained token or
AST record.

Discovery completes before selection. Rows are ordered by the closed matrix
rank `P0 < P1 < P2 < P3`, then mutation-operator ASCII bytes, then
`mutantSha256` bytes. Exclusions use a closed reviewed reason set and never
erase a discovered identity. If a declared hard cap prevents full execution,
selection takes the first rows in that order; all unselected mutants remain
listed and outside the score. `mutantPopulationSha256` is computed under domain
`tl-mltl.mutant-population/v1` over the compact JSON array of
`[candidateRevision,toolConfigurationSha256,discoveredRows]`, with each row
carrying its identity, priority sources, exclusion/selection state, and reason.
The digest is carried outside the hashed bytes.

The exact baseline command must pass three consecutive no-mutant controls before
any mutant runs. Stability means the same enumerated test-identity set and the
same pass/skip/fail outcome for each test; elapsed time and console spelling are
retained but are not equality inputs. Each selected mutant executes alone in a
clean isolated tree against the same locked test selection and finite timeout.
The selected order is divided into consecutive batches of 20, with the final
batch possibly shorter. A no-mutant control runs immediately before and after
each batch. If either control differs, every result in that batch becomes
`suspect`, is excluded from scoring, and must be rerun after the instability is
resolved. After every run, the source digest and repository status must match
the candidate before another mutant starts. A restoration or enumeration
failure aborts the campaign and prevents a score.

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
an outcome, denominator, or historical ratio. A raw `missed` or `timeout`
outcome may enter the FR-014 candidate census before its final disposition;
FR-014 does not depend on an `equivalent_within_declared_bound` disposition that
only its proof could justify. Every missed or timed-out mutant ultimately
receives exactly one reviewed disposition:
`test_gap` with a requirement-tagged regression ticket, `spec_gap` returning to
the specification cycle, `equivalent_within_declared_bound` with exact bounded
proof and reviewer,
`duplicate_of` another survivor, `tool_defect` with an upstream issue, or
`accepted_risk` with owner, rationale, expiry, and affected requirement. A
`duplicate_of` target must be a distinct survivor in the same population and
the directed duplicate graph must be acyclic and terminate at a non-duplicate
disposition. `accepted_risk.expiry` is an RFC 3339 UTC instant and an expired
record is unresolved. A machine guess cannot mark a mutant equivalent.
Survivors are prioritized by the same closed matrix rank and identity order; a
ratio never hides the queue.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-013-AC-1 | Discovery records every mutant under the specified path, span, identity, ordering, and population-digest rules before selection; exclusions and cap-driven unselected rows remain visible, and repeating discovery at the same revision/configuration produces the same ordered populations. | Test (TC-059, TC-060) |
| FR-013-AC-2 | A failed or inconsistent three-run baseline/control, concurrent mutants, changed test selection, timeout ambiguity, dirty restoration, missing source identity, or partial enumeration aborts or makes the affected batch suspect and cannot produce a score. | Test (TC-060, TC-061) |
| FR-013-AC-3 | Caught, missed, timeout, unviable, tool-error, not-run, and cancelled outcomes round-trip separately; the shared score uses caught over caught plus missed plus timeout only for a complete nonzero viable population and retains all adjacent counts. | Test (TC-061, TC-062) |
| FR-013-AC-4 | Every raw missed or timed-out mutant can enter the proof-candidate census without a circular final disposition and ultimately receives exactly one reviewed closed-set disposition with its required ticket/proof/owner/expiry evidence; duplicate chains are acyclic and expired risks are unresolved. | Test (TC-062, TC-063) |
| FR-013-AC-5 | Seeded mutant controls demonstrate that population, execution-state, score-denominator, isolation, restoration, and survivor-routing checks fail red while an unmutated control stays green. | Test (TC-064) |

## Dependencies

Depends on FR-009 path/retention rules and the FR-011 grounded property
baseline. FR-012 is a conditional input
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
