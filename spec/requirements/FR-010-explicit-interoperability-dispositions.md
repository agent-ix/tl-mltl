---
id: FR-010
title: "Record explicit interoperability and loss dispositions"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-004
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-005
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-007
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: depends_on
---

# FR-010: Record explicit interoperability and loss dispositions

## Description

When a corpus case crosses a target-mapping or external-observation boundary,
the campaign shall retain separate semantic-loss, target-availability, and
comparison states and shall never promote target acceptance into equivalence or
qualification.

## Inputs

- Exact source formula/profile/context and FR-009 fixture identity.
- Versioned Rust mapping profile, target language/version, and emitted bytes or
  typed mapping refusal.
- Optional retained external observation with exact tool/configuration/input/
  output identity and execution provenance.

## Outputs

- Per-obligation loss state: `preserved`, `conditional`, `unrepresented`, or
  `refused`, with reason and source span/identity.
- Target availability: `available`, `unavailable`, or `unsupported`.
- Optional comparison: `agreement`, `mismatch`, or `non_conclusive`, never a
  Boolean qualification result.

## Behavior

The three state axes describe independent facts but their serialized
combinations are closed:

| Loss | Availability | Comparison | Valid meaning |
|---|---|---|---|
| `preserved` or `conditional` | `available` | absent | exact mapping and observation exist; comparison was not requested |
| `preserved` or `conditional` | `available` | `agreement`, `mismatch`, or `non_conclusive` | exact mapping and observation were compared |
| any loss state | `unavailable` or `unsupported` | absent | no admitted external observation exists |
| `unrepresented` or `refused` | `available` | absent | an independently retained observation exists, but no comparable emitted mapping exists |

Every other combination refuses. `conditional` also requires a nonempty closed
condition list; `unrepresented` and `refused` require a typed reason and emit no
target bytes. `preserved` describes a reviewed mapping, not target execution.
`available` says an exact observation exists, not that it agrees. `agreement`
applies only to matching exact identities and exercised values, not the target
language, tool, monitor, source profile, or unexercised cells. Unsupported and
unavailable are retained rather than dropped from the population.

The target catalog is:

| Target | Mapping role | External-observation rule |
|---|---|---|
| C2PO/R2U2 | `tl-mltl` Rust-emitted monitor/mapping target for reviewed profile subsets | the retained 4.2/C2PO 4.1 exchange remains an exact eight-case observation only; new execution is unavailable unless separately authorized |

The catalog is closed to targets `tl-mltl` itself emits. A target emitted by an
agent-ix/Quire repository is not a row here: under ADR-002 its dispositions are
`quire-mltl`'s, and naming one is an `unsupported` target/profile combination in
this catalog.

An external observation is admitted only when its exact executable or source
revision, configuration, input, raw output, invocation environment, license,
contributor, and artifact digests are present and mutually consistent. Unknown
fields or enums, missing required identity, digest mismatch, stale context, and
profile/clock/history mismatch are typed admission refusals and emit no
comparison. An admitted exact run whose raw output records tool failure,
partial output, ambiguous aggregation, or no comparable verdict is
`available` with `non_conclusive` comparison and a typed reason. No authorized
or retained run is `unavailable`; a target/profile combination outside the
catalog is `unsupported`. None can be rewritten as agreement.

Build, test, CI, and qualification paths do not execute an external monitor or
foreign runtime. A manually supplied observation remains contextual evidence;
it does not discharge mapping correctness. Mapping correctness is verified from
the canonical TL semantics, exact Rust emitter, loss report, and independent
expected output. Parser acceptance is at most an observation and never a
semantic oracle.

The canonical `tl-syntax` formula graph is the source authority for every case
in this catalog. A disposition cannot be derived from, or fall back to, a
source language this repository does not own.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-010-AC-1 | Every target case retains independent loss, availability, and comparison axes; every combination in the closed validity table round-trips, every other combination refuses, conditional/refusal reasons are required, and no non-conclusive/unsupported/unavailable state becomes Boolean success. | Test (TC-100, TC-101) |
| FR-010-AC-2 | C2PO/R2U2 claims remain bounded to the exact retained cases and identities, a target outside the closed catalog is `unsupported` rather than admitted, and no build, test, CI, or qualification path executes Java, Node, Electron, C2PO, or R2U2. | Test (TC-100, TC-102) |
| FR-010-AC-3 | Each admitted external observation binds exact tool/source/configuration/input/raw-output/environment/license/contributor digests; malformed, missing, stale, digest-mismatched, or profile-incompatible inputs refuse before comparison, while admitted partial, failed, aggregation-ambiguous, or verdict-absent output is available but non-conclusive. | Test (TC-100) |
| FR-010-AC-4 | Rust mapping correctness is tested against canonical TL semantics, exact expected output, and complete loss reports without using parser acceptance or external execution as a semantic oracle. | Test (TC-093, TC-100) |

## Dependencies

Depends on FR-004/FR-005 mapping and comparison semantics, FR-007 contextual
identity, and FR-009 fixture identity. Future-derived and past rows depend on
accepted tl-syntax #37/#38 and their routed evaluator/adapter implementations.
