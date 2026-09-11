---
id: NFR-003
title: Make qualification controls explicit and fail closed
type: NFR
quality_attribute: reliability
---

# NFR-003: Make qualification controls explicit and fail closed

## Statement

Candidate qualification shall keep the producer boundary observable, derive every
attested result from the bytes a producer wrote, keep the twelve verification
outcomes distinguishable, keep the external-monitor comparison from collapsing
into a boolean, and grant no release authority.

## Scope

This requirement owns the shared-assurance intake path: the pinned toolchain
declaration in `assurance/pins.json`, the change-assurance declaration in
`assurance/change-assurance.json`, the driver `scripts/assurance_chain.py`, the
pin classifier `scripts/check_shared_pins.py`, the three domain producers under
`examples/`, the manual-only hosted workflow in `.github/workflows/ci.yml`, and
the tests that exercise them.

It no longer owns `tools.lock`, a host-scoped executable census, Make
execution-control probes, a collector, a finalizer, an envelope builder, a
manifest verifier, an anchor verifier, or a retraction registry. Those were
removed with the local evidence framework they belonged to.

It no longer owns a legacy-evidence compatibility view either. The repository
owner released the preservation constraint for the pre-stable phase on
2026-09-02 (`agent-ix/engineering-assurance#7`), and the retained records, the
reader, its fixtures and the two frozen schemas were deleted under
`agent-ix/tl-mltl#16`. Nothing was rewritten to look like it still verifies. The
constraint re-applies unchanged at the move toward stable releases, and evidence
retained under it from that point is immutable.

## What removing the Make guard actually costs, measured here

That is a real reduction in local detection, and its extent is stated rather
than minimised. It was measured in this repository, not inherited from a sibling.

With a syntax error introduced into `src/lib.rs`, `make -k ci` exits 2 and 10
of the 14 `ci` prerequisites do not complete: `fmt-check`, `lint`, `test`,
`conformance`, `differential`, `cli-conformance`, `test-census`, `msrv`,
`rustdoc` and `assurance`. Four still complete: `check-corpus`, `deny`,
`audit-unsafe` and `spec`. Adding a single `.IGNORE:` line to the `Makefile`
makes all 10 report success and `make ci` exits 0, with every individual recipe
error in the run log ignored — including `assurance-chain` exiting 2, which is
the chain correctly refusing empty producer output. Nothing in this repository inspects Make's own execution
controls to notice, because the parse-time guard block and
`scripts/check_failure_propagation.py` that used to do so were removed with the
collector they were protecting.

A structural backstop exists but covers only part of the gate set. Quoin binds
each retained input by digest and every attested result is derived from the
producer's own bytes, so a *producer* that did not run yields an absent or empty
input the chain names and refuses. That protects the six proofs whose work is
re-run inside `make assurance-inputs`. It does **not** protect a gate whose
recipe writes nothing the chain reads: `fmt-check`, `lint`, `check-corpus`,
`deny`, `audit-unsafe`, `rustdoc`, and the `quire validate` half of `spec` can
each be neutered and every remaining check stays green. And `.IGNORE:` at the
top of the file suppresses the chain's own refusal as well, so the structural
backstop does not close the class either — it narrows it.

The residue is recorded as an open unknown in the change-assurance declaration
and tracked as `agent-ix/tl-mltl#14`, which carries the reproduction. It is not
claimed to be closed by the structural replacement.

## Measurement and Evaluation

| Metric | Target | Threshold | Method |
|---|---|---|---|
| Components classified by the packaged matrix | 4/4 | 4/4 | Test |
| Verification outcomes demonstrated and matched | 12/12 | 12/12 | Test |
| Differential comparison classifications observed | 3/3 | 3/3 | Test |
| External-monitor states observed and kept separate | 4/4 | 4/4 | Test |
| Negatives without an accepted positive control | 0 | 0 | Test |
| Attested results not derived from producer bytes | 0 | 0 | Test |
| Gates that execute the external monitor | 0 | 0 | Test |
| Child processes the driver starts that are neither Quoin nor a version observation | 0 | 0 | Test |
| Executable hosted-workflow tokens naming an ix-flow package other than `@agent-ix/ix-flow@0.0.4` after YAML comments are removed | 0 | 0 | Test |
| Automatic hosted-workflow triggers | 0 | 0 | Test |
| Automatic release decisions | 0 | 0 | Inspection |

