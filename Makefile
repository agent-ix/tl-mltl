# =============================================================================
# TL MLTL Makefile
# =============================================================================
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the reference-conformance replay and the R2U2
# differential replay for the semantics, quire for static export.
#
# This file is not a trust root and does not try to be one. The parse-time
# guards that used to police Make's own execution controls — MAKEFLAGS,
# the rustup and loader overrides, CARGO_TARGET_DIR, the tool origin checks and
# the static failure-propagation inspection — went with the collector they were
# protecting.
#
# `make ci` alone still trusts Make's own execution controls and Make's own
# exit code. Nothing in this Makefile changed that.
# What changed is that `make ci` is no longer the assured entry point: run
# `make guarded-ci` instead. It wraps `make ci` with a Rust program, external
# to Make, that (1) refuses to invoke Make at all if this file's text — or any
# file it `include`s — carries an execution-control surface capable of
# suppressing prerequisite-failure propagation, or if the calling
# environment's MAKEFLAGS carries the same suppression; and (2), after Make
# returns, reconciles the set of gates that actually wrote a completion
# record against the declared `ci` prerequisite set below, independent of
# Make's own exit code. See `spec/requirements/NFR-006-gate-set-integrity.md`
# and `src/ci_guard.rs`. `make ci` remains directly invocable for local
# convenience; a person who runs it directly instead of `make guarded-ci`
# bypasses the binding, and that residual is disclosed rather than solved by
# removing the convenience — see NFR-006's Scope.
#
# Tracked as agent-ix/tl-mltl#14 (Linear TL-65), remediated by
# NFR-006-gate-set-integrity.

CARGO ?= cargo
GATE ?= $(CARGO) run --quiet --manifest-path tools/gate/Cargo.toml --
QUIRE ?= quire

# `ci_guard record` is the last step of every `ci` prerequisite's recipe, so
# it only runs on that recipe's own success. Run alone (e.g. `make lint` for
# local iteration, outside `make guarded-ci`), it is a deliberate no-op.
CI_GUARD ?= $(CARGO) run --quiet --bin ci_guard --

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make test-features    - cargo test under default and infinite-trace features"
	@echo "  make sweep-full       - Nightly full small-scope sweep against tl-oracle (release, long)"
	@echo "  make kani-check       - Verify every bounded Kani proof (docs/verification-tooling.md)"
	@echo "  make mutants          - Full cargo-mutants run + kill-rate thresholds (hours, weekly)"
	@echo "  make mutants-diff     - cargo-mutants on the diff against origin/main + gate"
	@echo "  make coverage         - llvm-cov line/function/region coverage (stable) with floor"
	@echo "  make coverage-branch  - llvm-cov line + branch coverage (nightly)"
	@echo "  make conformance      - Replay the shared corpus through the evaluator"
	@echo "  make differential     - Replay the retained R2U2 exchange"
	@echo "  make r2u2-past-differential - Live pinned R2U2 over the past grid (R2U2_ROOT)"
	@echo "  make r2u2-past-replay - Recompute that comparison from recorded outputs"
	@echo "  make r2u2-nightly     - Past grid + future suite against pinned R2U2 (R2U2_ROOT)"
	@echo "  make cli-conformance  - Drive the built CLI over its declared requests"
	@echo "  make campaign-inventory - Check the campaign member list is complete"
	@echo "  make campaign-run     - Run all campaign members, write target/campaign/report.json"
	@echo "  make campaign-replay REPORT=path - Source-bound replay of a retained report"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make msrv             - Check all targets and features with Rust 1.98.1"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make miri             - Unit and property suites under Miri (pinned nightly)"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make ci               - All CI gates locally, unguarded (see Makefile header)"
	@echo "  make guarded-ci       - The assured entry point: run this, not 'make ci'"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check
	$(CI_GUARD) record fmt-check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings
	$(CI_GUARD) record lint

# The proofs are verifier-only and do not run in `cargo test`; `cargo kani --lib`
# runs every harness; the gate instead runs each name in scripts/kani-harnesses.txt
# with --exact, and tests/kani_harnesses.rs fails when that list and the
# #[kani::proof] functions disagree, so a renamed or missing one fails.
.PHONY: kani-check
kani-check:
	$(CARGO) test --test kani_harnesses
	$(CARGO) run --quiet --bin kani_gate -- --list scripts/kani-harnesses.txt
	$(CI_GUARD) record kani-check

