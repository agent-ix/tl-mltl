---
id: FR-029
title: Route infinite-trace evidence through the opt-in provider
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-027
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-028
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-291
    type: depends_on
---

# FR-029: Route infinite-trace evidence through the opt-in provider

## Description

When infinite-trace capability evidence is reported, tl-mltl shall attribute
lasso, fairness and inductive-semantics results to `tl_mltl::infinite`, its
exact feature-enabled source revision, `mltl.infinite-trace/v1` profile and
the scope of the selected subject.

## Inputs

- The accepted syntax, provider, oracle and qualification revisions and their
  exact feature selections.

## Outputs

- A dependency/evidence route from tl-syntax admission through the provider
  registration and verification campaign, retaining each owner identity.

## Behavior

The dependency order is Linear TL-207 syntax admission, completed STD-13 QSpec
FR-160/161 authority at revision `2449ceb`, TL-210 provider semantics and
registration, then TL-211 export and TL-212 verification. Implementation
starts only after the combined TL-215 review is accepted. TL-13 implements
the provider; TL-7 consumes the qualified result. Bounded results remain
attributed to existing bounded APIs. The QSL
`quire.temporal.infinite-trace/v1` member is a comparison correspondence and
is never emitted as the TL profile. A green provider test or Quoin receipt
does not itself grant a source release or native parity decision.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-029-AC-1 | The dependency route names TL-207, STD-13, TL-210, TL-211, TL-212, TL-215, TL-13 and TL-7 with one owner and predecessor per step. | Inspection (TC-088) |
| FR-029-AC-2 | Every infinite result names the exact module feature, provider revision, TL profile, graph, clock and subject scope; bounded results are not reattributed. | Test (TC-089, TC-140) |
| FR-029-AC-3 | No infinite implementation or qualification claim is advanced before syntax and combined-review acceptance gates; a missing provider settles `unsupported`. | Inspection (TC-088) |

## Dependencies

FR-027/028 establish the provider boundary; tl-syntax FR-291 supplies the
upstream syntax evidence route.
