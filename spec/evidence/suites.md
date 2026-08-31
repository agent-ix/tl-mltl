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
| SUITE-002 | Specification validation | `make spec` | quire 0.31.0 | Analysis |
| SUITE-003 | Requirement coverage | `quire coverage --scope . --strict` | quire 0.31.0 | Analysis |
| SUITE-004 | Shared temporal corpus | `cargo test --test shared_corpus` | Rust test harness | Integration |
| SUITE-005 | CLI and schema conformance | `cargo test --test cli` | Rust test harness | Integration |
| SUITE-006 | Monitor differential corpus | `cargo test --test differential` | Rust test harness | Integration |
| SUITE-007 | PGM-01 evidence envelope | exact merged schema and validator | Python Draft 7 | Analysis |
