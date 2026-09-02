# Shared assurance

This directory holds what tl-mltl *declares*. It holds no evidence, no manifest,
no verdict, and no store.

| File | What it is |
|---|---|
| `pins.json` | The Engineering Assurance release this repository adopts, and the digests of the artifacts it reads from that release. Component versions are deliberately not restated: the packaged compatibility matrix is their authority. |
| `change-assurance.json` | The author's statement about the change under issue #13, in the shape Quoin's FR-063 record requires. |

## How the pieces relate

```
make assurance-inputs        the ONLY target that runs a producer
   |
   +-> target/assurance/*    structured results, written by domain tools
          |
          +-> scripts/assurance_chain.py        reads those bytes, seals through quoin
          +-> scripts/legacy_evidence_view.py   reads evidence/ through the pinned mapping
          +-> scripts/check_shared_pins.py      classifies the toolchain through the matrix
```

Four rules make this different from what it replaced.

**The driver never produces.** If an input is absent, the chain says so and
names `make assurance-inputs`. It does not run the producer itself. A driver
that can produce its own inputs can produce a green run out of nothing.

**Every attested result is read from producer bytes.** `derive_result()` reads a
field the producer wrote — row outcomes, `matched`, or cargo's own
`build-finished` message. Nothing is inferred from an exit code alone, and
nothing is scraped from a transcript.

**Nothing here executes R2U2.** The external monitor and its C2PO compiler were
run once, out of band, at R2U2 `4.2-release` / C2PO `4.1.0`. Their exact
exchanged artifacts — spec binary, signal map, trace, stdout, and both
executable digests — are checked in under `corpus/r2u2-v4.2/` and verified by
`make check-corpus`. Every later claim is a replay against those retained bytes.
A gate that re-ran the external monitor would be producing the thing it is
supposed to be checking, and the differential would stop being a comparison
between two independent implementations.

**A refusal is a result.** This repository's six retained envelopes are
`quire.derivation-evidence/v1`, which the pinned PGM-01 mapping does not cover,
so it answers `incompatible` for every one of them. That answer was measured here
rather than inherited from a sibling — the same question has five different
answers across this campaign — and it is reported as it stands. It is not a
pass, it is not a defect of those records, and it is not a reason to write a
local mapper, which is precisely what this migration removed. Filed upstream as
`agent-ix/engineering-assurance#21`.

## What is not here

No evidence envelope, manifest, anchor verifier, retention store, audit store,
tool lock, failure-propagation policer, or aggregate verdict. Retained bytes
under `evidence/` are immutable, and Git history plus pull-request review are the
integrity boundary for them. `evidence/ANCHORS` and `evidence/RETRACTIONS.json`
survive as retained bytes; nothing interprets them any more.

`Makefile` is orchestration and is not a trust root. What that costs is measured
and recorded in `change-assurance.json` under
`UNKNOWN-make-failure-propagation-guard-removed`, and tracked as
`agent-ix/tl-mltl#14`.
