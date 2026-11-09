---
id: FR-008
title: "Cover every operator and interval-shape cell with an oracle-confirmed corpus"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-003
    type: depends_on
---

# FR-008: Cover every operator and interval-shape cell with an oracle-confirmed corpus

## Description

When the boundary corpus is replayed, tl-mltl shall check that every operator and
interval-shape cell has a positive, a negative, and a boundary case, and shall
check that every stored expected result agrees with an independent oracle.

## Inputs

- The hand-authored boundary corpus (`boundary-v1`), owned by tl-syntax and read
  through the compiled dependency.
- The independent `tl-oracle` reference.
- The production evaluators for the closed-trace, online-prefix, and
  origin-complete-history profiles.

## Outputs

- A pass, or a failure that names the cell, edge condition, or case that is
  missing or in disagreement.

## Behavior

The cell list is plain data in the corpus, not a generated product. A cell is
one profile, one operator, and one interval shape.

| Dimension | Classes |
|---|---|
| profile | `mltl.closed-trace/v1`, `mltl.online-prefix/v1`, `mltl.origin-complete-history/v1` |
| future operator | Future, Globally, Until, Release |
| past operator | Once, Historically, Since, Triggered, Yesterday |
| Boolean and constants | true, false, Not, And, Or, Implies, Equivalent |
| interval shape | `[0,0]`, `[0,b]` with `b>0`, `[a,a]` with `a>0`, `[a,b]` with `0<a<b`, `[u32::MAX,u32::MAX]` |
| observation shape | empty, singleton, short of the required extent, exact, longer, early witness, early counterexample, no witness or counterexample |
| result | true, false, Pending, typed refusal |

Boolean connectives have no interval, so their interval class is
not-applicable and they need a positive and a negative case. An interval at
`u32::MAX` has only a refusal verdict. Derived W/M operators have no cell: they lower to
primitive graphs whose verdicts FR-016 verifies. A graph that mixes future and
past operators is refused by profile validation and has no cell.

The gap check fails when any cell lacks a positive, negative, or boundary case,
or when no case exercises a named edge condition: `a=b`, `a=0`, upper bound at
the trace end, empty trace, horizon equal to the prefix length, integer
extremes, nested windows that only just fit, and windows reaching before the
origin. The classes enumerate equivalence and boundary classes, not every
numeric value; property and sweep populations cover values inside a class and
never become canonical fixtures.

Every stored expected result is derived by hand from the profile definitions
before replay. Replay checks it against production and, for the closed-trace and
origin-complete-history profiles, against `tl-oracle`. The oracle shares only
the public canonical input types and does not call a production evaluator. The
online-prefix profile is not modelled by the oracle, so its progress results are
checked against production alone. A case tagged as work-limited or as one where
the oracle abstains is checked by production only. A refusal case stores the
refusal rather than a Boolean.

The corpus must be able to fail. Replay is also run against production with a
seeded fault (interval end off by one, lower bound off by one, dual operator
swapped, negation dropped), and each fault must turn the replay red.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-008-AC-1 | Every profile, operator, and interval-shape cell in the cell list has a positive, a negative, and a boundary case, every named edge condition is exercised, and a missing case fails the gap check naming the cell. | Test (TC-092) |
| FR-008-AC-2 | Every stored expected result is confirmed by the independent oracle where the oracle models the profile, and by production for every case; the two never disagree silently. | Test (TC-092, TC-093) |
| FR-008-AC-3 | Interval and observation partitions exercise every stated minimum, maximum, short, exact, long, witness, counterexample, progress, malformed, and resource boundary without treating a representative case as exhaustive value coverage. | Test (TC-093, TC-094, TC-095) |
| FR-008-AC-4 | Past-history cells are present for every past operator under the origin-complete-history profile, and derived W/M operators are verified through FR-016 lowering rather than a second cell population. | Test (TC-096, TC-097) |
| FR-008-AC-5 | A seeded evaluator fault or a stored result that disagrees with the oracle turns the replay red, and the oracle never calls a production evaluation path. | Test (TC-093, TC-101) |

## Dependencies

Depends on FR-001 through FR-003 for the evaluation semantics. The dimension
catalog admits no native-predicate or native-temporal correspondence class: that
dimension is `quire-mltl`'s under ADR-002.
