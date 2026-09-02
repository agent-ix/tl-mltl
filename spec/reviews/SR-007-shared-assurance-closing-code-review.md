---
id: SR-007
title: Closing code review of the tl-mltl shared assurance migration
type: SpecReview
analysis: code-review
scope: assurance/**, scripts/**, examples/**, tests/**/*.rs, tests/fixtures/**, corpus/r2u2-v4.2/manifest.json, schemas/README.md, Makefile, .github/workflows/ci.yml
review_set: all
---

# Closing code review of the tl-mltl shared assurance migration

## Summary

An independent adversarial review was commissioned against `7fc9e3b` with one
instruction: **find false greens.** It ran in an isolated worktree, was free to
break anything, and reported what it actually executed rather than what it read.

It found **two high and four medium** false greens, plus seven low findings. One
of the highs was a defect in the fix for a defect SR-005 had already reported —
the producer-isolation probe still could not see an executed `quire coverage`.
That is the campaign's recurring shape, and it is the reason self-review is not
sufficient: SR-005 believed that finding closed and it was not.

The review also reproduced the `.IGNORE:` measurement exactly, verified the
schema-digest claims against the envelope bytes, and confirmed that four of the
eight claims could not be broken by what it tried. It listed what it did not
attack, which is what makes the rest of its report usable.

Combined with SR-005's six findings: **19 findings — 15 FIXED, 3 ACCEPTED,
1 DEFERRED.**

## Findings

| ID | Severity | Summary | Refs | Escape Cause |
|---|---|---|---|---|
| FND-701 | high | The producer-isolation probe still could not see an executed `quire coverage`. The review injected it into the driver with the output discarded and the test passed: no shim invocation, no recreated input file, no trace. SR-005's fix caught only a driver that *recreates a declared input*, and both the test comment and NFR-003-AC-2 claimed more than that. FIXED by enforcing the boundary inside the driver with an audit hook that refuses any child which is neither the pinned Quoin CLI nor a version observation; verified against the review's exact injection, exit 2. | scripts/assurance_chain.py, tests/shared_assurance.rs | correct-requirement-no-evidence |
| FND-702 | high | Nothing gated the `ci:` prerequisite list; dropping `test` and `assurance` left `make ci` at exit 0 with the whole test suite and the whole chain gone. Already FIXED at `a5c093d`, which the review did not see, and confirmed against its exact attack. | Makefile, tests/shared_assurance.rs | correct-requirement-no-evidence |
| FND-703 | medium | The twelve-state gate was label-driven: `states_demonstrated` is built from a free-text string typed next to each assertion, so deleting the only real `suspect` demonstration and relabelling an unrelated probe kept every gate green. FIXED by binding each state to the named case that owns it and requiring the two audit-derived states to carry the finding that produced them. | scripts/assurance_chain.py, tests/shared_assurance.rs | wrong-requirement |
| FND-704 | medium | `unsupported` was discharged by an unconditional literal. The R2U2 manifest's unsupported case carried no formula, so the producer printed the word by iterating a list, the count oracle was the length of that same list, and deleting the profile refusal from `map_to_c2po` changed nothing. FIXED: the case now carries a real out-of-profile formula that the adapter must refuse. | corpus/r2u2-v4.2/manifest.json, examples/r2u2_differential.rs | correct-requirement-no-evidence |
| FND-705 | medium | The frozen-schema census walked seven named directories and four named files while `schemas/README.md` claimed it walked the whole tree; `build.rs`, which runs on every cargo invocation, was outside it. FIXED: the census now walks the repository root recursively with exactly the four exclusions the README names. | tests/shared_assurance.rs, schemas/README.md | implementation-bug-despite-evidence |
| FND-706 | medium | Five of seven sealed attestations named a `tool.version` nobody observed; two Python script producers were attested as the Rust crate version. FIXED for the two Python producers by observing their interpreters; the three compiled examples keep the crate version and the docstring now says that is a stated fact rather than a probe. | scripts/assurance_chain.py | wrong-requirement |
| FND-707 | low | "The only target that runs a producer" is false as written — `conformance`, `differential`, `cli-conformance`, `test-census` and `spec` run producers too. FIXED: reworded to "the only target that writes the chain's inputs" everywhere it appeared. | Makefile, CLAUDE.md, assurance/README.md, spec/requirements/FR-006-shared-assurance-intake.md | wrong-requirement |
| FND-708 | low | "Nothing is scraped from a transcript" was unqualified while two producers decide outcomes from message substrings. FIXED: the claim now says nothing *the chain attests* is scraped, and names both producers and why. | CLAUDE.md | wrong-requirement |
| FND-709 | low | The "16 individual recipe errors" sub-figure in the `.IGNORE:` measurement did not reproduce; the review measured 17. My count came from a truncated grep. FIXED by dropping the sub-figure; the load-bearing figures reproduced exactly. | spec/requirements/NFR-003-qualification-integrity.md | implementation-bug-despite-evidence |
| FND-710 | low | The coverage assertion called 68 "matrix row count" without naming the population; the test matrix contributes 24 of the 68. FIXED: 35 acceptance criteria + 24 test-matrix rows + 9 suite-registry rows. | tests/shared_assurance.rs | wrong-requirement |
| FND-711 | low | Task-001 claimed `0.22.5` "appeared exactly once in the tree"; it now appears five times in prose. FIXED: restated as one *resolved pin*, which is a different thing from one occurrence. | spec/plans/PLAN-002-shared-assurance-migration/tasks/Task-001-inventory-and-pins.md | implementation-bug-despite-evidence |
| FND-712 | low | `cli_conformance` will replay a binary from an edited-but-unrebuilt working tree; the stamped revision is still HEAD. ACCEPTED — a modification-time comparison was tried and rejected, because Git rewrites source mtimes on checkout while Cargo correctly declines to rebuild on content. The protection is the build line Make runs first, and TC-024's graph check now asserts it is present. | examples/cli_conformance.rs, Makefile | correct-requirement-no-evidence |
| FND-713 | low | `upstream_pin_mismatches` checked `TL_SYNTAX_REVISION` but not the new `TL_SYNTAX_CORPUS_BASIS`. FIXED. | scripts/check_shared_pins.py | missing-requirement |

## The high finding, in full

SR-005 reported that injecting `quire coverage` into the driver went undetected,
and reported it fixed. It was not. The replacement — moving each declared input
aside and requiring the driver to refuse rather than recreate it — catches a
driver that *writes* an input it should have consumed. It does not catch a
driver that *runs a producer and throws the output away*, which is what the
review injected, and which leaves nothing behind at all.

The instrument was wrong in kind. PATH shims cannot work here, because
`quoin evidence record` legitimately runs `quire coverage` inside the store it
writes; and file-based detection only sees producers whose output lands
somewhere the test looks.

What works is asking the driver what it did. A CPython audit hook observes every
`subprocess.Popen` the process starts — including calls a future edit adds
without thinking — and refuses anything that is neither the pinned Quoin CLI nor
a version observation, naming the argv it refused. The list of children the
driver actually started is now in the report, so a test can assert on it rather
than on an absence.

Three properties this has that the PATH shim did not: it sees calls that produce
no file, it sees calls to tools nobody thought to shim, and it fails closed on
anything unfamiliar rather than only on a named list.

## Mutation probes

Thirty-seven probes across two runs. Each applies one named mutation, verified to
have changed the bytes before the gate runs. A probe that crashes is a broken
probe, not a detection: nothing here has an `except` clause that could turn a
traceback into a green tick.

| Probe | First | Current |
|---|---|---|
| P01 every attested result hardcoded to `passed` | **NOT DETECTED** (exit 0) | detected (exit 1) |
| P02 all three producer streams rewritten to fail | detected (exit 2) | detected |
| P03 reference-conformance stream emptied | detected (exit 2) | detected |
| P04 quire export replaced with `{}` | detected (exit 1) | detected |
| P05 msrv `build-finished` success flipped to false | detected (exit 1) | detected |
| P06 compatibility census `matched=false` | detected (exit 1) | detected |
| P07 test-census entry set to `unavailable` | detected (exit 1) | detected |
| P08 an outcome word the driver does not name | detected (exit 2) | detected |
| P09 tamper scenario made to change nothing | detected (exit 1) | detected |
| P10 a control names a scenario that does not exist | detected (exit 2) | detected |
| P11 unobservable tool version defaulted to `0.0.0` | **NOT DETECTED** (exit 0) | detected (exit 101) |
| P12 adapter outcome map collapses `fail` into `pass` | **NOT DETECTED** (exit 0) | detected (exit 1) |
| P13 comparison classification dropped from every row | detected (exit 101) | detected |
| P14 external monitor state dropped from every row | detected (exit 101) | detected |
| P15 pending and tool_error folded into unsupported | detected (exit 101) | detected |
| P16 declared-unsupported row removed | detected (exit 1) | detected |
| P17 mismatch control rows relabelled agreement | detected (exit 1) | detected |
| P18 `compare_external` always returns Agreement | detected (exit 101) | detected |
| P19 producer stops emitting the declared unsupported case | detected (exit 1) | detected |
| P20 malformed rows relabelled pass | detected (exit 1) | detected |
| P21 declared-cause check disabled in the producer | **NOT DETECTED** (exit 0) | detected (exit 1) |
| P22 a declared corpus fixture deleted | detected (exit 2) | detected |
| P23 declared-cause markers made non-discriminating | detected (exit 1) | detected |
| P24 CLI determinism check disabled | **NOT DETECTED** (exit 0) | detected (exit 1) |
| P25 a retained evidence byte modified | detected (exit 1) | detected |
| P26 a derived fixture hand-edited off its derivation | detected (exit 2) | detected |
| P27 mirror registry reference added to a scanned file | detected (exit 1) | detected |
| P28 compiled revision and corpus basis collapsed | detected (exit 1) | detected |
| P29 a consumed-artifact digest changed | detected (exit 1) | detected |
| P30 a requirement-tagged test marked `#[ignore]` | detected (exit 1) | detected |
| P31 a requirement-tagged test configured out | detected (exit 1) | detected |
| P32 a frozen schema edited | detected (exit 101) | detected |
| P33 a frozen schema referenced by a script | detected (exit 101) | detected |
| P34 deleted machinery restored under its old name | detected (exit 101) | detected |
| P36 CLI marker helper weakened to a constant | — | detected (exit 1) |
| P37 CLI stale-binary check forced to a wrong revision | — | detected (exit 1) |
| **P35 `.IGNORE:` with the crate broken — RESIDUAL** | not detected (exit 0) | **not detected (exit 0)** |

Five of the six first-run misses share one shape: a check that passes on a green
tree, so disabling it changes nothing observable. In each case the fix is to hand
the same function inputs that must produce a *different* answer, routed through
one implementation so weakening it breaks the half that has to come out false.

The review's four demonstrated attacks were each re-run after the fix and each
is now caught. P35 stays undetected by design; a residual that quietly closed
would be a claim this change should not be making.

## What the review could not break, and what it tried

Recorded because a claim nobody attacked should not read as a verified one.

- **Attested results are read from producer bytes.** Three attacks, all
  detected, including a real code defect introduced into `src/horizon.rs`. The
  sibling failure — a chain green while proofs declare `inconclusive`,
  `not_computed` or `unavailable` — is genuinely gated in both the chain and the
  test. With `rustup` stubbed to fail, the chain exits 2 rather than sealing
  `0.0.0`.
- **Nothing executes R2U2 or C2PO.** Every `Command::new`/`subprocess` in the
  tree enumerated; the driver run with `r2u2` and `c2po` shims on `PATH` left an
  empty log.
- **The differential is a comparison.** Three classifications and four external
  states genuinely produced through the real `compare_external`, and gated in
  both places. Only the `unsupported` count was a tautology, now fixed.
- **Evidence bytes.** 283 files, one changed and it is `evidence/README.md`.
  Appending one byte to `evidence/ANCHORS` → exit 1.
- **The schema-digest claims.** Verified against the envelope bytes: manifest
  `8744bfe2…` named by 6 of 6 and equal to the file; input schema named as
  `808fd9f3…` by 4 and `d763369e…` by 2, and the on-disk `7b7e4725…` by none.
- **The `.IGNORE:` measurement.** Reproduced exactly: exit 2 → exit 0, and
  precisely 10 of 14 prerequisites, with the same four completing either way.
- **All five compatibility mutation probes genuinely mutate**, and probe 4 is not
  vacuous: probe 3's audit reports no `suspect-link` and probe 4's reword
  produces one.

Explicitly **unchecked**, per the reviewer: hosted CI was not dispatched; Quoin,
Quire and engineering-assurance internals were not audited; the pin classifier's
incompatible and exit-2 paths were not exercised; `deny`, `audit-unsafe`,
`fmt-check`, `lint`, `rustdoc` and `check-corpus` were not attacked; the
`assurance-record` operator target was not run; the derived compatibility
fixtures were not individually broken; `quire validate` was not attacked; and the
`evidence/` file-count assertion was read rather than exercised.

## Verdict

**ACCEPT.** Every high and medium is fixed and each fix is verified against the
attack that found it. Three lows are accepted with rationale and one class is
deferred to `agent-ix/tl-mltl#14` with a reproduction. `make ci` exits 0 at the
reviewed head with 32 Rust tests and a clean tree.
