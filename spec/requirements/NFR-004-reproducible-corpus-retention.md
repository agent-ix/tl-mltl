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

The corpus campaign shall reproduce its results from exact versioned inputs while
preserving every gap, refusal, and limitation and leaving release and
qualification decisions to humans.

## Scope

Applies to the FR-008 cell list and oracle agreement, FR-009 fixtures and
generated inputs, the FR-010 comparison outcomes, and every coverage or
interoperability claim derived from them.

## Rationale

A high fixture count or percentage is misleading when a cell can silently lose
its cases, generated cases masquerade as conformance fixtures, or a target
refusal disappears into an agreement figure. Exact populations and distinct
states make change and remaining risk reviewable.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Cells in the cell list without a positive, negative, and boundary case | 0 | 0 | Test |
| Stored expected results not confirmed by the oracle or production | 0 | 0 | Test |
| Seeded evaluator faults the corpus fails to detect | 0 | 0 | Test |
| Generated inputs counted as canonical conformance | 0 | 0 | Test |
| Pending, unsupported, refused, or unavailable states reported as agreement | 0 | 0 | Test |
| Automated source-release, qualification, or certification decisions | 0 | 0 | Test |

## Verification

Deterministic Rust tests enumerate the cell list, replay the corpus, and apply
seeded faults. The campaign report reproduces from a recorded commit. An
independent reviewer and the release owner evaluate limitations separately from
automated results.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-004-AC-1 | Repeating the corpus replay and the campaign report at the same commit and corpus produces the same results, and a replay of a retained report refuses a moved or dirty source. | Test (TC-101, TC-103) |
| NFR-004-AC-2 | A dropped case, changed stored result, generated-for-canonical substitution, or collapsed comparison state makes its owning check fail and names the affected cell or fixture. | Test (TC-099, TC-101, TC-103) |
| NFR-004-AC-3 | Reports state the admitted, refused, unsupported, unavailable, and not-comparable counts separately, and never infer source release, qualification, certification, or monitor acceptance. | Test (TC-100, TC-103) |

## Dependencies

Constrains FR-008 through FR-010.
