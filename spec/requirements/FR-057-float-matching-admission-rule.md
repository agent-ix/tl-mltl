---
id: FR-057
title: Admit a numeric predicate to export only when float matching cannot change a verdict
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-014
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-056
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-015
    type: depends_on
---

# FR-057: Admit a numeric predicate to export only when float matching cannot change a verdict

## Description

When exporting a numeric predicate, tl-mltl shall apply the ADR-015 admission test in exact rational arithmetic against a recorded target numeric profile and emit midpoint thresholds for admitted predicates.

## Inputs

- `TargetNumericProfile { integer_bits, float_bits (32 or 64), equality_tolerance
  (exact rational or none), evidence }`, read from the retained exchange record
  (FR-058). Absent or unpinned means `TargetNumericProfileUnpinned` for every
  numeric predicate.

## Behavior

- For a predicate with scale `s`, the grid spacing is `g = 10^-s`. The rounding
  bound `R` is the sum over expression operations of one half-ulp at the maximum
  magnitude on the declared domain, at `float_bits`, as an exact rational
  (binary32: `2^-24` relative; binary64: `2^-53` relative); `T` is the profile's
  tolerance or zero. Admit iff `R + T < g/2` and all domain bounds and literals
  fit `integer_bits` (Integer) or the target float range (Decimal).
- Threshold rendering: `x < c` as `x < c - g/2`; `x <= c` as `x < c + g/2`;
  `x > c` as `x > c + g/2`; `x >= c` as `x > c - g/2`; `x == c` as
  `x > c - g/2 && x < c + g/2`; `x != c` as its negation. Where the whole
  predicate has the form `e op c`, `e` is the left expression and `c` the folded
  constant of the right; a general `e1 op e2` is rendered as `e1 - e2 op 0`
  first, which is admissible only if the combined `R` bound includes the
  subtraction.
- All arithmetic of the test uses `i128` rationals with checked operations; an
  overflow is `ResourceExhausted`, never an admission.
- The exact semantics of tl-mltl FR-054 is the oracle: an admitted predicate's
  exported form shall agree with FR-054 at every grid value of the declared domain
  (verified by exhaustive grid enumeration for domains up to 2^16 values per signal
  and by sampled boundary values otherwise).
- Unpinned profile values are DEFERRED (FR-058); tests use an explicitly labelled
  synthetic profile and never claim it describes R2U2.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-057-AC-1 | With a synthetic profile, every predicate in a declared grid is admitted or refused exactly as the closed-form test dictates, and admitted exports agree with exact semantics at every grid value including `x == c` boundaries. | property-based-testing (TC-1585, TC-1586) |
| FR-057-AC-2 | With no pinned profile every numeric predicate is refused with `TargetNumericProfileUnpinned`, and pure Boolean export is unchanged. | unit-testing (TC-1587) |
| FR-057-AC-3 | A profile whose `R + T` is at or above `g/2` refuses with `FloatMatchingUnsafe` carrying the exact rationals. | unit-testing (TC-1588) |

## Dependencies

FR-056 emits; FR-058 supplies the pinned profile.
