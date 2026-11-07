---
id: SR-079
title: IR-1377 tl-mltl#126 code-review review
type: SpecReview
analysis: code-review
scope: agent-ix/tl-mltl@b7a013600419af3465e7f2608745eae426f99f95; Cargo.toml, Cargo.lock,
  fuzz/Cargo.toml, fuzz/Cargo.lock, tests/future_interop.rs, tests/tc138_feature_boundary.rs
review_set: subset
---

## Summary

Rust/code review (rust-review lane folded in) of tl-mltl#126, which switches tl-syntax from `rev = 6aa9b11e` to `branch = "main"` (resolves d97d3d73) in Cargo.toml and fuzz/Cargo.toml and replaces one manifest assertion in tests/future_interop.rs. Ticket: IR-1377. Read-only review: no cargo build or test was run (shared host); findings are traced from source and upstream manifests.

## Verdict

**FAIL** — the switch is clean (no shim, no re-export, no compat layer) and the future_interop.rs assertion change is forced by upstream and keeps FR-017-AC-3's intent, but tests/tc138_feature_boundary.rs still generates an external consumer that pins tl-syntax `rev = 6aa9b11e`, which reproduces the duplicate-tl-syntax E0308 this PR exists to remove. Upstream check: tl-syntax d97d3d7 corpus/future-operators/manifest.json has no `source_cross_check` (nor `files`) key and has `derived_dialect: "tl-parse.clean-ascii/v2"`; 6aa9b11 had `source_cross_check.parser: "tl-parse"`; d97d3d7 is 14 commits ahead of 6aa9b11, 0 behind. The second Cargo.lock tl-syntax entry (0.4.0, rev 6e2fc17, on upstream branch codex/tl-207-v1-spec, diverged from main) comes from the dev-dependency tl-oracle@9bf3994; dev-dependencies of a dependency are not resolved in a downstream graph, so it cannot cause a Formula mismatch in quire-mltl. quire-mltl itself must still name tl-syntax with the identical `branch = "main"` source (a `rev =` pin, even to d97d3d7, is a distinct Cargo source).

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | TC-138's generated external consumer still declares tl-syntax with `rev = 6aa9b11e`; with tl-mltl now on `branch = "main"` the consumer resolves two tl_syntax crates, so passing its `tl_syntax::Formula` into `tl_mltl::evaluate_closed`/`map_to_c2po` is an E0308 and the `cargo run` success asserts fail. Contradicts the PR's "cargo test OK" and is the same duplicate-source defect the PR targets. Change it to `branch = "main"`. | tests/tc138_feature_boundary.rs:130 |
| FND-002 | low | The comment after the replaced assertion still says the parser "is a recorded upstream cross-check", and the new assert message says the manifest "records the tl-parse cross-check dialect"; upstream main removed the cross-check record, so both now describe data that no longer exists. | tests/future_interop.rs:352-360 |

## Coverage

Examined: Cargo.toml, fuzz/Cargo.toml, Cargo.lock, fuzz/Cargo.lock (tl-syntax entries), tests/future_interop.rs (TC-083 / FR-017-AC-3), tests/tc138_feature_boundary.rs (consumer manifest), upstream tl-syntax manifests at 6aa9b11 and d97d3d7, tl-oracle@9bf3994 Cargo.toml, quire-mltl Cargo.toml. gap-analysis skipped: no production code changed; manual check of FR-017-AC-3 against TC-083 found its three clauses still asserted. No cargo commands run; CI is workflow_dispatch-only and has no run for this head.