.PHONY: test
test:
	$(CARGO) test --all-targets --all-features
	$(CI_GUARD) record test

# Feature matrix for the test suites: default (no features) and infinite-trace.
# `test` above covers --all-features, which is the infinite-trace build; both are
# run here so a feature-gated test file or API cannot silently stop compiling or
# passing standalone. Named `test-features` (no campaign-inventory keywords).
.PHONY: test-features
test-features:
	$(CARGO) test --locked --all-targets
	$(CARGO) test --locked --all-targets --features infinite-trace
	$(CARGO) test --locked --doc
	$(CARGO) test --locked --doc --features infinite-trace
	$(CI_GUARD) record test-features

# =============================================================================
# MLTL domain
# =============================================================================

# The fast small-scope subset runs inside `make test`. The full sweep (wider
# intervals, longer traces, depth two over every shape and a seeded depth-three
# sample) is nightly work and is deliberately not a `ci` prerequisite. Evidence
# JSON lands in $$TL_MLTL_EVIDENCE_DIR (default target/tmp/small-scope-evidence).
.PHONY: sweep-full
sweep-full:
	$(CARGO) test --release --test finite_oracle -- --ignored --nocapture

.PHONY: conformance
conformance:
	$(CARGO) run --quiet --example reference_conformance
	$(CI_GUARD) record conformance

.PHONY: differential
differential:
	$(CARGO) run --quiet --example r2u2_differential -- \
		--manifest corpus/r2u2-v4.2/manifest.json
	$(CI_GUARD) record differential

# Live past-time R2U2 differential (TL-217). Needs a built R2U2 checkout at
# $(R2U2_ROOT) on the revision pinned in campaign/tool-versions.json. Without
# one it writes the generated inputs and a not_run report and exits 2; it never
# reports a pass. `r2u2-past-replay` recomputes the comparison from the recorded
# outputs under target/r2u2 without running R2U2.
.PHONY: r2u2-past-differential
r2u2-past-differential:
	$(CARGO) run --quiet --bin r2u2_live -- run --out target/r2u2

.PHONY: r2u2-past-replay
r2u2-past-replay:
	$(CARGO) run --quiet --bin r2u2_live -- replay --out target/r2u2

# Nightly live R2U2 differential (TL-237): the past grid and the exhaustive
# future suite against the pinned checkout at $(R2U2_ROOT). Local make target by
# design (hosted CI is manual-only); run it from cron/launchd on a machine that
# has R2U2 built. Exits 2 and reports `not_run` when R2U2 is absent.
.PHONY: r2u2-nightly
r2u2-nightly:
	$(CARGO) run --quiet --bin r2u2_live -- run --out target/r2u2/past
	$(CARGO) run --quiet --bin r2u2_live -- run --suite future --out target/r2u2/future

# The example drives the CLI binary, so the binary has to exist. It is built
# here rather than located by the example, because an example that can build its
# own subject can report a green run against a stale one.
.PHONY: cli-conformance
cli-conformance:
	$(CARGO) build --quiet --bin tl-mltl
	$(CARGO) run --quiet --example cli_conformance -- \
		--requests tests/fixtures/cli-requests/manifest.json
	$(CI_GUARD) record cli-conformance

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Verification effectiveness (TL-231, TL-234). Heavy, so deliberately outside
# `make ci`; see docs/verification-tooling.md for budgets and measured numbers.
# =============================================================================

MUTANTS_OUT ?= target/mutants
MUTANTS_JOBS ?= 4
MUTANTS_BASE ?= origin/main
COVERAGE_MIN_LINES ?= 80
COVERAGE_IGNORE = '(ci_guard|kani_|fuzz_campaign)'

# The wrapper accepts cargo-mutants' "survivors found" status and judges the
# result against mutation-thresholds.toml (see tools/gate).
.PHONY: mutants
mutants:
	$(GATE) mutants --source-env --jobs $(MUTANTS_JOBS) $(MUTANTS_OUT)

.PHONY: mutants-diff
mutants-diff:
	$(GATE) mutants --source-env --jobs $(MUTANTS_JOBS) --diff-base $(MUTANTS_BASE) $(MUTANTS_OUT)
	$(CI_GUARD) record mutants-diff

