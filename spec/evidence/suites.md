---
id: SUR-001
title: tl-mltl v0.1 evidence suite registry
type: SuiteRegistry
---

# tl-mltl v0.1 evidence suite registry

## Suites

| ID | Name | Command | Tool | Evidence Kind |
|---|---|---|---|---|
| SUITE-001 | Complete repository CI | `make ci` | GNU Make and Cargo | Integration |
| SUITE-002 | Specification validation | `quire validate --scope . 'spec/**/*.md'` | quire-cli 0.31.0 | Analysis |
| SUITE-003 | Requirement coverage export | `quire coverage --scope . --json` | quire-cli 0.31.0 | Analysis |
| SUITE-004 | Shared temporal corpus replay | `cargo run --example reference_conformance -- --manifest corpus/tl-syntax-v1/manifest.json` | tl-mltl reference conformance producer | Integration |
| SUITE-005 | CLI conformance | `cargo run --example cli_conformance -- --requests tests/fixtures/cli-requests/manifest.json` | tl-mltl CLI conformance producer | Integration |
| SUITE-006 | R2U2 differential replay | `cargo run --example r2u2_differential -- --manifest corpus/r2u2-v4.2/manifest.json` | tl-mltl R2U2 differential producer | Integration |
| SUITE-008 | Compiled Rust test census | `python3 scripts/rust_test_census.py` | tl-mltl test census | Static |
| SUITE-009 | Shared pin classification | `.venv-assurance/bin/python scripts/check_shared_pins.py` | engineering-assurance 0.2.0 | Analysis |

## What backs these rows, and what does not

Six of the eight rows are bound by a test that invokes that suite's own command.
`SUITE-001` and `SUITE-002` are deliberately unbacked.

`SUITE-007`, the legacy evidence compatibility view, is gone rather than
unbacked: the repository owner released the preservation constraint for the
pre-stable phase on 2026-09-02 (`agent-ix/engineering-assurance#7`) and the
retained records and their reader were deleted under `agent-ix/tl-mltl#16`. The
row is removed with the suite; it is not left standing as a command nobody can
run.

A row backed by a test that does not run its command is a row that reports
coverage and measures nothing, which is why each remaining binding names one
suite and runs that suite's command.

`SUITE-001` is `make ci`, the composite that contains every other suite. Nothing
binds it because a test that ran `make ci` would be a test the composite runs,
which recurses. `SUITE-002` is the `quire validate` half of `make spec`; it
writes no structured result the chain reads, and it is one of the gates named in
`NFR-003` as unprotected by the structural backstop. Both are stated here rather
than bound by a tag that would make the registry look complete.
