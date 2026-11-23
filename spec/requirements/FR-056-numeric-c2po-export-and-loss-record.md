---
id: FR-056
title: Export numeric predicates to C2PO or refuse with a typed loss record
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-004
    type: extends
  - target: ix://agent-ix/tl-mltl/FR-041
    type: extends
  - target: ix://agent-ix/tl-syntax/ADR-015
    type: depends_on
---

# FR-056: Export numeric predicates to C2PO or refuse with a typed loss record

## Description

When a formula with numeric predicates is exported, tl-mltl shall emit a C2PO
specification and a versioned numeric mapping manifest only if every node and
predicate has an exact representation under FR-057, and otherwise shall return a
typed loss record and neither expression nor manifest.

## Behavior

- A new entry point `map_numeric_to_c2po` consumes the validated bound formula,
  predicate table, catalog, event-position clock and the pinned target numeric
  profile; it emits `tl-mltl.numeric-mapping/v1`. Existing
  `map_to_c2po`, `map_past_to_c2po` and their manifests are unchanged
  byte for byte.
- `INPUT` declarations type each signal from its catalog domain and the target
  profile (integer or float type); comparison constants and thresholds are
  rendered by FR-057.
- **Loss record** `tl-mltl.numeric-loss/v1`: `{ cause, subject, detail }` with
  `cause` in a closed set and `subject` a proposition id, expression node id or
  signal id. Causes: `TargetNumericProfileUnpinned`, `TargetIntegerRangeExceeded`,
  `FloatMatchingUnsafe` (detail: grid spacing, error bound, tolerance, as exact
  rationals), `RescaleBeyondTarget`, `UnsupportedNumericOperator`,
  `PredicateWithoutDefinition`, `ForeignIdentity`, `ResourceExhausted`. The
  record is serialized with the strict-reader pattern; a refusal never carries a
  partial expression.
- **Partition.** Each numeric node kind (signal, integer, decimal, neg, abs, add,
  sub, mul, rescale, six comparisons) crossed with each interval/profile context
  that FR-041 already classifies has exactly one class, mapped or one cause above;
  the FR-041 table is extended, with no wildcard, and unknown additions fail
  closed.
- Projection to an FR-341-style result follows FR-041: unsupported export is
  `unsupported`, resource exhaustion is `failed`.
- The numeric export is counted: for a corpus, the totals of admitted and of
  each refusal cause are reported, and a refusal is never counted as agreement.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-056-AC-1 | Admitted predicates export to C2PO text that parses back to the expected expression structure and keeps graph, clock and catalog identities; legacy manifests are byte-identical. | property-based-testing (TC-1581, TC-1582) |
| FR-056-AC-2 | Each loss cause is produced by a minimal input, serialized, strictly read, and produces no expression and no manifest. | unit-testing (TC-1583) |
| FR-056-AC-3 | The node-kind by context partition has one class per cell and no wildcard, with a positive or negative test per cell. | unit-testing (TC-1584) |

## Dependencies

FR-004 and FR-041 own the existing export and refusal partition.
