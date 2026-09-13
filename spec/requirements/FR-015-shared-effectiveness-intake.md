---
id: FR-015
title: "Route effectiveness records through shared assurance"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-003
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-006
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-009
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-011
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-012
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-013
    type: depends_on
  - target: ix://agent-ix/tl-mltl/FR-014
    type: depends_on
---

# FR-015: Route effectiveness records through shared assurance

## Description

When a property, fuzz, mutation, or bounded-proof result is presented for
review, the owning TL repository shall produce the domain record in Rust and use
Quire for static facts and Quoin for wrapper validation, binding, retention,
audit, and receipts without creating a repository-local replacement.

## Inputs

- Exact MRS-003 campaign identity and one FR-011 through FR-014 domain record.
- Source revision, requirement/statement hashes, test/harness/target identities,
  tool/dependency/configuration/environment identities, raw artifact digests,
  status, limitations, and owning repository.
- Pinned Engineering Assurance compatibility catalog and Quire/Quoin versions.

## Outputs

- A domain-produced structured result accepted or refused by the shared intake.
- Quoin-retained records and receipts that preserve every domain state and bind
  exact source/static-fact/artifact digests.
- Separate campaign observations and human review/release decisions.

## Behavior

Each TL repository owns only the Rust producer for its domain tests. Quire
exports requirement, property-shape, matrix, and distinct non-coverage
`implements` facts without
running a producer. The Rust producer validates its native domain record. Quoin
validates the MeasurementCollection wrapper and losslessly retains `rawEvidence`
without claiming to validate opaque domain semantics or
running property tests, fuzzers, mutation tools, Kani, external monitors, or
foreign runtimes. Engineering Assurance owns compatibility semantics. No TL
repository creates a generic envelope, collector, runner, audit/retention store,
receipt, approval service, or duplicate identity framework.

The production-symbol relation required by FR-013 is the Quire 0.31
`implements` export delivered after `agent-ix/quire-rs#171`; it is scope and
never backs an acceptance criterion. The current tl-mltl export reports six
production bindings for FR-001 through FR-005. Each mutation population must
first demonstrate complete applicable bindings at its exact revision. No
producer may infer or maintain a private substitute.

The current Quoin 0.23.1 cargo-mutants evidence adapter is insufficient as the sole
FR-013 intake because it emits a per-function score while discarding atomic
mutant identities, survivor dispositions, environment, and the complete raw
report. The FR-013 producer shall instead populate Quoin MeasurementCollection
v2 with the complete JSON domain record in `rawEvidence` and the exact
verification stack, or wait for an accepted shared adapter that preserves the
same facts. Quoin 0.23.1 exposes no demonstrated generic immutable
binary-attachment store. Fuzz/crash or proof artifacts that cannot be embedded
losslessly remain blocked until Quoin owns an accepted content-addressed
attachment contract (`agent-ix/quoin#363`); the TL repositories shall not
create a local archive as a substitute.

Quoin 0.23.1 resolves a MeasurementPlan locally and accepts a new collection
only for an exact `active` plan/definition. MP-003 through MP-006 in this
repository govern tl-mltl only and remain `proposed` during review. Before its
first collection, each sibling owner must install and independently accept its
own local active plan referencing MRS-003, or an accepted Quoin release must
support exact external-plan references. Promotion from proposed to active is a
reviewed successor change after all applicable admission gates land.

MeasurementCollection v2 also requires `verificationStack.buildProfile:
release`. An instrumented fuzz, mutation, or Kani build that is not truthfully a
release profile cannot be relabelled. Its shared intake remains blocked until
the exact released schema represents the actual build profile
(`agent-ix/quoin#364`) or the producer demonstrates that the executed artifact
really satisfies the release profile.

The authoritative shared vocabulary is
`engineering-assurance.verification-semantics-ownership/v1`; this specification
does not restate or narrow it. Domain states map without erasure:

