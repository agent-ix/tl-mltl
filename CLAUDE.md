# tl-mltl

Finite-trace MLTL evaluation, horizon analysis, and runtime-monitor
interoperability.

## Commands

```bash
make fmt              # format with rustfmt
make fmt-check        # verify formatting (CI gate)
make lint             # clippy with -D warnings
make test             # cargo test
make conformance      # replay the shared corpus through the evaluator
make differential     # replay the retained R2U2 exchange
make cli-conformance  # drive the built CLI over its declared requests
make test-census      # bind requirement-tagged tests to compiled tests
make deny             # cargo deny check licenses and sources
make audit-unsafe     # check that every unsafe block has a // SAFETY: comment
make spec             # validate specs and strict coverage
make msrv             # check all targets and features with Rust 1.98.1
make rustdoc          # build warning-free public docs
make ci               # complete local gate, unguarded (see Makefile header)
make guarded-ci       # the assured entry point; run this, not 'make ci'
```

GitHub Actions is intentionally `workflow_dispatch`-only. Use local `make
guarded-ci` while iterating and dispatch hosted CI only for a finalized
revision.

## Specification workflow

All new or changed work must be specified before implementation: use `quoin
write` to obtain the current artifact contracts, then update the relevant
requirements, plans, tasks, matrix rows, and evidence links. Before requesting
review, run `quoin review` over the affected scope and validate with Quire.
Record selected analyses and findings; final Quoin acceptance remains a human
decision and must not be advanced automatically.

## Producers

- `reference_conformance` distinguishes three wire-boundary refusals that
  `tl_syntax` surfaces only in a message, and `cli_conformance` distinguishes
  five CLI refusals that all exit 2. Both are cross-checked so a marker matching
  more than one case fails.
- **Nothing executes R2U2 or C2PO.** The external exchange under
  `corpus/r2u2-v4.2/` is retained; every claim is a replay against those bytes.

## The Makefile is not a trust root

Adding `.IGNORE:` to the `Makefile` makes recipes report success without running,
and a bare `make ci` does not notice.

**Run `make guarded-ci`, not a bare `make ci`.** This is remediated by
`NFR-006-gate-set-integrity` (Linear TL-65, `agent-ix/tl-mltl#14`): a Rust
program external to Make (`src/ci_guard.rs`, `src/bin/ci_guard.rs`) that
refuses to invoke Make at all if the Makefile text or the invocation
environment carries a state capable of suppressing prerequisite-failure
propagation, and reconciles the declared `ci` prerequisite set against the
gates that actually wrote a completion record, independent of Make's own
exit code. `make ci` remains directly invocable for local convenience and is
not itself the assured gate; a person who runs it directly instead of `make
guarded-ci` bypasses the binding, and that residual is disclosed rather than
solved by removing the convenience. See
`spec/requirements/NFR-006-gate-set-integrity.md`.

## Safety scaffolding

Backported from `agent-ix/ecaz`:

- `clippy.toml` pins MSRV to `1.98.1` and caps cognitive complexity / arg count
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
tests/                 # reference, corpus, differential, and CLI tests
corpus/                # shared and R2U2 differential records
spec/                  # requirements artifacts (from /spec-create-spec)
scripts/               # the test census and the unsafe-comment audit
```
