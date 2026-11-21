---
id: SR-614
title: QSL-2324 upstream gap-analysis
type: SpecReview
analysis: gap-analysis
scope: agent-ix/tl-mltl; QSL-2324; src/wire/common.rs,
  tests/owner_reader_errors.rs, existing reader callers, Cargo.toml, Cargo.lock
review_set: subset
relationships:
  - target: ix://agent-ix/tl-mltl/MRS-001
    type: references
---

## Summary

Bounded gap review of the changed canonical-reader API, its callers and owner tests, with a repository-wide static matrix retained as context. Plan completion: not assessed.

Frozen head: dccf18830b2e8560620c860fb6bec03b7cb53cf4. Base and live remote main: d8457736d431b1401802e81581d43a715a227549. Ticket: QSL-2324; PR: tl-mltl#187. No applicable AssuranceProfile. No spec/plan changes, so spec-review and its sub-analyses do not apply.

## Verdict

CONDITIONAL — bounded changed-surface inspection found no additional gap; this is not a repository assurance PASS. The repository matrix reports 26 untagged rows (including withdrawn and unchanged metric/constraint rows); their full adjudication and a whole-tree reverse-gap audit are outside this authorized bounded review.

## Findings

| ID | Severity | Summary | Refs |
| --- | --- | --- | --- |
| FND-001 | low | No findings (placeholder) | - |

## Coverage

The change preserves preflight → deserialize/end → semantic validator → canonical equality → expected equality. Boundary errors convert through From; semantic errors propagate directly. Owner maxima, clamping, allocation/traversal checks and merged usage are unchanged. No copied parser, RefCell, shim, placeholder production behavior, or CI/gate weakening was introduced. Existing callers in trace, command, past history/requirement/result and tests were inspected. Syntax dependency and lock agree on revision 9f7b1c868df605cb6e885291db95a948cd08829c; tl-oracle retains its own older syntax dev dependency. Explicit generic calls at tests/a0_00_wire_survivors.rs:262,265,296,523 need propagation; merely adding a third underscore may also require an OwnerReadError annotation where Ok-only validators leave E ambiguous.

Retained evidence reviewed: owner-reader-errors.log has 4 passed, 0 failed, 0 ignored; commands.txt records scoped clippy and rustfmt exit 0. No additional Cargo run by reviewer. Byte-limit runtime is pending elsewhere; the local log is empty and commands.txt states its queued attempt was cancelled before Cargo. Full guarded-ci, Miri, deny, all-target compilation and whole-crate tests were not run in this bounded authorization.

Reconciliation: quire matrix (quire 0.36.2, engine 0.50.2), no run evidence read; exit 0. Repository static status: tagged 131; untagged 26; method-without-symbol 2. All FR-018 AC1–3 and NFR-001 AC1–5 are tagged. quire coverage exit 0: untracked_symbols is empty; Rust census candidates 523, tagged 443, bound 438. Diagnostics include archetypes matching nothing and uncatalogued Withdrawn verification methods; zero criteria or extraction success is not inferred from those diagnostics. See matrix.json and coverage.json. New tests exercise real code with discriminatory typed assertions. Bounded reader behavior inventory: 5 stages, 0 new unowned behaviors, 0 source/test stubs; no whole-tree reverse-gap claim. Optional semantic audit skipped.

Plan completion: not assessed

## Examined scope

