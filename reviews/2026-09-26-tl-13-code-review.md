---
id: SR-061
title: code-review of TL-13 PR 99
type: SpecReview
analysis: code-review
scope: agent-ix/tl-mltl@fd116ec3b770bc21372a1e954327b7414c71e079; Cargo.toml, Cargo.lock,
  src/infinite/mod.rs, src/infinite/periodic.rs, src/lib.rs, src/main.rs, tests/future_parity.rs,
  tests/infinite_corpus.rs, tests/infinite_oracle.rs, tests/infinite_trace.rs, tests/support/oracle_bridge.rs,
  tests/tc149_semantic_laws.rs, spec/assurance/AD-002.md, spec/decisions/ADR-003-infinite-provider-feature-boundary.md,
  spec/infinite-trace-test-matrix.md, spec/requirements/FR-027-infinite-trace-crate-boundary.md,
  spec/requirements/FR-028-liveness-backend-registration-boundary.md, spec/requirements/FR-029-infinite-trace-downstream-evidence.md,
  spec/requirements/FR-030-infinite-possibility-semantics.md, spec/requirements/FR-031-infinite-temporal-semantics.md,
  spec/requirements/FR-032-lasso-fairness-admission.md, spec/requirements/FR-033-infinite-settlement.md,
  spec/requirements/FR-034-infinite-identity-limits.md, spec/spec.md, spec/test-matrix.md
review_set: subset
---

# code-review of TL-13 PR 99

## Summary

Ticket TL-13; frozen agent-ix/tl-mltl PR #99 at fd116ec3b770bc21372a1e954327b7414c71e079, base ca9fab04ea5f3b64686e697a3870561329e03fd2. Reviewed Cargo.toml, Cargo.lock, src/infinite/mod.rs, src/infinite/periodic.rs, src/lib.rs, src/main.rs, tests/future_parity.rs, tests/infinite_corpus.rs, tests/infinite_oracle.rs, tests/infinite_trace.rs, tests/support/oracle_bridge.rs, tests/tc149_semantic_laws.rs, spec/assurance/AD-002.md, spec/decisions/ADR-003-infinite-provider-feature-boundary.md, spec/infinite-trace-test-matrix.md, spec/requirements/FR-027-infinite-trace-crate-boundary.md, spec/requirements/FR-028-liveness-backend-registration-boundary.md, spec/requirements/FR-029-infinite-trace-downstream-evidence.md, spec/requirements/FR-030-infinite-possibility-semantics.md, spec/requirements/FR-031-infinite-temporal-semantics.md, spec/requirements/FR-032-lasso-fairness-admission.md, spec/requirements/FR-033-infinite-settlement.md, spec/requirements/FR-034-infinite-identity-limits.md, spec/spec.md, spec/test-matrix.md.

## Verdict

FAIL.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | high | Registered liveness settlement drops typed reason, evidence, and provider identity: a conflicting or resource-incomplete lasso routed through ProviderRegistry yields only LivenessDisposition, so clients cannot obtain FR-028/029/033 detail from the selected provider. | src/infinite/mod.rs:1190-1206 |

## Assurance Context

AP-001 (spec/assurance/AP-001.md) was considered against exact candidate fd116ec3b770bc21372a1e954327b7414c71e079 and base ca9fab04. It explicitly names finite reference verdict and horizon impact, while this PR adds an opt-in infinite provider and changes the syntax pin. AD-002 and ADR-003 boundaries were examined. No source-release, measurement, exception, or independent assurance decision for this candidate is available; no such conclusion is inferred from focused tests.

## Rust review

Checked public docs, exhaustive enum mappings, input bounds, checked arithmetic, panic surface, wire typing, feature gating, test seams, and the independent oracle bridge. No CI workflow file changed. Focused checks reported by the coder were fmt, default check, feature Clippy, 25 selected feature-on and seven feature-off tests; aggregate gates were expressly withheld.

## New findings (disposition pass 1)