| Domain observation | Shared result state |
|---|---|
| property pass / counterexample / vacuous / suspect / not run | `passed` / `failed` / `vacuous` / `suspect` / `not_computed` |
| fuzz budget complete / reproduced crash / timeout input / cancelled / tool error / unavailable target | `observed` / `failed` / `timed_out` / `skipped` / `error` / `unavailable` |
| fuzz instrumentation loss or irreproducible crash | `inconclusive` or `suspect`, with the domain state retained |
| mutation caught / missed / timeout / unviable / tool error / not run / cancelled | `passed` / `failed` / `timed_out` / `not_applicable` / `error` / `not_computed` / `skipped` |
| Kani proved / counterexample / unwind incomplete or solver unknown / vacuous / timeout / unsupported / tool error / not run | `passed` / `failed` / `inconclusive` / `vacuous` / `timed_out` / `unsupported` / `error` / `not_computed` |

`defined`, `observed`, `passed`, `failed`, `error`, `skipped`, `unavailable`,
`not_computed`, `not_applicable`, `inconclusive`, `unsupported`, `rejected`,
`timed_out`, `pending`, `malformed`, `stale`, `suspect`, `vacuous`, `tampered`,
and `unreadable` remain available exactly as the registry defines them. A shared
intake state never erases the property/fuzz/mutation/proof outcome. Missing,
empty, unreadable, stale, partially written, digest-mismatched, or
schema-incompatible producer bytes refuse and cannot be reconstructed from exit
status or console text.

Every referenced artifact and manifest obeys the exact `/`-only UTF-8 path,
digest, checked-`u64`, and inclusive resource maxima in
`tl-mltl.corpus-limits/v1` from FR-009 unless an accepted versioned shared
attachment contract declares tighter bounds. Backslash, absolute, empty,
dot/dotdot, duplicate-byte, symlinked, non-regular, untracked, escaping,
over-count, over-length, over-size, over-depth, unknown-limit, or
digest-mismatched inputs refuse before allocation or decode. Raw JSON may be
embedded in MeasurementCollection v2 only within those bounds. No out-of-tree
binary is claimed retained until the accepted shared attachment capability
exists.

Campaign summaries report exact applicable/excluded/blocked/not-run and every
domain outcome population before any ratio. They cannot hide unavailable or
non-conclusive work, substitute an older revision, aggregate different
instrumentation identities, or authorize release. Only the human authority in
the applicable assurance profile can make a release decision.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-015-AC-1 | Rust domain producers, Quire static export, Quoin MeasurementCollection v2 wrapper/intake/retention, and Engineering Assurance compatibility each perform only their allocated role, and process-spawn probes prove Quire/Quoin do not execute campaign or external producers. | Test (TC-071, TC-072) |
| FR-015-AC-2 | Every domain state maps to the exact shared ownership-registry state without erasure; absent, partially written, stale, suspect, vacuous, tampered, unreadable, or digest-mismatched producer bytes refuse and cannot be inferred from exit status, logs, counts, or an older result. | Test (TC-072, TC-073) |
| FR-015-AC-3 | Every source, requirement, test/harness/target, tool, dependency, configuration, environment, raw artifact, status, limitation, owner, and retention identity is bound by exact digest or typed identity before a result is credited. | Test (TC-067, TC-073) |
| FR-015-AC-4 | Every FR-009 path/digest/resource boundary and any tighter accepted shared bound refuses before bytes are decoded or allocated, and an out-of-tree binary artifact remains blocked unless an accepted Quoin capability binds immutable storage identity and content digest. | Test (TC-074) |
| FR-015-AC-5 | Every collection resolves an exact active plan local to its owning repository and truthfully records its build profile; a proposed, external-only, missing, version-mismatched, or falsely release-labelled plan/stack is refused. | Test (TC-075) |
| FR-015-AC-6 | No local generic evidence framework, foreign-runtime adapter, automated approval, or collapsed aggregate is introduced, and summaries retain every population before ratios without claiming release, qualification, certification, or monitor acceptance. | Test (TC-068, TC-071) |

## Dependencies

Depends on FR-006 shared-assurance allocation, FR-009 path/resource rules, and
all four domain record contracts. Complete reviewed local `implements`
bindings, atomic mutation raw
evidence, immutable binary attachment support,
truthful non-release build-profile representation, and local active plan lookup
must be demonstrated in the pinned Quoin contract or land in its owning project
before the corresponding implementation proceeds. Reusable missing shared
capabilities are not implemented as local stopgaps.
