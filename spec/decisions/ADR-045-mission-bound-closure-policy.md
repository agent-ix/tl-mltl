---
id: ADR-045
title: "Refuse implicit mission-bound closure and allow it only by an explicit closure policy"
type: ADR
status: proposed
owner: tl-mltl-maintainer
relationships:
  - target: ix://agent-ix/tl-mltl/FR-090
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-094
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-syntax/ADR-044
    type: depends_on
---

# ADR-045: Refuse implicit mission-bound closure and allow it only by an explicit closure policy

## Status

Proposed (TL-427). Ready for human acceptance; not accepted. Whether the closure
conversion is built at all is an open decision for Peter (see Open decision).

## Context

Converting an unbounded formula to a bounded one with a mission time `M`
(`G p` to `G[0,M] p`, `p U q` to `p U[0,M] q`) changes meaning in known
cases. Over a finite trace closed-trace padding makes `G[0,M] p` false on a
short trace where LTLf `G p` is true; `F[0,M] p` can miss a witness an
unbounded `F p` finds. The closed-trace evaluator pads the proposition state at
positions past the end of the trace (every proposition false; tl-mltl
`src/future/evaluate.rs`, `proposition`, closed branch), not the truth of a
subformula, so next operators are affected too: a closed-trace `F[1,1] p` at the
last position reads the padded state, which strong next (false at the last
position) and weak next (true at the last position) do not. FRET exports bounded
MLTL after applying mission time, so meaning may already have changed before this
repository sees the formula.

An earlier draft bound the override to an acknowledgement holding the SHA-256 of
a closure report. That is dropped: an unbounded formula has no horizon, so the
report to be digested was undefined, and a caller can compute the digest from the
same library, so it attests identity and not that anyone read anything.

The repository already ships `export_safety_monitor`
(`src/infinite/export.rs`), which accepts `G[0,)psi` with a finite-horizon `psi`
and exports the body `psi` as a target expression with `refutation_only: true`
("a target pass is never proof of the infinite property"). It invents no bound and
claims no equivalence, so it is **not** a mission-bound closure. Its own refusals
(`UnboundedLiveness`, `UnboundedUntilRelease`, `UnsupportedShape`) are the
refusals of that boundary.

## Decision

Options considered: (A) refuse; (B) warn and proceed; (C) allow with an explicit
policy. **Recommendation: A by default; C only if Peter decides the conversion is
built.**

1. No importer, exporter, evaluator or API converts an unbounded formula
   (LTLf or infinite-trace) to a bounded formula implicitly. The attempt is the
   typed refusal `MissionClosureRequired` (also a `LossKind`, FR-091) and no
   artifact is emitted.
2. The retained `export_safety_monitor` is the one sanctioned partial path and is
   unchanged: it exports the body of `G[0,)psi` as a refutation-only expression
   and keeps its own error variants. Every other unbounded shape is refused by
   it with those variants, which are that exporter's expression of this refusal;
   it never returns a bounded formula and never returns `MissionClosureRequired`.
3. The closure **report** API (FR-094) is always built: given an unbounded formula
   and a mission time it returns, for each unbounded temporal node, the known-case
   row for that operator and mission time (computed table, FR-094), the closure
   the table assumes, and the statement that the meaning may change. It converts
   nothing.
4. The closure **conversion** (only if built) takes a required `ClosurePolicy`
   argument with no default and no `Default` implementation; the only converting
   variant is `ClosurePolicy::ChangeMeaning { mission_time }`, whose
   `mission_time` must equal the requested one. The output carries a provenance
   block: source canonical graph identity, the policy, `M` and the cited
   known-case rows. The policy attests only that the caller named this conversion
   and this `M`; it does not claim a human read a report.
5. Closure rules for the next operators: `Xs p` closes to `F[1,1] p` and `Xw p` to
   `G[1,1] p`, independent of `M`. Both are classified as changing meaning at the
   last position (the closed-trace form reads the padded all-false state and is true there
   exactly when the operand holds on that state, whereas strong next is false at
   the last position and weak next is true). `Xw` has no exact bounded closed-trace equivalent, so its
   known-case row cannot be "agrees on all traces".
6. There is no warning-only outcome: a warning is not machine-actionable and in a
   regulated pipeline is discarded; B is rejected.
7. Importing an already-bounded formula is not a closure by this library and is not
   refused. It is reported: where the horizon analysis shows the signature of a
   mission bound (FR-090), the report says the requirement was possibly changed
   upstream. The report never claims it knows the original meaning.

## Open decision (Peter)

Build the closure conversion (items 4 and the policy path of FR-094) or ship
refusal and report only. **Recommendation: refusal and report only.** Under that
recommendation no code path converts, `ClosurePolicy` is not introduced, and the
conditional criterion FR-094-AC-4 (TC-333) is withdrawn.

## Alternatives rejected

- B (warn): see item 6.
- Acknowledgement bound to a report digest: undefined for an unbounded formula and
  computable by the caller (Context).
- Allow silently with a documentation caveat: the failure this ADR exists to stop.
- Prove preservation per formula and allow when proven: valuable but beyond
  P-TL-151; the known-case classification of FR-094 is the interim aid.

## Consequences

The Rust API gains one refusal and one report function; with the open decision
taken the other way it also gains a policy type and a converting function.
