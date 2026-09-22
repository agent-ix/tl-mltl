---
id: NFR-006
title: Bind the declared and executed CI gate set
type: NFR
quality_attribute: reliability
relationships:
  - target: ix://agent-ix/tl-mltl/FR-006
    type: depends_on
  - target: ix://agent-ix/tl-mltl/NFR-003
    type: extends
---

# NFR-006: Bind the declared and executed CI gate set

## Statement

If any prerequisite declared on the `ci` target does not both run its own
recipe to completion and report success from that recipe's own outcome, then
the CI entry point shall report `ci` as failed, naming every such
prerequisite. While invoking Make, the CI entry point shall refuse to proceed
if the Makefile text or the invocation environment it is about to hand to
Make carries an execution-control state capable of suppressing
prerequisite-failure propagation.

The CI entry point is a dedicated program distinct from and outside the Make
process it drives, invoked in place of a bare `make ci` by every local and
hosted caller.

## Scope

This requirement owns a single artifact: a CI entry point that is not itself
a Make recipe, invoked in place of a bare `make ci` by every local and hosted
caller. It owns that entry point's three checks, and the tests that exercise
each:

1. **Static inspection** of the Makefile text before delegating to Make, for
   every execution-control surface `NFR-003`'s "What removing the Make guard
   actually costs, measured here" section and the retired parse-time guard
   named: `SHELL`, `.SHELLFLAGS`, `MAKEFLAGS`, `.ONESHELL:`, `.DEFAULT:`,
   `.IGNORE:`, `.SILENT:`, a `-`-prefixed recipe line, `$(eval …)`, and
   `include` — together with three recipe-content patterns the original
   guard did not police: a recipe line containing `|| true`, one redirecting
   stderr to `/dev/null`, and one that joins a check and any later shell
   command with a bare `;` (Make's per-recipe-line exit status is the *last*
   shell command's, so `false; ci_guard record gate-a` reports success
   because `ci_guard record` always exits 0 — this is not a hypothetical:
   it defeated an early version of the identical mechanism in sibling
   repositories tl-rewrite and tl-parse before their fix, and this
   requirement bakes the fix in from the start rather than discovering it
   later). The bare-`;` check must not flag a `;;` case-statement terminator
   or a `;` that precedes `do`, `done`, `then`, `else`, `elif`, `fi`, or
   `esac` — ordinary shell control flow inside a recipe, not a
   failure-swallowing separator. An `include`d file is scanned by the same
   check rather than exempted; `$(eval` is refused outright rather than
   partially analyzed.
2. **Invocation-environment control**: the entry point does not forward an
   inherited `MAKEFLAGS` to the Make process it starts, and invokes Make with
   an explicit, minimal flag set rather than trusting the caller's shell
   environment — closing the vector where `-i`/`-k`/`-S`-equivalent behavior
   is requested through the environment rather than through Makefile text,
   which static inspection alone cannot see. Recognizing an `-i`/
   `--ignore-errors`-equivalent `MAKEFLAGS` value must account for GNU
   Make's own flag-bundling (a single re-exported token such as `-ik` or
   `ki` carries `-i`, not just an exact bare `i` word), and an `|| true`
   detector must match wherever it appears on the recipe line — including
   before a trailing `;` and further commands (`cmd || true; more`) — rather
   than only at end-of-line or a whitespace boundary. Both narrow forms were
   found, after initial shipment, to let real bypasses through in sibling
   repositories; this requirement specifies the wider match from the start.
3. **Declared/executed reconciliation** after Make returns: each `ci`
   prerequisite's recipe writes a per-gate completion record naming itself
   and its own recipe's exit status, written only on that recipe's own
   successful completion; the entry point compares the resulting record set
   against the exact declared `ci` prerequisite set — parsed from the
   Makefile's own `ci:` rule, currently 15 prerequisites (`fmt-check`,
   `lint`, `kani-check`, `test`, `check-corpus`, `conformance`,
   `differential`, `cli-conformance`, `test-census`, `deny`, `audit-unsafe`,
   `spec`, `msrv`, `rustdoc`, `assurance`) — and reports a violation naming
   any mismatch in either direction, rather than trusting Make's own exit
   code.

