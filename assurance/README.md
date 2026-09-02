# Shared assurance

This directory holds what tl-mltl *declares*. It holds no evidence, no manifest,
no verdict, and no store.

| File | What it is |
|---|---|
| `pins.json` | The Engineering Assurance release this repository adopts, and the artifacts it reads from that release. Component versions are deliberately not restated: the packaged compatibility matrix is their authority. |
| `change-assurance.json` | The author's statement about the change under issue #16, in the shape Quoin's FR-063 record requires. |

## How the pieces relate

```
make assurance-inputs        the ONLY target that runs a producer
   |
   +-> target/assurance/*    structured results, written by domain tools
          |
          +-> scripts/assurance_chain.py        reads those bytes, seals through quoin
          +-> scripts/check_shared_pins.py      classifies the toolchain through the matrix
```

Three rules make this different from what it replaced.

**The driver never produces, and enforces that on itself.** If an input is
absent, the chain says so and names `make assurance-inputs`. It does not run the
producer itself — and it does not merely intend not to: an audit hook refuses any
child process other than the pinned Quoin CLI and a version observation, so a
future edit that reaches for `quire coverage` or `cargo run` is refused at exit
2 naming the argv. An adversarial review found that a PATH shim could not
establish this, because `quoin evidence record` legitimately runs `quire
coverage` itself and a driver that ran a producer and discarded the output left
no trace at all.

Note that `make assurance-inputs` is the only target that writes *the chain's
inputs*, not the only target that runs a producer. `conformance`,
`differential`, `cli-conformance`, `test-census` and `spec` run the same
producers as ordinary gates.

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

## What is not here

No evidence envelope, manifest, anchor verifier, retention store, audit store,
tool lock, failure-propagation policer, or aggregate verdict — and, since issue
#16, no retained legacy evidence and no reader for it. The repository owner
released the preservation constraint for the pre-stable phase on 2026-09-02
(`agent-ix/engineering-assurance#7`), so `evidence/`, `schemas/`, the
compatibility view and the `PROOF-legacy-compatibility` obligation were deleted
rather than carried. Nothing was rewritten to look like it still verifies; the
records and every claim that argued from them are simply gone. The constraint
re-applies unchanged at the move toward stable releases.

`Makefile` is orchestration and is not a trust root. What that costs is measured
and recorded in `change-assurance.json` under
`UNKNOWN-make-failure-propagation-guard-removed`, and tracked as
`agent-ix/tl-mltl#14`.
