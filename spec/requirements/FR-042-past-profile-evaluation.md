---
id: FR-042
title: Evaluate the bounded past profile over origin-complete history
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/StR-001
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-001
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-syntax/FR-012
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-090
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-092
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-205
    type: depends_on
  - target: ix://agent-ix/quire-specification/FR-252
    type: depends_on
---

# FR-042: Evaluate the bounded past profile over origin-complete history

## Description

When a validated past-profile formula is evaluated at an anchor, tl-mltl shall
admit only an immutable, origin-complete position history, apply the closed v1
past operator vocabulary with its inclusive reverse-offset satisfaction
relation, and report the history the formula requires, under the declared
clock and resource limits.

tl-syntax owns the past operator vocabulary, graph admission, profile and wire
identities, and result attribution and correction relations (tl-syntax FR-011-AC-4,
FR-012-AC-3, FR-012-AC-4). This requirement owns the evaluator that gives those
nodes their truth values, the history it reads, the required-history analysis,
and the clocks and owner non-values it accepts. The criteria below carry the
evaluator text that tl-syntax withdrew when it re-homed evaluation here.

## Inputs

- A formula under profile `tl-syntax.past-operators/v1`, tagged
  `mltl.origin-complete-history/v1`.
- A `tl-mltl.position-history/v1` history: identity, revision, digest, origin
  position zero, through-position, and one observation per position with its
  Boolean propositions, under a declared clock.
- An evaluation anchor, a proposition-map identity, and evaluation limits.
- Checked inclusive intervals `[a,b]` with `0 <= a <= b <= u32::MAX`.

## Outputs

- A Boolean verdict at the anchor, or a typed refusal.
- A `tl-mltl.history-requirement/v1` analysis in positions.

## Behavior

Truth at anchor `t`:

| Node | Meaning at anchor `t` |
|---|---|
| `Once[a,b] p` / `O` | true iff some offset `j` in `[a,b]` makes `p` true at `t-j` |
| `Historically[a,b] p` / `H` | true iff every offset `j` in `[a,b]` makes `p` true at `t-j` |
| `StrongPrevious p` / `Y` | exactly the `Once[1,1] p` truth relation; primitive, and it does not imply a weak or physical-position-existence boundary |
| `p Since[a,b] q` / `S` | true iff some witness offset `j` in `[a,b]` makes `q` true at `t-j` and `p` is true at every offset `k` in `[a,j)` |
| `p Triggered[a,b] q` / `T` | exactly the Boolean dual `not((not p) S[a,b] (not q))` |

Offsets before `a` are irrelevant to Since, matching the future U lower-bound
convention in reverse. Triggered is a primitive canonical node whose semantics
equal its stated dual; it is not an independent truth table.

For a negative absolute position before origin, every proposition is false;
constants retain their Boolean values and Boolean and past nodes evaluate
recursively from those values. Subtraction uses checked offset comparison, not
unsigned wraparound. This is pre-origin false extension, not missing history.

History is complete only when it is nonempty, starts at origin zero, contains
exactly one observation for every position through its declared end, matches
its digest and revision, and includes the anchor. Empty, gapped, duplicate,
out-of-order, stale-digest, or out-of-range-anchor inputs refuse, and each
violated dimension has its own typed refusal. Pre-origin offsets generated
while evaluating an otherwise complete history use the false extension above
and are not gaps.

Required history is a maximum reverse offset, not a minimum physical trace
length:

- atoms and constants require zero;
- Boolean unary nodes retain their child's value and Boolean binary nodes take
  the maximum of their children;
- O and H add the interval upper bound to the child's required history;
- Y adds one position to its child's required history; and
- S and T add the interval upper bound to the maximum child requirement.

All additions produce `u64` and refuse overflow. The analysis depends only on
graph structure, so structurally equal formulas report the same value
regardless of source spans. Evaluation separately enforces the declared
recursion, temporal-span, step, and input-position limits; an interval
cardinality over the temporal-span limit refuses before iteration, and no
refusal falls back to a verdict.

