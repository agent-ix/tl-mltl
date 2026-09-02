# tl-mltl

Finite-trace MLTL evaluation, horizon analysis, and runtime-monitor
interoperability.

## Commands

```bash
make fmt              # format with rustfmt
make fmt-check        # verify formatting (CI gate)
make lint             # clippy with -D warnings
make test             # cargo test plus the shared-assurance tests
make check-corpus     # verify shared and R2U2 corpus checksums
make conformance      # replay the shared corpus through the evaluator
make differential     # replay the retained R2U2 exchange
make cli-conformance  # drive the built CLI over its declared requests
make test-census      # bind requirement-tagged tests to compiled tests
make deny             # cargo deny check licenses and sources
make audit-unsafe     # check that every unsafe block has a // SAFETY: comment
make spec             # validate specs and strict coverage
make msrv             # check all targets and features with Rust 1.75
make rustdoc          # build warning-free public docs
make assurance-env    # create the pinned shared-assurance interpreter
make assurance-inputs # run the producers and write their structured results
make assurance        # pins + compat-view + assurance-chain
make ci               # complete local gate
```

GitHub Actions is intentionally `workflow_dispatch`-only. Use local `make ci`
while iterating and dispatch hosted CI only for a finalized revision.

## Shared assurance

This repository is on the released Engineering Assurance / Quire / Quoin /
ix-flow contracts. It owns its domain producers and owns no evidence framework.

| Component | Version |
|---|---|
| quire-cli | 0.31.0 (engine 0.46.0) |
| quoin | 0.23.1 |
| ix-flow | 0.0.4 |
| engineering-assurance | 0.2.0 (git tag) |

Three rules to keep in mind before changing anything under `assurance/`,
`scripts/` or `examples/`:

- **`make assurance-inputs` is the only target that runs a producer.** Everything
  downstream consumes those files and refuses to create them. A driver that can
  produce its own inputs can produce a green run out of nothing.
- **Every attested result is read from the bytes a producer wrote.** Nothing is
  inferred from an exit code alone, and nothing is scraped from a transcript.
- **Nothing executes R2U2 or C2PO.** The external exchange under
  `corpus/r2u2-v4.2/` is retained and pinned by digest; every claim is a replay
  against those bytes.

`.venv-assurance/` is built by `make assurance-env` from
`requirements-assurance.txt` and is ignored. The shared-assurance lane runs there
rather than in the system interpreter, because `engineering-assurance` is pinned
as a git tag.

## The Makefile is not a trust root

Adding `.IGNORE:` to the `Makefile` makes recipes report success without running,
and nothing here notices. Measured in this repository: with a syntax error in
`src/lib.rs`, 10 of the 14 `ci` prerequisites fail and `make ci` exits 2; with
`.IGNORE:` added, all 10 report success and `make ci` exits 0. The structural
backstop — Quoin binding each retained input by digest — covers only the seven
producers that feed the chain. Tracked as `agent-ix/tl-mltl#14`.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs locally via `make audit-unsafe`. Every
  `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines,
  or be listed in `scripts/unsafe_comment_baseline.txt`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping
- `rust-toolchain.toml` pins to stable + rustfmt + clippy

## Layout

```
src/lib.rs             # crate root
src/evaluate.rs        # bounded closed/prefix reference evaluation
src/horizon.rs         # checked structural lookahead
src/mapping.rs         # deterministic C2PO mapping manifest
src/differential.rs    # external-verdict comparison, never a boolean
src/main.rs            # JSON command CLI
examples/              # the three domain producers
tests/                 # reference, corpus, differential, CLI, and assurance tests
corpus/                # pinned shared and R2U2 differential records
evidence/              # immutable retained records; nothing writes here
schemas/               # two frozen evidence schemas; nothing validates against them
assurance/             # what this repository declares; no evidence, no verdict
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # the pin classifier, the compatibility view, the chain driver
```
