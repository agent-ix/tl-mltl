---
id: MP-005
title: "TL Rust mutation-pilot effectiveness"
type: MeasurementPlan
status: retired
owner: tl-verification-campaign-owner
metric: tl.mutation-effectiveness
definition_version: tl-mltl.mutation-effectiveness/v2
stage: baseline
objective:
  direction: zero
statistical_design:
  population: every mutant discovered at an exact green tl-mltl source revision and mutation configuration, partitioned into excluded, selected, and cap-unselected identities before execution
  sampling: complete selected population in deterministic reviewed-matrix-priority, operator-stratum, and identity order; a declared cap leaves the full unselected population visible
  repetitions: 1
  estimator: count
  error_model: nondeterministic baseline or tests, population drift, invalid exclusion, concurrent mutation, timeout ambiguity, unviable mutants, dirty restoration, tool failure, equivalent-mutant misclassification, and denominator suppression
  uncertainty: the pilot measures only the exact selected population; cap-unselected, excluded, unviable, cancelled, tool-error, and not-run rows remain explicit limitations, while timeout is a separately reported conservative non-kill
  decision_rule:
    comparator: eq
    threshold: 0
relationships:
  - target: ix://agent-ix/tl-mltl/FR-022
    type: measures
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: measures
---

# TL Rust mutation-pilot effectiveness

> Historical campaign artifact, superseded for V1 by MRS-004 and TM-006.
> Existing implemented tests and retained evidence remain available.

## Decision Use

The pilot measures whether the selected test configuration distinguishes a
declared set of source mutations and identifies concrete survivor work. It does
not define a universal acceptable score or approve release.

## Population

The discovered population and all exclusions are frozen before selection.
Every selected and cap-unselected identity remains visible. The exact source,
mutation tool/configuration, test command, toolchain, environment, timeout, and
resource limits are part of the population identity.

Only tl-mltl mutants are governed by this plan. Sibling repositories must
install and accept local plans and owner-native producers before collecting
their corresponding measures.

## Collection Procedure

Verify three stable baseline controls, execute each selected mutant alone with
interleaved no-mutant controls, capture its exact outcome, verify clean
restoration, and route missed/time-out rows under FR-022. Retain the complete
JSON producer record through Quoin MeasurementCollection v2. Abort scoring on
baseline, control, enumeration, isolation, or restoration failure.

## Interpretation

`statistical_design.decision_rule` evaluates only a count of closure blockers:
the missed or timed-out mutants that still lack one reviewed disposition, plus
the executed mutants, of any outcome, whose restoration check has not passed. It
holds -- and campaign closure is permitted -- only when that count is exactly
zero. The caught/(caught + missed + timeout) mutation score remains descriptive
and is never the evaluated quantity. No score is published when execution is
partial, that is, when any selected mutant lacks an execution state, or when the
viable denominator is zero. Report raw populations, outcomes, exclusions,
dispositions, and the missed/timeout queue before a killed fraction. The
fraction describes only a complete selected viable population, with timeout
conservatively counted as a non-kill and still reported separately.
A missed mutant is actionable evidence, not proof that the requirement or test
is wrong until its reviewed disposition establishes the cause.
