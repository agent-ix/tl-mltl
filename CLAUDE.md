# tl-mltl

Finite-trace MLTL evaluation, horizon analysis, and runtime-monitor interoperability.

## Commands

```bash
make fmt            # format with rustfmt
make fmt-check      # verify formatting (CI gate)
make lint           # clippy with -D warnings
make test           # cargo test
make build          # release build
make clean          # cargo clean
make deny           # cargo deny check licenses and sources
make audit-unsafe   # check that every unsafe block has a // SAFETY: comment
make check-corpus   # verify shared and R2U2 corpus checksums
make verify-evidence # verify checksums and re-derive retained outcomes
make spec           # validate specs and strict coverage
make rustdoc        # build warning-free public docs
make evidence-tool  # test evidence classifiers and schemas
make ci             # complete local gate
```

GitHub Actions is intentionally `workflow_dispatch`-only. Use local `make ci`
while iterating and dispatch hosted CI only for a finalized revision.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.75` and caps cognitive complexity / arg count
- `deny.toml` allow-lists licenses and denies unknown registries/git sources
- `scripts/check_unsafe_comments.sh` runs in CI and locally via `make audit-unsafe`. Every `unsafe {` block must have a `// SAFETY:` comment within the 3 preceding lines, or be listed in `scripts/unsafe_comment_baseline.txt`. Update the baseline with `bash scripts/check_unsafe_comments.sh --update-baseline`.
- `rustfmt.toml` uses 100-char width and `StdExternalCrate` import grouping. CI fails on drift.
- `rust-toolchain.toml` pins to stable + rustfmt + clippy.

## Layout

```
src/lib.rs             # crate root
src/evaluate.rs        # bounded closed/prefix reference evaluation
src/horizon.rs         # checked structural lookahead
src/mapping.rs         # deterministic C2PO mapping manifest
src/main.rs            # JSON command CLI
tests/                 # reference, corpus, differential, CLI, and evidence tests
corpus/                # pinned shared and R2U2 differential records
evidence/              # immutable candidate collections
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # local tooling
```
