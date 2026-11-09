---
id: FR-009
title: "Keep corpus fixture families single-owner and separate from generated inputs"
type: FR
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-002
    type: implements
  - target: ix://agent-ix/tl-mltl/FR-008
    type: depends_on
---

# FR-009: Keep corpus fixture families single-owner and separate from generated inputs

## Description

When a canonical or observed fixture is used by tl-mltl, the fixture shall have one
owning repository, a versioned corpus identity, and recorded provenance, and
generated inputs shall stay out of the canonical population until a reviewed
minimal reproducer with an independently confirmed expected result is committed.

## Inputs

- A fixture family and its manifest, read from its owner through the dependency.
- For a retained external observation: the tool and version, the input, and the
  raw output.
- For a generated input: the generator and seed that produced it.

## Outputs

- A replay that uses the owner's bytes unchanged, or a failure that names the
  fixture whose bytes, identity, or provenance do not match.

## Behavior

Each family has exactly one authoritative owner and consumers read the owner's
bytes rather than keeping a copy.

| Family | Owner | How tl-mltl reads it |
|---|---|---|
| shared formula, profile, schema, and proposition-map cases; future-operator, past-history, and boundary corpora | `tl-syntax` | `tl_syntax::CORPUS_DIR`, no retained copy |
| evaluation, prefix, horizon, history, CLI request, and mapping overlays | `tl-mltl` | this repository |
| retained R2U2 exchange (`r2u2-v4.2`), live R2U2 evidence (`r2u2-live-v1`), past C2PO mapping corpus (`past-c2po-v1`) | `tl-mltl` | this repository, replayed without running the tool |
| internal text parse and format pairs | `tl-parse` | not read by tl-mltl |
| equivalence pairs and counterexamples | `tl-rewrite` | not read by tl-mltl |

A canonical fixture record names its case and family, its formula and profile,
the trace or history, the expected result with the oracle or derivation that
produced it, and its corpus revision. A retained external observation also
names the tool and version or source revision, the configuration, the input, the
raw output, with the license of any vendored artifact. The bytes are kept in
version control, so a change to them is a reviewable change.

Canonical fixtures are append-only within one corpus identity (for example
`tl-syntax-corpus/v1`). Changing a fixture's bytes, expected result, oracle, or
profile requires a successor corpus identity, and a case cannot be removed to
improve a ratio. A consumer asserts the corpus identity it reads.

Generated property, sweep, fuzz, mutation, and model-checking inputs live
outside the canonical directories and are never counted as conformance fixtures.
A disagreement found by a generated population is minimised and committed as a
regression fixture whose expected result is confirmed by the independent oracle
or a hand derivation. Seed counts and campaign output alone are not conformance
evidence.

Evidence retention belongs to the repository's own files and the campaign report.
tl-mltl adds no generic evidence store, audit store, or approval mechanism.

## Acceptance Criteria

| ID | Criteria | Verification |
|---|---|---|
| FR-009-AC-1 | Every canonical fixture and retained observation carries its identity, expected or observed outcome, and provenance, and a record with a missing or unknown field is refused. | Test (TC-098, TC-099) |
| FR-009-AC-2 | Every family has exactly one authoritative owner, and tl-mltl replays the owner's bytes through the dependency without keeping or restamping a copy. | Test (TC-098) |
| FR-009-AC-3 | A changed expected result, or a removed case makes replay fail and names the fixture, and a changed corpus identity is detected. | Test (TC-099, TC-101) |
| FR-009-AC-4 | Generated inputs stay outside the canonical population, and a disagreement enters it only as a minimised reproducer with an independently confirmed expected result. | Test (TC-099) |
| FR-009-AC-5 | The retained shared and R2U2 corpus bytes, and identities are unchanged by this campaign. | Test (TC-099) |
| FR-009-AC-6 | Withdrawn: fixture-loader resource profile with checked arithmetic over a corpus manifest. | Withdrawn |

## Dependencies

Depends on FR-008 for the cell list. A family whose authoritative owner is an
agent-ix/Quire repository is out of this campaign's scope under ADR-002 and
belongs to `quire-mltl`'s own corpus.