.PHONY: coverage
coverage:
	$(CARGO) llvm-cov --all-features --summary-only \
		--ignore-filename-regex $(COVERAGE_IGNORE) \
		--fail-under-lines $(COVERAGE_MIN_LINES)

.PHONY: coverage-branch
coverage-branch:
	$(CARGO) +nightly llvm-cov --all-features --branch --summary-only \
		--ignore-filename-regex $(COVERAGE_IGNORE) \
		--fail-under-lines $(COVERAGE_MIN_LINES)

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses
	$(CARGO) deny check sources
	$(CI_GUARD) record deny

# Campaign (TL-236). The member list is campaign/members.json; `inventory`
# refuses an evidence-bearing Make target or fuzz target no member covers.
# `campaign-run` writes target/campaign/report.json; `campaign-replay` reruns
# REPORT against the exact source it names and writes a new attempt.
.PHONY: campaign-inventory
campaign-inventory:
	$(CARGO) run --quiet --bin campaign -- inventory
	$(CI_GUARD) record campaign-inventory

.PHONY: campaign-run
campaign-run:
	$(CARGO) run --quiet --bin campaign -- run

.PHONY: campaign-replay
campaign-replay:
	$(CARGO) run --quiet --bin campaign -- replay $(REPORT)

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md'
	$(QUIRE) coverage --scope . --strict
	$(CI_GUARD) record spec

.PHONY: msrv
msrv:
	rustup run 1.98.1 $(CARGO) check --locked --all-targets --all-features
	$(CI_GUARD) record msrv

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features
	$(CI_GUARD) record rustdoc

# =============================================================================
# Undefined-behavior check
# =============================================================================

# Miri needs a nightly toolchain, so it pins its own here instead of using
# rust-toolchain.toml. -Zmiri-disable-isolation lets the suites read the
# repository's corpus files; PROPTEST_DISABLE_FAILURE_PERSISTENCE stops proptest
# writing regression files; the property suites reduce their own case counts
# under Miri. MIRI_TESTS names the integration suites that run under Miri.
# Excluded (NFR-001-AC-5): suites that start processes or read external tools
# (ci_guard, campaign_run, cli, r2u2_live), and the exhaustive corpus, oracle and
# infinite-trace replays. past_mapping and infinite_laws run with a sampled
# pattern set and two proptest cases under cfg!(miri) in the tests themselves.
MIRI_TOOLCHAIN ?= nightly-2026-08-21
MIRI_TESTS ?= limit_register error_variants property past_history reference interop \
	contextual tc_090_owner_wire_contracts miri_gate past_readers past_mapping infinite_laws

.PHONY: miri
miri:
	MIRIFLAGS="-Zmiri-disable-isolation" PROPTEST_DISABLE_FAILURE_PERSISTENCE=true \
		rustup run $(MIRI_TOOLCHAIN) cargo miri test --all-features --lib $(addprefix --test ,$(MIRI_TESTS))
	$(CI_GUARD) record miri

# =============================================================================
# Fuzzing
# =============================================================================

# fuzz-check compiles every cargo-fuzz target and audits the fuzz package's
# dependency graph; it never starts a campaign. It needs a nightly toolchain
# and cargo-fuzz.
.PHONY: fuzz-check
fuzz-check:
	rustup run nightly cargo fuzz build
	cd fuzz && $(CARGO) deny check --config ../deny.toml
	$(CI_GUARD) record fuzz-check

# fuzz-smoke replays every checked-in corpus entry through its target once
# (no mutation, no campaign). Longer campaigns are manual: see fuzz/README.md.
.PHONY: fuzz-smoke
fuzz-smoke:
	rustup run nightly cargo fuzz list | xargs -I % rustup run nightly cargo fuzz run % fuzz/corpus/% -- -runs=0

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint kani-check test test-features conformance differential cli-conformance \
	campaign-inventory deny fuzz-check spec msrv rustdoc miri mutants-diff

# The assured entry point (NFR-006). Builds and runs the guard, which refuses
# to invoke `make ci` at all if this file's execution controls or the calling
# environment's MAKEFLAGS could suppress a prerequisite's failure, then
# reconciles the gates that actually completed against the declared list
# above regardless of Make's own exit code. `make ci` alone still does
# neither of those — see the header comment.
.PHONY: guarded-ci
guarded-ci:
	$(CI_GUARD) ci