## Verification

Behaviour tests invoke the gates rather than reimplementing them. The producer
boundary is asserted with two runs — producers replaced by logging stubs with the
log required to be empty, and a control that stubs the tool the chain does use
and requires the chain to fail — because an empty log and an unconsulted `PATH`
are otherwise the same observation. Mutation probes remove one load-bearing
check at a time and require the corresponding gate to go red. Tests that read or
temporarily mutate shared assurance inputs serialize through a private guard
token whose only constructor locks the process-global input mutex. Every helper
that reads or temporarily mutates those inputs requires that token, including
the shared-pin reader and the requirements-file mutation probe; a poisoned guard
is a distinct failure because a panicking predecessor may have left those inputs
mutated.

The hosted-workflow control parses the tracked YAML bytes, selects only semantic
`jobs.*.steps[*].run` string scalars, requires the semantic trigger set to be
exactly [`workflow_dispatch`], and inspects command-position bare or
path-qualified npm executables in shell groups, direct commands, and statically
literal nested `sh`/`bash -c` scripts for ix-flow package arguments consumed by
every documented npm-install alias. Npm- or shell-shaped data arguments are not
commands. YAML metadata, including `defaults.run`, and
comments never enter the executable population. Within a run script, `#` begins
a shell comment only before a word starts; quoted and word-internal hashes remain
executable argument content. It also
observes `ix-flow --version` from the local gate environment so a correctly
spelled package token cannot silently run a different executable version.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| NFR-003-AC-1 | Every attested proof result is derived from the producer's own structured output; a producer whose output is absent, empty, or unreadable is an error naming the target that writes it, and never a pass. Every test that reads or temporarily mutates shared assurance inputs holds the repository's private serialization guard for the full access. | Test (TC-018, TC-019, TC-024) |
| NFR-003-AC-2 | Neither Quire nor Quoin executes a producer, and no gate executes R2U2 or C2PO. Demonstrated four ways, because no single one is sufficient: every producer on `PATH` replaced by a logging stub with the log required to be empty; a control that stubs Quoin and requires the chain to fail; every declared input moved aside in turn with the driver required to refuse rather than recreate it; and an audit hook inside the driver that refuses any child process which is neither the pinned Quoin CLI nor a version observation, exercised by injecting `quire coverage` into a copy of the driver. The injected-child refusal is paired with the unmodified driver succeeding in the same owned scratch, whose Quoin store is proved outside the repository store. A PATH shim alone cannot establish this, because Quoin legitimately runs `quire coverage` itself. | Test (TC-019) |
| NFR-003-AC-3 | The twelve verification outcomes stay distinguishable, each demonstrated by a case that produced it and matched, with every negative paired with a positive control and a control naming a non-existent scenario refused. | Test (TC-022) |
| NFR-003-AC-5 | Across semantic `jobs.*.steps[*].run` string scalars, independent of YAML spelling or style, a bare or path-qualified npm executable at command position after leading assignments or redirections, directly or through shell groups and a statically literal `sh`/`bash -c` script, using every documented npm-install alias (`install`, `add`, `i`, `in`, `ins`, `inst`, `insta`, `instal`, `isnt`, `isnta`, `isntal`, `isntall`) consumes exactly one executable ix-flow package, `@agent-ix/ix-flow@0.0.4`, and no other scoped, unscoped, alias-form, or identity-bearing alternate; npm- or shell-shaped data arguments and `sh`/`bash` invocations without `-c` are not nested commands and do not suppress later commands in the same scalar, YAML metadata and YAML/shell comments do not change that population, a `#` after any started word remains argument content, the semantic trigger set is exactly [`workflow_dispatch`], and the local qualification gate observes `ix-flow --version` as exactly `0.0.4`. | Test (TC-036) |

## Qualification Boundary

These controls make a presented candidate and its retained artifacts
reproducible and reviewable. They confer no qualification, certification, or
accreditation, and no external-monitor endorsement: the retained R2U2 exchange
is evidence of agreement at one external revision, not a qualification of R2U2
and not a qualification of this crate as a monitor. Branch protection and the
remote review history, not the local repository, establish resistance to history
replacement.