It does not own the correctness of any individual gate's recipe (formatting,
lint, the MLTL domain replays, and the rest each remain owned by the
requirement that specifies that gate's behavior), and it does not own
producer-input integrity, which `NFR-003` and the Quoin-bound
`assurance-inputs` chain already establish by deriving attested results from
a producer's own written bytes. Those two mechanisms are complementary rather
than overlapping: the digest chain notices a producer that never ran by
finding its output absent, and this requirement depends on that chain
remaining intact so it need not duplicate coverage of the gates re-run inside
`assurance-inputs` (`conformance`, `differential`, `cli-conformance`,
`test-census`, `spec`'s `quire coverage` half, and `msrv`). It is blind,
however, to a gate — `fmt-check`, `lint`, `check-corpus`, `deny`,
`audit-unsafe`, `rustdoc`, and the `quire validate` half of `spec` — whose
recipe writes nothing the chain reads, and whose failure Make's own
execution controls can suppress before it is ever observed. This requirement
closes exactly that residual, matching `NFR-003`'s own disclosure that the
structural backstop narrows the class rather than closing it.

The binding this requirement establishes holds only for invocations that go
through the entry point. `make ci` remains directly invocable as a Makefile
target for local convenience; it is not itself the assured gate once this
requirement is implemented, and nothing mechanical stops a person from typing
it directly and asserting a result without the entry point's report. This
requirement closes that gap for every *documented and hosted* path rather
than for every possible human action: every reference to running the full
local gate set — this repository's README, `CLAUDE.md`, and the manual-only
hosted workflow dispatch — names the entry point, not a bare `make ci`, so
the ordinary and automated paths cannot bypass the binding by using the
pre-existing alias. A person who deliberately runs `make ci` directly and
reports its exit code without the entry point is outside what an
in-repository control can prevent; that residual is disclosed here rather
than claimed away, matching this repository's existing convention (see
`NFR-003`) of stating what a control does not reach.

The mechanism is repository-local and domain-specific to this Makefile and
this `ci` target; it is not a proposal for a shared, cross-repository
control. `agent-ix/engineering-assurance#11` was investigated as a possible
shared home for this exact class (Linear TL-65) and found to be unrelated —
it defines producer/runner qualification campaigns, not gate-set binding —
and no matching shared control exists upstream under any tracked issue. A
future generalization of this guarantee into Quoin or Engineering Assurance
remains a separate, later decision and is out of scope here; sibling
repository tl-rewrite (`NFR-004-gate-set-integrity`, TL-64) and tl-parse
(TL-66) each independently closed the identical local gap the same way,
which is the precedent this requirement follows rather than waiting further
on a shared control that does not yet exist.

## Rationale

Make's failure-propagation behavior is not an invariant of the tool; it is a
default that a single misplaced directive, recipe prefix, dynamically
evaluated fragment, included file, or inherited flag overrides for the entire
run, silently and without changing which targets appear to exist. This is
not hypothetical for this repository: with a syntax error introduced into
`src/lib.rs`, `make -k ci` exits 2 and 12 of 15 `ci` prerequisites do not
complete, and a single `.IGNORE:` line added to the Makefile makes all 12
report success and `make ci` exit 0 — including `assurance-chain` correctly
refusing empty producer output, its own refusal suppressed along with
everything else. A reviewer reading `ci:`'s prerequisite list, or a person
trusting a green `make ci`, is trusting that nothing in the file or its
invocation environment has done that — a property Make itself has no
mechanism to assert about its own execution, and a property a check that
only reads Makefile text cannot fully assert either, since some of these
overrides arrive through the environment rather than the file. Binding what
was declared to what actually executed, from a vantage point outside Make
and with the entry point controlling both the text it delegates to and the
environment it delegates with, converts that trust into a checked property:
the entry point either observes 15 genuine passes or it does not report a
pass at all, regardless of which mechanism — present or future, textual,
dynamic, or environmental — caused a gate not to run its own work.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Execution-control surfaces recognized as failure-propagation risks (`SHELL`, `.SHELLFLAGS`, `MAKEFLAGS`, `.ONESHELL`, `.DEFAULT`, `.IGNORE`, `.SILENT`, `-`-prefixed recipe line, `$(eval …)`, `include`, `\|\| true`, stderr-to-null redirect, bare `;` command separator) | 13/13 | 13/13 | Test |
| `ci` invocations through the entry point where the declared prerequisite set and the executed-completion-record set are reconciled before a result is reported | 100% | 100% | Test |
| Induced-failure classes from the tracked reproduction (`.IGNORE:` prepended; every recipe replaced by a failing stub; a bare-`;` chained check-and-record recipe line; a backdated or removed completion record) rejected by the entry point | 4/4 | 4/4 | Test |
| False rejections of an unmodified, passing Makefile, ordinary shell control-flow semicolons in a recipe, and a clean invocation environment | 0 | 0 | Test |
| Documented or hosted invocation paths for the full local gate set that still name a bare `make ci` instead of the entry point | 0 | 0 | Inspection |

