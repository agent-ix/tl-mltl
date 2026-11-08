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
# PYTHONOPTIMIZE, the rustup and loader overrides, CARGO_TARGET_DIR, the
# CARGO/PYTHON/QUIRE/SHA256SUM/BASH origin checks, and the static
# check_failure_propagation.py inspection — went with the collector they were
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
PYTHON ?= python3
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
	@echo "  make kani-check       - Verify the bounded Kani horizon proof"
	@echo "  make sweep-full       - Nightly full small-scope sweep against tl-oracle (release, long)"
	@echo "  make conformance      - Replay the shared corpus through the evaluator"
	@echo "  make differential     - Replay the retained R2U2 exchange"
	@echo "  make cli-conformance  - Drive the built CLI over its declared requests"
	@echo "  make campaign-inventory - Check the campaign member list is complete"
	@echo "  make campaign-run     - Run all campaign members, write target/campaign/report.json"
	@echo "  make campaign-replay REPORT=path - Source-bound replay of a retained report"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make msrv             - Check all targets and features with Rust 1.98.1"
	@echo "  make rustdoc          - Build warning-free public documentation"
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

# The proof is verifier-only and does not run in `cargo test`; keep its exact
# harness name in a local CI gate so a renamed private primitive cannot rot it.
.PHONY: kani-check
kani-check:
	$(CARGO) kani --lib \
		--harness future::horizon::kani_proofs::horizon_bound_addition_matches_checked_add \
		--exact --unwind 4 --output-format terse
	$(CI_GUARD) record kani-check

.PHONY: test
test:
	$(CARGO) test --all-targets --all-features
	$(CI_GUARD) record test

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
# Fuzzing
# =============================================================================

# fuzz-check compiles every cargo-fuzz target and audits the fuzz package's
# dependency graph; it never starts a campaign. It needs a nightly toolchain
# and cargo-fuzz.
.PHONY: fuzz-check
fuzz-check:
	$(CARGO) +nightly fuzz build
	cd fuzz && $(CARGO) deny check --config ../deny.toml
	$(CI_GUARD) record fuzz-check

# fuzz-smoke replays every checked-in corpus entry through its target once
# (no mutation, no campaign). Longer campaigns are manual: see fuzz/README.md.
.PHONY: fuzz-smoke
fuzz-smoke:
	@for target in $$($(CARGO) +nightly fuzz list); do \
		$(CARGO) +nightly fuzz run $$target fuzz/corpus/$$target -- -runs=0 || exit 1; \
	done

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint kani-check test conformance differential cli-conformance \
	campaign-inventory deny fuzz-check spec msrv rustdoc

# The assured entry point (NFR-006). Builds and runs the guard, which refuses
# to invoke `make ci` at all if this file's execution controls or the calling
# environment's MAKEFLAGS could suppress a prerequisite's failure, then
# reconciles the gates that actually completed against the declared list
# above regardless of Make's own exit code. `make ci` alone still does
# neither of those — see the header comment.
.PHONY: guarded-ci
guarded-ci:
	$(CI_GUARD) ci
