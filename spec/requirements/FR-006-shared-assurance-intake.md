---
id: FR-006
title: Adopt the shared assurance intake path
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
---

# FR-006: Adopt the shared assurance intake path

## Description

The repository shall produce its evaluation, horizon, prefix, monitor-mapping,
differential and CLI results with its own tools in declared structured formats,
and shall obtain every static specification fact from Quire and every retention,
integrity, audit and receipt behaviour from Quoin, without either tool executing
a producer, without any gate executing the external monitor, and without a
repository-local generic evidence framework.

## Behavior

- Component versions are classified by the compatibility matrix packaged with
  the pinned Engineering Assurance release. This repository observes what is
  installed and restates no version rule of its own.
- One target, `make assurance-inputs`, writes the structured results the chain
  reads. Everything downstream consumes those files and refuses to create them;
  an absent input is an error naming that target, never a skip. The driver
  enforces this on itself: it refuses to start any child process other than the
  pinned Quoin CLI and a version observation.
- Each proof attestation states the verdict read out of the bytes its producer
  wrote. No verdict is inferred from a transcript, an exit code alone, or a
  caller's expectation.
- Retained evidence under `evidence/` is read through the Engineering Assurance
  compatibility mapping and is not modified. The mapping's answer is reported as
  it stands, including when that answer is a refusal.
- A malformed formula the shared corpus declares invalid is reported as
  malformed, rejected at its declared stage and naming its declared cause. It
  does not fail its proof obligation, and it is never transcribed as a pass.
- An R2U2 differential result stays a comparison. Agreement, mismatch and
  non-conclusive remain three answers; conclusive, pending, unsupported and
  tool-error remain four external states; and the reference truth value and
  verdict time are retained separately from the external ones.
- The external monitor is never executed by a gate. Its exact exchanged
  artifacts are retained and every claim is a replay against those bytes.
- Twelve verification outcomes remain distinguishable across the intake path,
  each demonstrated by a case that produced it, and each negative paired with a
  positive control observed to be accepted.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-006-AC-1 | The adopted component versions are classified by the packaged Engineering Assurance compatibility matrix, not by a local restatement of it, and no component resolves from the internal mirror. | Test (TC-018) |
| FR-006-AC-2 | Native evaluation, horizon, prefix, mapping, differential, CLI and test-census results are produced by this repository's tools in a declared structured format and transcribed by Quoin without Quoin, Quire, or any gate executing a producer or the external monitor. | Test (TC-019) |
| FR-006-AC-3 | Static specification, obligation, and coverage facts come from a Quire export that names every requirement in the repository, and Quire executes no producer. | Test (TC-020) |
| FR-006-AC-4 | Retained evidence bytes are read through the Engineering Assurance compatibility mapping without being modified, and the mapping's answer is reported without collapsing it into pass or fail. | Test (TC-021) |
| FR-006-AC-5 | Pass, fail, unavailable, unsupported, inconclusive, not-computed, malformed, partial, stale, suspect, vacuous, and tampered remain twelve distinguishable states, each demonstrated and each negative paired with a positive control. | Test (TC-022) |
| FR-006-AC-6 | An R2U2 differential result is never reduced to a boolean: three comparison classifications and four external-monitor states are observed and stay separate, every supported case is compared, the declared unsupported case is reported unsupported rather than compared, the counts agree with the corpus manifests' own declarations, and the states survive into the bytes Quoin retained. | Test (TC-023) |
| FR-006-AC-7 | No repository-local generic runner, evidence envelope, manifest, identity framework, retention store, audit store, anchor verifier, failure-propagation policer, or aggregate verdict remains in the execution path, and the two retained evidence schemas are frozen and referenced by nothing. | Test (TC-024) |

## Dependencies

Depends on the released Engineering Assurance, quire-cli, Quoin and ix-flow pins
recorded in `assurance/pins.json`, and on FR-001 through FR-005 for the domain
behaviour whose results it transcribes.
