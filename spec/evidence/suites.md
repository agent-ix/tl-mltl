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
| SUITE-004 | Shared temporal corpus replay | `cargo run --example reference_conformance` | tl-mltl reference conformance producer | Integration |
| SUITE-005 | CLI conformance | `cargo run --example cli_conformance -- --requests tests/fixtures/cli-requests/manifest.json` | tl-mltl CLI conformance producer | Integration |
| SUITE-006 | R2U2 differential replay | `cargo run --example r2u2_differential -- --manifest corpus/r2u2-v4.2/manifest.json` | tl-mltl R2U2 differential producer | Integration |
| SUITE-008 | Compiled Rust test census | `python3 scripts/rust_test_census.py` | tl-mltl test census | Static |