A pure-past result is final for its exact anchor once history through that
anchor is complete. Silence advances nothing by itself. Two clocks are
admitted: an event-position clock, and a fixed-sample clock with a normalized
exact epoch `e`, a strictly positive normalized exact period `p`, and an
opaque bounded unit `u`, whose sample instant at position `n` is exactly
`e + n*p` in `u`. Formula interval bounds remain integer position offsets
under either clock; they never become elapsed-time durations. Multiplication,
common-denominator construction and addition use checked integer arithmetic;
overflow refuses before a history is constructed. No floating value, unit
alias, duration conversion, rounding, resampling or wall-clock input enters
the equation, and the unit is compared byte for byte. An absent runtime sample
under an otherwise valid fixed-sample binding is incomplete; an absent or
malformed binding, non-positive period, numeric overflow, unexpected sample, or
mismatched unit or instant refuses. Dense or timestamped events, duration
reinterpretation, rounding, implicit resampling and wall-clock inference are
unsupported and refuse.

Capture and typed-predicate evaluation remain outside TL. A native Quire
bridge resolves captures, anchors, clocks and total Boolean propositions before
constructing the history, and tl-mltl accepts only the resulting total Boolean
valuations. When the owner supplies an incomplete, unavailable, unsupported,
failed, refused, or conflict state instead of a history, tl-mltl returns that
state unchanged; it never turns absence into a false proposition or a
fabricated history row.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-042-AC-1 | For all valid histories, anchors, intervals, propositions, constants, and Boolean nests, O and H equal the stated existential and universal reverse-offset definitions, including `[0,0]`, a singleton history, and pre-origin extension, under both clocks. | property-based-testing (TC-178) |
| FR-042-AC-2 | For all valid inputs, S uses a witness in `[a,b]`, requires its left operand only at reverse offsets `[a,j)`, ignores offsets before `a`, and handles witnesses at both inclusive endpoints. | property-based-testing (TC-179) |
| FR-042-AC-3 | For all valid inputs, T is structurally distinct but evaluates exactly as the Boolean dual of S under every boundary and pre-origin case. | property-based-testing (TC-180) |
| FR-042-AC-4 | Y is a distinct primitive node whose truth equals `Once[1,1]` for propositions, constants and Boolean nests at every anchor including origin. | property-based-testing (TC-181) |
| FR-042-AC-5 | Every accepted history is nonempty, gap-free, duplicate-free, ordered from zero through its declared end, digest- and revision-consistent, and contains the anchor; each violated dimension returns a distinct refusal. | unit-testing (TC-182) |
| FR-042-AC-6 | Required-history analysis follows the stated recursive equations for atoms, constants, Boolean, O, H, Y, S and T nodes and reports the same value and formula identity for structurally equal formulas independent of source spans; a formula outside the past profile is refused. | unit-testing (TC-183) |
| FR-042-AC-7 | Required-history addition uses checked `u64` arithmetic and refuses overflow rather than wrapping or saturating. | unit-testing (TC-184) |
| FR-042-AC-8 | Event-position and fixed-sample histories following `sample(n)=epoch+n*period` in one exact opaque unit are accepted with bounds interpreted only as positions; overflow, malformed bindings, non-positive periods, timestamped or dense time, duration reinterpretation, rounding and implicit resampling refuse, absent samples are incomplete, and each owner-produced capture or predicate non-value is returned unchanged without Boolean coercion. | unit-testing (TC-185) |
| FR-042-AC-9 | Exceeding the recursion-depth, temporal-span, step, or input-position limit returns that limit's typed refusal with no verdict, including a Y whose one-position span exceeds a zero temporal-span limit. | unit-testing (TC-186) |

## Dependencies

tl-syntax FR-011 and FR-012 own the past vocabulary, profile identities and
attribution relations this evaluator consumes. FR-001 owns intervals. The native
clock, capture and predicate correspondence is owned by quire-contract-ir and
consumed from `ix://agent-ix/quire-specification/FR-090`, `FR-092`, `FR-205`,
and `FR-252`.