```yaml
scope:
  - id: FR-018-AC-1
    path: spec/requirements/FR-018-publish-temporal-owner-wire.md
    role: examined
    excerpt: Every one of the five retained owner contracts publishes exact schema
      bytes/digest, and its bounded reader accepts only canonical bytes matching
      that digest, returning a constructor-private validated view.
  - id: FR-018-AC-2
    path: spec/requirements/FR-018-publish-temporal-owner-wire.md
    role: examined
    excerpt: Every reader rejects unknown/duplicate/missing/reordered fields,
      trailing/noncanonical bytes, and identity/digest mutation with a typed
      error and no partial output; every owner-declared resource ceiling is
      enforced before allocation or traversal.
  - id: FR-018-AC-3
    path: spec/requirements/FR-018-publish-temporal-owner-wire.md
    role: context_only
    excerpt: Removing the `filament-observation`-coupled request/result/mapping
      layer (TL-179) changes no existing future evaluator, past evaluator, CLI
      schema, or `mapping::legacy` C2PO mapping behavior.
  - id: NFR-001-AC-1
    path: spec/requirements/NFR-001-determinism-resources.md
    role: context_only
    excerpt: Repeated evaluation, analysis, mapping, context binding, digesting,
      comparison, and serialization are byte-for-byte stable.
  - id: NFR-001-AC-2
    path: spec/requirements/NFR-001-determinism-resources.md
    role: context_only
    excerpt: Overflow and work-limit exhaustion are explicit errors with no partial
      success claim.
  - id: NFR-001-AC-3
    path: spec/requirements/NFR-001-determinism-resources.md
    role: examined
    excerpt: "Each limit in the Limit Register below is tested on both sides: a
      value equal to the limit is accepted; a value one past the limit is
      refused with the typed error that names that limit. The native run is
      never skipped. For a fixed ceiling (the recursion ceiling, the owner-wire
      maxima, the past size constants) it uses the production value; for a limit
      the caller supplies (`EvaluationLimits`, `PastEvaluationLimits`, infinite
      `EvaluationLimit`, the mapping work limit) it sets the limit to the exact
      requirement of the input. The Miri run (NFR-001-AC-5) runs the same code
      path with a lowered limit "
  - id: NFR-001-AC-4
    path: spec/requirements/NFR-001-determinism-resources.md
    role: context_only
    excerpt: Every variant of every public error and error-code enumeration is
      produced by at least one test that asserts the variant itself, not its
      message; a variant that no reachable input can produce is removed from the
      enumeration in the same change instead of being left untested. The one
      exception is `ContextualBindingError::Refused`, which preserves a refusal
      of the non-exhaustive upstream `FormulaBindingError`; it is produced only
      when tl-syntax adds a second binding refusal.
  - id: NFR-001-AC-5
    path: spec/requirements/NFR-001-determinism-resources.md
    role: context_only
    excerpt: "The unit and property suites complete under Miri, on the pinned
      nightly named in the Makefile, with no undefined-behavior report, using
      the recorded flags and reduced property case counts. Heavy limits run
      under Miri with a caller-lowered limit where the API takes one, and with a
      named `cfg(miri)` constant otherwise, exactly as the Limit Register
      states. Because of those `cfg(miri)` constants in `src/past/mod.rs`, Miri
      does not exercise the production values of `MAX_HISTORY_POSITIONS` and
      `MAX_PROPOSITIONS_PER_POSITION`; the native run does, and this is an
      accepted risk. Miri runs the library "
  - id: src/wire/common.rs::read_expected
    path: src/wire/common.rs
    role: examined
    excerpt: pub fn read_expected<T, F, E>(
  - id: Cargo.toml::tl-syntax
    path: Cargo.toml
    role: examined
    excerpt: tl-syntax = { version = "=0.4.0", git =
      "https://github.com/agent-ix/tl-syntax.git", rev =
      "9f7b1c868df605cb6e885291db95a948cd08829c", default-features = false,
      features = ["alloc", "serde"] }
  - id: tests/a0_00_wire_survivors.rs::read_expected
    path: tests/a0_00_wire_survivors.rs
    role: examined
    excerpt: read_expected::<Vec<u32>, _>
bindings:
  - test_id: semantic_refusal_propagates_before_expected_value_comparison
    ac_id: FR-018-AC-2
    trace: correct
  - test_id: boundary_refusal_converts_before_the_validator_runs
    ac_id: FR-018-AC-2
    trace: correct
  - test_id: boundary_refusal_converts_before_the_validator_runs
    ac_id: NFR-001-AC-3
    trace: correct
  - test_id: canonical_and_expected_refusals_convert_to_the_caller_error
    ac_id: FR-018-AC-2
    trace: correct
  - test_id: canonical_value_and_usage_survive_the_caller_error_type
    ac_id: FR-018-AC-1
    trace: correct
```

## Disposition round 1

Disposition round 1 inspected frozen head 424a66102c2a52ba7252dd041e074c8300cd07da. The only source delta from dccf18830b2e8560620c860fb6bec03b7cb53cf4 imports OwnerReadError and updates the four existing explicit calls in tests/a0_00_wire_survivors.rs:262,269,300,527 to three type arguments with E explicitly OwnerReadError. This supplies the correct arity, resolves the Ok-only validator error inference and retains the original typed error assertions. The complete bounded reader caller inventory has no remaining old two-argument explicit calls. No production code, dependency, lock, validator, or limit change was introduced in this fix round. No new findings.

Scoped rustfmt and diff checks passed per parent receipt. Earlier four owner-reader runtime tests and scoped clippy remain retained evidence for the unchanged production code. Three exact affected caller test filters at this head and the one production byte-limit test at production-identical dccf188 are queued elsewhere; runtime receipts remain pending and owned by the parent. No reviewer Cargo invocation or broad gate. Merge readiness is conditional on those runtime receipts and parent custody of these updated artifacts.

Gap verdict remains CONDITIONAL: no new gap was introduced, the initial placeholder is not a defect requiring disposition, and the unchanged matrix baseline and bounded-audit limits remain as recorded.

## Final runtime evidence

Final runtime receipts inspected at 061cace5ed58642f08fdcfba550bddec04659b33. The delta from reviewed 424a66102c2a52ba7252dd041e074c8300cd07da changes only the two committed review artifacts; production remains identical to dccf18830b2e8560620c860fb6bec03b7cb53cf4. final-regressions/byte-limit.log records 1 passed, 13 filtered, 0 failed, 0 ignored. final-regressions/callers.log records 3 passed, 9 filtered, 0 failed, 0 ignored, including all three affected caller tests. Parent reports both commands exited 0 and both remote queued attempts were cancelled before starting. Combined with the retained 4 owner-reader-error tests, 8 focused tests passed. Scoped clippy and rustfmt retain their prior passing results. No runtime condition remains open; no further Cargo execution or full gate is claimed. Parent must copy the latest two scratch artifacts verbatim for review custody before merging. QSL-2324 remains open for filament-mltl#31 and the FSL consuming change.

The gap-analysis verdict remains CONDITIONAL for the unchanged matrix baseline and bounded audit scope; passing runtime does not establish a full repository assurance PASS.
