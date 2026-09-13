---
id: MP-004
title: "TL retained fuzz-campaign effectiveness"
type: MeasurementPlan
status: proposed
owner: tl-verification-campaign-owner
metric: tl.fuzz-effectiveness
definition_version: tl-mltl.fuzz-effectiveness/v1
stage: baseline
statistical_design:
  population: every reviewed Fuzz-kind target selected from applicable FR-011 obligations at exact source and instrumentation identities
  sampling: three distinct seeds per target under tl-mltl.fuzz-baseline/v1, each capped at 900 target-execution seconds and 1000000 executed inputs with the first stopping condition controlling
  repetitions: 3
  estimator: per-target feature growth, plateau classification, corpus change, and independently reproducible crash counts with every stopping/result state reported separately
  error_model: stochastic seed sensitivity, generator/harness bias, unstable instrumentation, counter reset, build or replay contamination, timeout/resource failure, corpus corruption, and irreproducible crashes
  uncertainty: report all repetitions and their exact environments; feature identities compare only within one instrumentation identity and no no-crash probability or confidence interval is inferred
  decision_rule: retain crashes as remediation work and use a valid plateau only to prioritize seed or bounded-analysis review; refuse an effectiveness claim when any required run, budget, snapshot, identity, or artifact is missing
relationships:
  - target: ix://agent-ix/tl-mltl/FR-012
    type: measures
  - target: ix://agent-ix/tl-mltl/NFR-005
    type: measures
---

# TL retained fuzz-campaign effectiveness

## Decision Use

The measure identifies reproducible crashes, coverage growth, and bounded
plateaus that prioritize test and proof work. It does not prove absence of
defects or qualify a parser, evaluator, mapping, source language, or monitor.

## Population

The population is the closed selected target list at exact source, target,
toolchain, sanitizer, configuration, dictionary, starting-corpus, and
instrumentation identities. Excluded, blocked, unavailable, and not-run targets
stay in the campaign report with reasons.

Only tl-mltl targets are governed by this plan. A sibling TL repository must
accept a local MeasurementPlan and run its own owner-native producer; this plan
is a template, not cross-repository authority.

## Collection Procedure

Run all three declared RNG seeds under both caps, snapshot exact feature sets or
stable bitmap identities under FR-012, retain the starting/final corpora and
crash artifacts by digest, and replay and minimize crashes independently. Record
every stop and result state. Rust domain producers emit the record; Quoin owns
retention only where its accepted attachment and build-profile contracts can
represent the actual artifacts and execution.

## Interpretation

Compare exact feature identity sets or stable bitmap identities only inside one
instrumentation identity; a count alone is insufficient. Report each
seed, raw stop, corpus, crash, and limitation before any summary. A plateau is
only the FR-012 final-window observation and can justify review of a different
seed strategy or bounded method; no-crash and plateau states remain
non-authoritative.
