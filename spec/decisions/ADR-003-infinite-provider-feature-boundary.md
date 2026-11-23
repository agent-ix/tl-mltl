---
id: ADR-003
title: "Keep the infinite-trace provider behind a non-default tl-mltl feature"
type: ADR
status: accepted
owner: kreneskyp
relationships:
  - target: ix://agent-ix/tl-mltl/FR-027
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-028
    type: relates_to
  - target: ix://agent-ix/tl-mltl/FR-029
    type: relates_to
---

# ADR-003: Keep the infinite-trace provider behind a non-default tl-mltl feature

## Status

**Accepted for the V1 spec cycle.** The owner's TL-88 decision selected a
feature-gated `tl_mltl::infinite` module by default, subject to verifying that
the bounded build remains separate. The owner clarified during this cycle that
TL is prerelease software and no whole-crate certification audit is being
requested. A speculative source-audit policy is not an architectural input.

## Context

The released tl-mltl 0.3.0 crate has no feature table and exposes bounded
`future`, `past`, `wire` and mapping modules. It has no `#![no_std]` declaration
and its bounded source already uses `std`. Adding infinite semantics need not
change these existing entry points or their wire formats. It does need a
separate opt-in API, because a finite prefix cannot prove liveness and callers
must explicitly select the TL-native `mltl.infinite-trace/v1` profile.

## Decision

Add a non-default `infinite-trace` Cargo feature exposing
`tl_mltl::infinite`. The module consumes only public tl-syntax
`formula-unbounded/v1`, lasso, fairness and partial-valuation contracts. The
bounded modules do not import `infinite::*`; the infinite module does not
reuse the bounded evaluator as its semantic oracle. Only this module can
construct an FR-341 `proved` infinite-trace result and register the one
`tl-syntax.liveness/v1` backend. The independent `tl-oracle` remains a
separate unpublished dev-only crate depending only on tl-syntax.

The feature is absent from the default build and from tl-rewrite's production
`tl-mltl` dependency. Cargo feature unification is checked at the resolved
graph: a consumer that enables `infinite-trace` enables it throughout that
resolution, which is normal Cargo behavior and must be visible in the lockfile
and feature tree. The repo builds and tests both default and feature-enabled
configurations. A feature-off build must preserve the 0.3.0 bounded API,
behavior, wire bytes and dependency closure except for explicit version pins.

## Boundary checks

- Inspect the module import graph: `future`, `past`, `wire`, `mapping` and their
  shared helpers have no import of `infinite` or feature-dependent branch that
  changes bounded semantics.
- Compare `cargo tree --no-default-features` and the default dependency graph
  against the 0.3.0 baseline. Any new provider-only dependency is optional and
  absent with the feature off.
- Run the bounded regression suite with the feature off and with all features;
  results, golden wire bytes and public bounded types must agree.
- Build an external consumer that depends on tl-mltl with the feature off and
  inspect the resolved feature tree. Then enable the feature in a separate
  consumer and verify that only the explicit opt-in reveals infinite APIs.
- Record that `no_std` is unsupported in the 0.3.0 bounded baseline; this ADR
  neither creates nor weakens a `no_std` claim.

## Consequences

The campaign remains four production TL crates. TL-213's release graph keeps
those four crates and checks both tl-mltl feature sets. TL-211's infinite C2PO
export and TL-212's infinite verification evidence live under the module's
feature gate; bounded exports remain unchanged. tl-rewrite uses tl-oracle in
tests and does not enable tl-mltl's infinite feature in production, so its
bounded dependency does not accidentally select the provider. If a measured
boundary check fails during implementation, the team must resolve that
specific failure before release; a new crate is a fallback design decision,
not a speculative requirement of this ADR.

## Rejected option

A separate `tl-infinite` crate would make package-level exclusion automatic,
but adds a fifth production pin, release lane and duplicated public-type
routing without an established need. The non-default module gives the needed
API and build separation for this prerelease system.
