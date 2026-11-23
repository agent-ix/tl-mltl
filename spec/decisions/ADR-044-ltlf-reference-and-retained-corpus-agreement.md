---
id: ADR-044
title: "Check LTLf agreement against an outside reference by replaying a retained corpus"
type: ADR
status: proposed
owner: tl-mltl-maintainer
relationships:
  - target: ix://agent-ix/tl-mltl/FR-083
    type: realizes_requirement
  - target: ix://agent-ix/tl-mltl/FR-084
    type: realizes_requirement
  - target: ix://agent-ix/tl-syntax/ADR-044
    type: depends_on
---

# ADR-044: Check LTLf agreement against an outside reference by replaying a retained corpus

## Status

Proposed (TL-427). Ready for human acceptance; not accepted. The tool
identities below are recommendations that no one has run or version-checked.

## Context

TL-428 requires agreement with an outside LTLf reference "chosen in the spec",
including the known finite-versus-infinite counterexample. An evaluator checked
only against its own author's reading of the end-of-trace rules proves nothing
about the rules. No outside tool may be executed during specification, and tool
output regenerated on every CI run would make the gate depend on a foreign
toolchain.

## Decision

1. **Reference.** The primary outside reference is the LTLf-to-DFA route
   (LTLf2DFA, which delegates to MONA) evaluated on the trace, as the tooling
   of the authors of the cited LTLf literature. The secondary reference is
   BLACK in its finite-trace mode (the same tool TL-435 exports to). Agreement
   is required with the primary; the secondary is used to arbitrate and is
   recorded when it disagrees. Both are chosen for being independent of
   tl-mltl, openly licensed, and used by the LTLf community; neither
   identity nor version is verified here. If the accepting human prefers another
   reference, only this ADR and the corpus provenance block change.
2. **Retained-corpus replay.** The reference is run once, by a documented
   regeneration procedure, and its verdicts are retained as corpus
   `tl-mltl.ltlf-reference-corpus/v1` with the tool name, exact version, command
   line, input digest and operator. CI replays the retained verdicts against the
   evaluator and never invokes the tool. Fresh generation is **DEFERRED**
   (not executable now; recorded in the plan's deferred-verification table).
3. **Counterexample.** The two derived separators CE-1 `G Xs true` and CE-2
   `F Xw false` of tl-syntax FR-335 are mandatory corpus members, with their
   LTLf verdicts from the reference and their infinite-trace verdicts from the
   existing lasso evaluator. Citing the formula the 2013/2014 papers
   themselves use for the same point is an open task for the accepting human;
   the spec does not assert which formula those papers print.
4. **Independent in-tree reference.** A test-only evaluator written directly
   from the ADR-044 (tl-syntax) definition table, sharing no code with the
   production evaluator, is compared on the same population. It never
   substitutes for the outside reference.

## Alternatives rejected

- Regenerate with the tool in CI: gate depends on a foreign toolchain.
- Self-agreement only: circular.
- Hand-computed verdicts for the whole population: not independent of the author.

## Consequences

Until the corpus is generated, the agreement criteria stay unmet and are shown
as such in TM-047. Provenance fields make a stale corpus detectable.
