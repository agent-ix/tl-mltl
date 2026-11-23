---
id: ADR-004
title: "Define requirement realizability as a fixed-horizon, inputs-first game with checkable evidence"
type: ADR
status: proposed
owner: kreneskyp
relationships:
  - target: ix://agent-ix/tl-mltl/FR-002
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-043
    type: realizes_requirement
  - target: ix://agent-ix/tl-syntax/FR-034
    type: realizes_requirement
  - target: ix://agent-ix/tl-solve/FR-100
    type: realizes_requirement
  - target: ix://agent-ix/tl-solve/FR-102
    type: realizes_requirement
  - target: ix://agent-ix/tl-solve/FR-104
    type: realizes_requirement
  - target: ix://agent-ix/tl-solve/FR-106
    type: realizes_requirement
  - target: ix://agent-ix/tl-solve/FR-108
    type: realizes_requirement
  - target: ix://agent-ix/quire-specification/FR-175
    type: relates_to
---

# ADR-004: Define requirement realizability as a fixed-horizon, inputs-first game with checkable evidence

## Status

**Proposed. Ready for human acceptance.** This record is the TL-413 decision
record for project P-TL-149. It has not been accepted; no requirement that
cites it may be called implemented against an accepted decision until a human
owner accepts it. One decision, number 5, crosses into the sibling project
P-TL-150 and is flagged for the program owner.

## Context

A requirement set can be satisfiable and still impossible to implement,
because it only holds when the environment cooperates. P-TL-147 gives TL a
satisfiability engine (tl-solve): is there some trace on which the
requirements hold. Requirement realizability asks a different question. Split
the signals into inputs, which the environment controls, and outputs, which the
system controls. Can some implementation choose outputs so that the
requirements hold for every possible input sequence?

No published work on MLTL realizability was found while specifying this
project, so the semantics must be stated here and not cited. Related results
exist for neighbouring logics: LTL and LDL on finite traces
(De Giacomo and Vardi, IJCAI 2015), requirement realizability checking in NASA
FRET (CAV 2022), and strong satisfiability (Shimakawa, Hagihara and Yonezaki).
They inform the shape of the checks; none of them define MLTL with explicit
inclusive intervals over a fixed trace, so none is a normative source here.

Controller synthesis is out of scope. The answer is a verdict plus evidence
that a separate, simpler procedure can check. It is not generated code.

tl-mltl already owns the closed-trace reference semantics
(`evaluate_closed_at`, `EvaluationLimits`) and the horizon analysis
(`analyze_horizon`, whose report carries `lookahead` and `required_buffer`).
tl-syntax owns the signal catalog (`SignalCatalog`, `SignalCatalogDocument`).
tl-solve, created by P-TL-147, owns the encoder and solver boundary on which
the checks run.

## Decision

1. **Name.** The problem is called *requirement realizability*. Every string
   identity that names the problem (wire identity, schema id, file name,
   command and subcommand, document title, result member and message) carries
   the two words together, as `requirement-realizability` or `requirement
   realizability`; the bare word "realizability" names the problem in no string
   identity this project mints. Rust items minted inside the module
   `requirement_realizability` of tl-solve (`RealizabilityProblem`,
   `RealizabilityRequestError`, `RealizabilityLimits`,
   `RealizabilityCertificate`) may use the short stem, because the module path
   is the qualification: they are exposed only by that path and never
   re-exported at a crate root or under another name. The verdict labels
   `Realizable` and `Unrealizable` name an outcome, not the problem, and are
   qualified by the envelope identity. Protocol realizability (decision 9) keeps
   its own name.

2. **Fixed horizon.** For a request over formulas `phi_1 ... phi_k` the play has
   exactly `L = H + 1` steps, numbered `0 ... H`, where `H` is the maximum
   `lookahead` reported by tl-mltl `analyze_horizon` over the formulas. The
   system cannot end the play early and the environment cannot end it at all.
   `L` is never supplied by the caller: a caller value that differs from the
   derived one is refused. Only the `mltl.closed-trace/v1` semantic profile is
   admitted. `lookahead` is an upper bound on the last position a formula reads
   (it sums interval ends along nesting), not always the exact last position.
   Because `H` bounds every read from position zero, no formula reads a
   position at or beyond `L`, so the closed-trace convention of an all-false
   valuation after the declared end is never exercised and cannot favour either
   player. A looser bound adds steps no formula reads, which cannot change a
   verdict.

