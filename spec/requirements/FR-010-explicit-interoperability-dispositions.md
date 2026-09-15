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
| FRETish | `quire-contract-ir` Rust-emitted output-only FS06 mapping | Electron/Node execution is unavailable and prohibited as production or qualification evidence |

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

Native Quire is not a target in this catalog. It remains the sole editable
source authority. Native-bridge cases preserve the native source digest/revision/clause/span,
model/type, predicate/catalog, anchor/capture/clock/history, TL formula/profile,
evaluator/result, and loss/refusal identities defined by #63/#64. Until both
bridges are accepted and executable, those cells remain blocked with no
fabricated Boolean or FRETish fallback.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-010-AC-1 | Every target case retains independent loss, availability, and comparison axes; every combination in the closed validity table round-trips, every other combination refuses, conditional/refusal reasons are required, and no non-conclusive/unsupported/unavailable state becomes Boolean success. | Test (TC-046, TC-047) |
| FR-010-AC-2 | C2PO/R2U2 claims remain bounded to the exact retained cases and identities, while FRETish remains output-only and no build, test, CI, or qualification path executes Java, Node, Electron, C2PO, or R2U2. | Test (TC-046, TC-048) |
| FR-010-AC-3 | Each admitted external observation binds exact tool/source/configuration/input/raw-output/environment/license/contributor digests; malformed, missing, stale, digest-mismatched, or profile-incompatible inputs refuse before comparison, while admitted partial, failed, aggregation-ambiguous, or verdict-absent output is available but non-conclusive. | Test (TC-046) |
| FR-010-AC-4 | Rust mapping correctness is tested against canonical TL semantics, exact expected output, and complete loss reports without using parser acceptance or external execution as a semantic oracle. | Test (TC-039, TC-046) |
| FR-010-AC-5 | Native bridge cells preserve every #63/#64 source, model, predicate, capture, clock, history, formula/profile, evaluator/result, and loss identity or remain blocked with no Boolean or external-source fallback. | Test (TC-043, TC-046) |

## Dependencies

Depends on FR-004/FR-005 mapping and comparison semantics, FR-007 contextual
identity, and FR-009 fixture identity. Native rows depend on reviewed #63/#64;
future-derived and past rows depend on accepted tl-syntax #37/#38 and their
routed evaluator/adapter implementations.