The fix round was reviewed at `1b29960d3612e88c61fbc1871642aee63ac8cf1f`. These defects were absent from the original findings table.

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-002 | high | The accepted FR-028/FR-033 contract requires timeout to settle as `failed` with `resource-incomplete`, but `EvaluationLimit` has only count ceilings and no deadline, elapsed-time budget, or cancellation-to-result path. A caller whose evaluation times out cannot receive the required typed result. | src/infinite/mod.rs:27-42; FR-028:51; FR-033:37 |
| FND-003 | medium | The new TC-138 external consumer runs nested Cargo without `--offline` or a retained lockfile. In a network-restricted ordinary test run it updates crates.io/git and fails before checking the feature boundary, even though the parent crate is already built; the same focused test passes with `CARGO_NET_OFFLINE=true`. | tests/tc138_feature_boundary.rs:8-26 |
| FND-004 | medium | TC-159 unconditionally expects `u64::MAX` event position to prove. On a 32-bit target `usize::try_from(u64::MAX)` fails and the provider correctly returns resource-incomplete, so this new test panics at `unwrap()` instead of checking the target-dependent result. | tests/infinite_trace.rs:793-799; src/infinite/mod.rs:509-510 |

## Dispositions

Round 1 reviewed `1b29960d3612e88c61fbc1871642aee63ac8cf1f` (fix commit `1b29960d3612e88c61fbc1871642aee63ac8cf1f`). Original finding text above is unchanged.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-001 | fixed | 1b29960d3612e88c61fbc1871642aee63ac8cf1f: `ProviderRegistry::settle_detailed` returns `InfiniteResult`; `evaluate_selected` shares exact subject binding with the coarse FR-290 route, and the new test checks reason, evidence and identity. |
| FND-002 | still-open | No deadline, elapsed-time limit or timeout mapping exists in `EvaluationLimit` or the provider path; FR-028/FR-033 timeout settlement remains unavailable. |
| FND-003 | still-open | TC-138's nested Cargo invocation requires index/git network access by default and has no lockfile; the network-restricted focused run failed before its assertions. |
| FND-004 | still-open | The test's `u64::MAX` success assertion is unconditional despite the checked `u64` to `usize` conversion on 32-bit targets. |

### Round 2

Reviewed `4ca956212063c718d26015d8a03c63d118175ed7` (fix commit `4ca956212063c718d26015d8a03c63d118175ed7`). No new findings arose from the changed source, tests, or requirement text.

| FND | outcome | sha/reason |
| --- | --- | --- |
| FND-002 | fixed | 4ca956212063c718d26015d8a03c63d118175ed7: `EvaluationLimit` accepts an optional monotonic `Instant` deadline, preflight and periodic work poll it, and expired lasso/prefix requests settle as typed `Failed`/`ResourceIncomplete` without evidence. The registered detailed and coarse routes preserve the matching dispositions. |
| FND-003 | fixed | 4ca956212063c718d26015d8a03c63d118175ed7: TC-138 sets `CARGO_NET_OFFLINE=true` for nested Cargo and uses a writable in-repo target scratch directory; the ordinary focused test now passes without a network environment override. |
| FND-004 | fixed | 4ca956212063c718d26015d8a03c63d118175ed7: TC-159 branches on checked `usize::try_from(u64::MAX)`, expecting proof only when representable and typed resource-incomplete otherwise, retaining the exact selected-position identity. |

### Post-PASS CLI fixture review

Reviewed frozen `67d567105e857d4cf6dfbe014151bfd77a184ad3` against trailing review head `051187a` and base `ca9fab04ea5f3b64686e697a3870561329e03fd2`. The only new diff updates `tests/cli.rs:96`'s expected tl-syntax revision from `4a5614193d21e5ae99950ae683b04ba0ec931358` to `cfc2761cbdf9aa6e30f1b04db5c2a0c00023e4a0`, matching `Cargo.toml:25` and `src/lib.rs:92`. This repairs the fixture without changing production behavior, test intent, or prior finding outcomes. No new findings or open findings.

Focused feature-off and feature-on CLI tests each passed 1/1. `cargo fmt --check`, `git diff --check 051187a..67d5671`, and the scratchpad-to-PR review-file comparison passed. No aggregate gate was run.