3. **Turn order.** In each step `t`, the environment first chooses the
   valuation of the input propositions, then the system, having seen it,
   chooses the valuation of the output propositions. The system therefore
   knows `i_0 ... i_t` and its own earlier choices `o_0 ... o_(t-1)` when it
   chooses `o_t`. It cannot know `i_(t+1)`. A strategy is a function from the
   input history through step `t` to `o_t` (Mealy-style); a counter-strategy is a
   function from the output history through step `t-1` to `i_t`. The system
   wins a play exactly when `evaluate_closed_at` at position zero returns an
   `EvaluationReport` whose verdict is `TruthValue::True` for every requirement
   formula. This is the only winning condition; the
   search, the re-check and every export are defined to agree with it.

4. **Declaring inputs and outputs.** Direction is a per-signal property of the
   tl-syntax signal catalog, carried by a new document version
   `tl-syntax.signal-catalog/v2` (tl-syntax FR-034). A proposition has the
   direction of the signal it is bound to. Existing v1 catalogs stay valid and
   read as `Undeclared`. A realizability request that references any signal
   with no declared direction is refused with a typed error before any search,
   solver call or export; there is no default direction and no inference from
   formula shape or naming. Direction is a declaration by the caller, not a
   checked property of any system.

5. **Environment assumptions: assume-guarantee contracts (OPEN DECISION FOR
   PETER; inter-project with P-TL-150; nothing below is accepted).**
   Environment assumptions are expressed as the assume-guarantee contracts of
   P-TL-150 (TL-425), and are not a separate TL-149 assumption syntax. This
   decision does not select among the options A, B and C of tl-syntax ADR-017
   (P-TL-150, "Contract interface required by realizability"); those are
   decided by Peter for the contract form, and tl-syntax FR-311-AC-2 forbids any
   realizability document from citing a contract as an assumption before that
   choice is recorded. What follows is therefore conditional, a proposal for the
   winning condition on top of whatever ADR-017 records, and V1 of this project
   accepts only plain requirement sets (the conjunction of formulas that must
   all hold). A request that carries a contract returns `Unsupported` with a
   reason naming contracts. No contract wire form is minted by P-TL-149.

   *Status vocabulary consumed.* P-TL-150 ADR-017 and tl-mltl FR-059 define the
   status of a contract at one evaluation position: `inactive` (assumption
   false), `invalid` (assumption true, guarantee false), `verified` (both true)
   and `pending` (not decided), reported as a per-position series, never
   collapsed.

   *Proposed default: the guarantee holds at every position at which the
   assumption has held so far.* Let `A` and `G` be the two roots of one
   contract and `look(c) = max(lookahead(A), lookahead(G))`. A position `p` is
   settled for the contract when `p + look(c) <= H`, so that no read at `p`
   reaches the end of the play. Over a complete play the system loses exactly
   when some settled position `p` has status `invalid` and no earlier position
   `q < p` has status `inactive` (the assumption has held at every position so
   far). Positions that are not settled are `pending` in the sense of ADR-017
   and never decide a play. Because settledness is computed here from
   `lookahead`, the closed-trace padding convention is not exercised for any
   contract position, which preserves the argument of decision 2. If the
   tl-mltl FR-059 series for a complete play reports `pending` at a settled
   position, that is a `Failed` result (a tool fault), never a verdict. This is
   the stronger reading ("`G` while `A` held") and replaces the whole-trace
   implication of the earlier draft; it removes the late-violation excuse that
   draft listed as limit (a).

   *Input-only rule (requirement of P-TL-149 on P-TL-150).* Every proposition
   occurring in `A` shall be bound to an `Input` signal, checked by tl-solve
   FR-100 against the tl-syntax partition. An assumption that mentions an
   output is refused with a typed request error, because the system could win
   by falsifying its own assumption. P-TL-150 shall expose `A` and `G` as
   separate validated roots in profile `mltl.closed-trace/v1` under one stable
   contract identity (tl-syntax FR-311 already does), shall keep the per-position
   series of FR-059 as it is, and shall not collapse statuses into a boolean.
   P-TL-149 asks nothing else of P-TL-150.

   *Residual limits, stated so the owner can reject them.* (a) An assumption
   with `lookahead(A) > 0` is a statement about future inputs, and the system
   may exploit it at a position before those inputs are chosen; only an
   assumption of lookahead 0 is causal. (b) An assumption no input sequence
   satisfies makes every guarantee realizable vacuously, so a contract
   request's result carries a mandatory assumption-satisfiable flag, computed
   by one satisfiability query over inputs, and a `false` flag is reported
   beside the verdict. (c) The whole `Realizable` label for a contract request
   is a verdict about this winning condition only and states which condition.

   *Open decision for Peter.* Pick one; until then contracts stay
   `Unsupported`.

   | | Winning condition for contract `(A, G)` | Consequence |
   |---|---|---|
   | 1 (proposed default) | `G` holds at every settled position at which `A` has held at all earlier and current positions: consumes the per-position series; the "`G` while `A` held" reading | Needs the series of FR-059 and the Input-only rule. Fits ADR-017 option C as written ("no contract is ever `invalid`" over the behaviours this project constrains). Positions are judged by this project's settledness rule. |
   | 2 (earlier draft) | the single implication `A -> G` judged once at position 0 of the whole trace | Needs only the roots of FR-311. Weaker: a late assumption violation excuses an earlier guarantee failure for plays the environment would not choose. Ignores the series. Does not use ADR-017 option C's per-position reading. |

   ADR-017 option A (invariant assumptions, a `contract-set/v2` with an explicit
   scope) and option B (a separate assumption document) remain open for
   P-TL-150 under either row; they change the contract form, not the choice
   above.

