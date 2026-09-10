---
id: NFR-004
title: "Keep corpus coverage reproducible and non-authoritative"
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-mltl/FR-008
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-009
    type: constrains
  - target: ix://agent-ix/tl-mltl/FR-010
    type: constrains
---

# NFR-004: Keep corpus coverage reproducible and non-authoritative

## Statement

The corpus campaign shall reproduce its complete declared population from exact
versioned inputs while preserving every gap, exclusion, limitation, and human
authority boundary.

## Scope

Applies to the FR-008 cell census, FR-009 fixture manifests and lifecycle, the
FR-010 target catalog, MP-002 results, and every coverage or interoperability
claim derived from them.

## Rationale

A high fixture count or percentage is misleading when the denominator can
silently shrink, generated cases masquerade as conformance fixtures, or target
unavailability disappears. Exact populations and distinct states make change
and remaining risk reviewable.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Declared v1 coverage cells classified exactly once | complete population | 100% | Test |
| Applicable cells with one canonical fixture and independent expected result | complete applicable population | 100% | Test |
| Excluded or blocked cells without a reason/dependency | 0 | 0 | Test |
| Duplicate case/cell identities or silent removals | 0 | 0 | Test |
| Canonical artifacts with missing digest, owner, source, license, or limitation | 0 | 0 | Test |
| Generated inputs counted as canonical conformance | 0 | 0 | Test |
| Unsupported/unavailable/non-conclusive target states collapsed or omitted | 0 | 0 | Test |
| Automated source-release, qualification, or certification decisions | 0 | 0 | Test |

## Verification

Deterministic Rust validators enumerate the declared population, verify strict
schemas and digests, and mutation-test denominator, status, identity, and
lifecycle controls. Quire reports requirement/matrix backing, and Quoin retains
producer results under shared ownership. An independent reviewer and the human
release owner evaluate limitations separately from automated results.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-004-AC-1 | Repeating the census and replay with identical repository, dependency, corpus, tool, configuration, and environment identities produces byte-identical ordered machine records. | Test (TC-047, TC-049) |
| NFR-004-AC-2 | Every denominator change, duplicate, silent removal, status lie, digest/provenance omission, generated/canonical substitution, or collapsed target state makes its owning control fail and names the affected identities. | Test (TC-045, TC-047, TC-049) |
| NFR-004-AC-3 | Reports publish applicable, excluded, blocked, covered, unsupported, unavailable, non-conclusive, and not-run populations separately and never infer source release, qualification, certification, or monitor acceptance. | Test (TC-046, TC-049) |

## Dependencies

Constrains FR-008 through FR-010 and uses the shared assurance boundary in
FR-006. Human release authority remains governed by AP-001 and PGM-01.
