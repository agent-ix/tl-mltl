---
id: SR-045
title: "Rust review — temporal evaluator owner boundary"
type: SpecReview
analysis: code-review
scope: "src/future; src/past; src/wire; src/mapping; src/clock.rs; TC-084"
review_set: subset
---

# Rust review — temporal evaluator owner boundary

## Summary

Applied the Rust review checklist to the reorganized evaluator and every new
public owner seam: error typing, constructor privacy, untrusted-input bounds,
integer conversions, allocation behavior, recursion accounting, panic/unsafe
surface, and API compatibility.

## Verdict

**PASS after remediation.** Strict Clippy, formatting, documentation, release,
MSRV, dependency-policy, corpus-integrity, unsafe-audit, and the full explicit
non-qualification test suite pass. All review findings are fixed; none is
waived or deferred.

## Findings

| ID | Severity | Summary | Refs |
|---|---|---|---|
| FND-4501 | high | **FIXED:** Canonical output uses a bounded writer that refuses before serialization can allocate beyond the caller's clamped byte limit and never returns partial bytes. | `wire::common::BoundedWriter`; TC-084 limit boundaries |
| FND-4502 | high | **FIXED:** Wire and persistence usage fields are `u64`; every `usize` conversion and limit bridge is checked instead of cast or truncated. | `wire::common::UsageWire`; `wire::request`; `wire::report` |
| FND-4503 | high | **FIXED:** Future and past evaluators report exact attempted evaluation and recursion usage on success and refusal without changing legacy report bytes. | `future::evaluate`; `past::evaluate`; TC-084 resource rows |
| FND-4504 | high | **FIXED:** Decision support is accumulated into a bounded ordered set and refuses before allocating one element beyond the effective owner limit. | `wire::report`; TC-084 support boundaries |
| FND-4505 | medium | **FIXED:** Formula graph traversal is allocation-free per node, recursion and visits are charged before work, and owner maxima clamp caller limits. | `wire::common`; `wire::request` |
| FND-4506 | medium | **FIXED:** Reachable production panic/unreachable assumptions were replaced by typed errors. The change adds no unsafe block, async/blocking bridge, lock, or interior-mutability seam. | `src`; `scripts/check_unsafe_comments.sh` |
| FND-4507 | low | **FIXED:** Unsupported nightly-only rustfmt options were removed so the stable Rust 1.98 formatting gate is warning-free and deterministic. | `rustfmt.toml`; formatting gate |

The public module split preserves legacy paths by re-export while placing new
owner behavior behind typed validated views rather than flags or loosely typed
maps.