6. **Result labels.** A completed request yields exactly one of five labels.
   A malformed request, including an undeclared signal, is a typed request
   error and not a label.

   | Label | Meaning | Evidence carried |
   |---|---|---|
   | `Realizable` | A strategy exists and the strategy was re-checked | Strategy certificate and re-check outcome `confirmed` |
   | `Unrealizable` | A counter-strategy exists and was re-checked | Counter-strategy certificate and re-check outcome `confirmed` |
   | `Inconclusive` | No answer within a declared limit, or the necessary check passed and the full game was not run | Typed reason with the limit and the value reached |
   | `Unsupported` | The request is well formed but outside the method's declared scope | Typed reason (for example past operators, unbounded profile, non-Boolean signal, contracts) |
   | `Failed` | The tool itself failed: a re-check disagreed with the search, a certificate was malformed, or a backend faulted | Typed reason and the rejected certificate |

   Passing the necessary check is never `Realizable`. A failed re-check is
   `Failed` and is never reported as either verdict. A re-check that exhausts
   its own budget without confirming or refuting is `Inconclusive` with the
   certificate attached and marked unverified; it is not a verdict.

7. **Checkable evidence.** The search that finds a strategy is not trusted.
   Every verdict carries a certificate in one format
   (`tl-solve.requirement-realizability-certificate/v1`) and the outcome of a
   satisfiability query that checks it against the tl-solve encoder of the
   original formulas, not against the search's own state (tl-solve FR-105 and
   FR-106). A necessary-check failure is reported as an `Unrealizable` result
   whose counter-strategy is the input sequence, in the same format. Results
   from external tools are reported as external and unchecked unless a
   certificate comes back and passes the same re-check (tl-solve FR-107,
   FR-108).

8. **Honest limits.** No method here is claimed to scale. The table below is
   normative for what the specification and the documentation may say.

   | Method | Decides | Worst case | Declared limits | Beyond a limit |
   |---|---|---|---|---|
   | Necessary check (TL-415) | Sound for `Unrealizable`; passing proves nothing about realizability | Up to `2^(\|I\|*L)` refinement rounds, each two satisfiability calls | Maximum rounds, per-call solver budget, encoder size | `Inconclusive` with the limit |
   | Full game (TL-416) | Exact for `Realizable` and `Unrealizable` | Reachable canonical states are bounded only by the formula and `L`; per state `2^\|I\| * 2^\|O\|` successor evaluations. No polynomial bound is claimed | Maximum states, input bits, output bits, trace length | `Inconclusive` with the limit |
   | Re-check (TL-417) | Confirms or refutes one certificate | One satisfiability call over the encoded formulas and the certificate | Solver budget, certificate size | `Inconclusive` (budget) or `Failed` (refuted or malformed) |
   | QDIMACS export (TL-418) | Nothing by itself; hands the problem to a QBF solver | Encoding size of the closed-trace encoder plus the quantifier prefix | Encoder size, variable count | Typed export refusal |
   | TLSF export (TL-418) | Nothing by itself; hands the problem to an LTLf synthesis tool | Unrolled size grows with `L` times formula size | Trace length, formula size | Typed export refusal |

   *Complexity.* With no input signals the problem reduces to MLTL
   satisfiability. MLTL satisfiability checking is NEXPTIME-complete
   (interval bounds written in binary); only MLTL0, the fragment in which every
   interval starts at 0, is PSPACE-complete (Li, Vardi and Rozier, CAV 2019).
   Source status: cited from publisher and index records; the paper body was
   not read by this project, and the classes are not re-derived here. The
   statement above is a correction of an earlier draft that called MLTL
   satisfiability PSPACE-complete. Consequences stated for the specification and
   the documentation: requirement realizability over general MLTL is at least
   NEXPTIME-hard (the no-input case), no upper-bound class is claimed for the
   game, and no method may be advertised as polynomial or as scalable. The
   declared small population uses the intervals `[0,0]`, `[0,1]`, `[1,1]`,
   `[0,2]` and `[1,2]`, so it is not MLTL0 (it contains `[1,1]` and `[1,2]`),
   but its interval ends are at most 2 and its play length at most 4, so the
   class says nothing about it: every case there is a bounded enumeration of at
   most 16^4 plays. Agreement on the population is not evidence about scaling
   in interval size. The interval bounds are checked integers, so `L` can be
   large; limits are on `L`, not on formula text size.