## Verification

A positive-control run against the unmodified Makefile, a clean invocation
environment, and every gate passing demonstrates the entry point reports
success and does not over-trigger, including on a recipe that uses ordinary
`for`/`case` shell control flow with semicolons. Negative controls reproduce
the tracked measurement directly: a Makefile copy with a single `.IGNORE:`
line prepended; a skeleton Makefile whose recipes are all replaced by a
failing command; a recipe line that joins a failing check and the
completion-record call with a bare `;` instead of running the record only on
success; and, separately, a completion-record set that is removed or
backdated for one gate while that gate remains declared as a `ci`
prerequisite. Each must be rejected by the entry point even though a bare
`make ci` on the same file would exit non-zero for the wrong reason, exit
zero with the directive present, exit zero because the last command on the
chained line always succeeds, or exit zero on a stale record the raw exit
code never inspects. A further control sets `MAKEFLAGS` in the calling
environment to a value equivalent to `-i`/`-k`, including a bundled form such
as `ik`, before invoking the entry point, without touching the Makefile
text, and must also be rejected — this demonstrates the environment-control
check independently of the static-text check, so a failure in one is
attributable without the other masking it. An inspection pass confirms the
README, `CLAUDE.md`, and the hosted workflow dispatch reference the entry
point rather than a bare `make ci`.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-006-AC-1 | The entry point parses the Makefile text, including any `include`d file, before invoking Make, and refuses to proceed if `SHELL`, `.SHELLFLAGS`, or `MAKEFLAGS` is assigned, if `.ONESHELL:`, `.DEFAULT:`, `.IGNORE:`, or `.SILENT:` appears as a special target, if any recipe line under a `ci` prerequisite is `-`-prefixed or contains `\|\| true`, a stderr-to-`/dev/null` redirect, or a bare `;` command separator (excluding `;;` and a `;` immediately before `do`/`done`/`then`/`else`/`elif`/`fi`/`esac`), or if `$(eval` appears anywhere in the text. | Test |
| NFR-006-AC-2 | The entry point does not forward an inherited `MAKEFLAGS` to the Make process it starts and invokes Make with an explicit, minimal flag set; an invocation whose calling environment sets `MAKEFLAGS` to an `-i`/`-k`/`-S`-equivalent value, including a bundled short-flag token (e.g. `ik`), is refused before Make runs. | Test |
| NFR-006-AC-3 | Each of the 15 declared `ci` prerequisites' recipes writes a per-gate completion record naming the gate and its own recipe's exit status only on that recipe's own successful completion. | Test |
| NFR-006-AC-4 | After Make returns, the entry point reconciles the set of completion records against the exact declared `ci` prerequisite set parsed from the Makefile and reports a violation naming every declared gate without a record and every record without a declared gate, rather than trusting Make's own exit code. | Test |
| NFR-006-AC-5 | A completion record that is missing, removed after being written, or backdated to a run that precedes the run under evaluation is treated as no record for that gate, and the run is reported as a violation rather than a pass. | Test |
| NFR-006-AC-6 | Reproducing the tracked measurement — a `.IGNORE:`-prepended Makefile copy, a skeleton Makefile with every recipe replaced by a failing stub, and a recipe line joining a failing check and the completion-record call with a bare `;` — against the entry point yields a non-zero exit and a named violation, not a reported pass, for each. | Test |
| NFR-006-AC-7 | An unmodified Makefile, a clean invocation environment, and every gate genuinely passing — including a recipe using ordinary shell `for`/`case` control-flow semicolons — yields a zero exit from the entry point with no violation reported. | Test |
| NFR-006-AC-8 | The repository's README, `CLAUDE.md`, and the hosted workflow dispatch that runs the full local gate set invoke the entry point rather than a bare `make ci`. | Inspection |
