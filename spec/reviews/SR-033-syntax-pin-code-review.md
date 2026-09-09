---
id: SR-033
title: "Code review — exact tl-syntax semantic-identity pin"
type: SpecReview
analysis: code-review
scope: "Cargo.toml, Cargo.lock, src/lib.rs, tests/cli.rs, README.md, corpus/README.md, assurance/pins.json"
review_set: subset
---

# Code review — exact tl-syntax semantic-identity pin

## Summary

This review examined the exact tl-syntax dependency revision, the exported
wire revision constant, lockfile, public CLI assertion, and factual pin
documentation. All live declarations now name one compiled revision while the
retained corpus basis remains explicitly distinct. No defect was found in the
changed dependency-alignment surface.

## Verdict

**ACCEPTED** — the durable tl-syntax main revision is aligned and tested. The
pre-existing local Python assurance tooling remains outside this patch.

## Assurance Context

AP-001 (`spec/assurance/AP-001.md`) applies because its scope includes the
exact syntax dependency. The evaluated baseline is `70b090f`; the reviewed
paths are the scope listed above. Available context was NFR-002-AC-2, the
CLI identity test, Cargo resolution, AP-001, and repository conventions. No
candidate-specific Quoin record was available; no AP-001 exception applies.
The legacy local Python assurance scripts were observed but not changed or
extended; they remain owned by the shared assurance migration.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-1201 | low | No defect found: Cargo.toml, Cargo.lock, `TL_SYNTAX_REVISION`, the CLI wire assertion, and the consumed assurance pin name the same semantic-identity revision, while the corpus basis stays a separate historical fact. | Cargo.toml:21, Cargo.lock:457, src/lib.rs:72, tests/cli.rs:95, NFR-002-AC-2 |
| FND-1202 | medium | Pre-existing Python assurance-chain scripts remain outside this dependency change and require the shared Quoin/Quire/Engineering-Assurance migration rather than a local replacement. | scripts/assurance_chain.py:1, scripts/check_shared_pins.py:1 |