9. **Relationship to protocol realizability.** Protocol realizability asks
   whether a choreography can be split into roles. It is tracked in QSL-1868,
   TL-387 and quire-specification FR-175, and its engine belongs to QSL. This
   project neither calls into nor reuses any of it, and it does not rename or
   absorb it. The two are linked here and in the requirements by reference
   only. They share no type, wire identity, command name or result label.
   Requirement realizability evidence is not protocol evidence and must not be
   presented as such.

## Consequences

- tl-mltl gains an exponential, bounded reference play-out as a test oracle
  (FR-043). It is the independent source of truth the tl-solve game and the
  re-check are compared against; it is not a production decision procedure.
- tl-syntax gains a new catalog document version. Consumers that never read a
  direction are unaffected and v1 documents are unchanged.
- tl-solve owns progression, search, certificates, the re-check and both
  exports, as FR-100 through FR-109 and NFR-100 through NFR-102 under ids that
  P-TL-147 reserves for this project.
- Strategy size is the dominant cost of a `Realizable` result, since the
  certificate is a table over reachable states and `2^|I|` input valuations.
  The certificate and the state limit are therefore reported together.
- Because the horizon is a derived upper bound and the end is fixed, every result is stated for
  one `L`. A result for a formula set is not a result for any longer run. That
  is a property of MLTL's bounded intervals and is stated in every result.
- Decision 5 leaves contract-bearing requests `Unsupported` until Peter picks
  a row of its open-decision table and records the ADR-017 option. That is a
  deliberate dependency, not a defect.
- The realizability checks need no strategy-judging code in tl-mltl production
  paths: FR-043 offers a callback-based judge (FR-043-AC-7) whose adapter lives
  in tl-solve test code (tl-solve FR-104-AC-6), because tl-mltl FR-113 forbids
  a tl-mltl to tl-solve dependency.

## Alternatives Considered

- **Let the system choose when the trace ends.** This is the semantics of
  LTLf synthesis. It lets a system satisfy a bounded `G` simply by ending
  early, which changes the meaning of a bounded MLTL requirement. It is
  rejected as the TL semantics and kept only as the difference an exporter must
  neutralise: the TLSF export carries a length guard so the external tool plays
  the fixed horizon (tl-solve FR-108).
- **Outputs first, then inputs.** The system would have to commit before
  seeing the step's input, which makes `output equals current input` unrealizable
  and is the Moore convention. Inputs-first matches the Mealy convention and
  the usual reading of a requirement such as "when the sensor is high the
  actuator shall be on in the same step".
- **A separate assumption syntax in TL-149.** This duplicates the contract form
  P-TL-150 already plans and forks the vocabulary for the same idea.
- **Direction on the proposition binding.** Two propositions bound to one
  signal could then disagree about who controls it. Direction on the signal
  removes that state.
- **Add an optional `direction` member to the v1 catalog schema.** v1 is a
  closed, byte-pinned contract. A new version keeps v1 fixtures and identities
  untouched.
- **Report the necessary check as a weaker verdict.** A passing necessary check
  could be shown as "probably realizable". That invites exactly the confusion
  the check cannot resolve, since the response may depend on a future input.
- **Trust the search and skip the re-check.** A strategy extracted from a
  game search is only as correct as the progression it uses. An independent
  satisfiability query costs one solver call and removes that dependence.
