# =============================================================================
# TL MLTL Makefile
# =============================================================================
#
# Native orchestration. Every target calls the toolchain that owns the job:
# cargo for the crate, the reference-conformance replay and the R2U2
# differential replay for the semantics, quire for static export, quoin for
# evidence. Nothing here computes a verdict, attests to its own correctness, or
# retains evidence of its own.
#
# This file is not a trust root and does not try to be one. The parse-time
# guards that used to police Make's own execution controls — MAKEFLAGS,
# PYTHONOPTIMIZE, the rustup and loader overrides, CARGO_TARGET_DIR, the
# CARGO/PYTHON/QUIRE/SHA256SUM/BASH origin checks, and the static
# check_failure_propagation.py inspection — went with the collector they were
# protecting.
#
# `make ci` alone still trusts Make's own execution controls and Make's own
# exit code, exactly as measured below. Nothing in this Makefile changed that.
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
# Read this before trusting a bare `make ci`. Measured in this repository, not
# assumed: with a syntax error introduced into src/lib.rs, `make -k ci` exits 2
# and 12 of the 15 `ci` prerequisites do not complete — fmt-check, lint,
# kani-check, test, conformance, differential, cli-conformance, test-census,
# spec, msrv, rustdoc and assurance. Adding a single `.IGNORE:` line to this
# file makes all 12 report success and `make ci` exits 0. `make guarded-ci`
# against the same `.IGNORE:`-prepended file refuses before Make ever runs.
#
# The structural backstop only goes so far. Quoin binds each retained input by
# digest and the chain derives every attested result from the producer's own
# bytes, so a producer that did not run yields an absent or empty input that the
# chain names and refuses. That covers the six proofs re-run inside
# `assurance-inputs`. It does not cover fmt-check, lint, check-corpus, deny,
# audit-unsafe, rustdoc, or the `quire validate` half of spec: those feed no
# input and are simply neutered. And under `.IGNORE:` the chain's own refusal is
# suppressed too — the measurement above recorded `assurance-chain` exiting 2 and
# being ignored — `make guarded-ci`'s completion-record reconciliation covers
# exactly that residue.
#
# Tracked as agent-ix/tl-mltl#14 (Linear TL-65), remediated by
# NFR-006-gate-set-integrity. Do not read the structural backstop above as a
# fix for this on its own.

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
QUOIN ?= quoin

# `ci_guard record` is the last step of every `ci` prerequisite's recipe, so
# it only runs on that recipe's own success. Run alone (e.g. `make lint` for
# local iteration, outside `make guarded-ci`), it is a deliberate no-op.
CI_GUARD ?= $(CARGO) run --quiet --bin ci_guard --

# The shared-assurance script asks the exact tagged native Engineering
# Assurance CLI for its verdict. It has no Python package dependency.

ASSURANCE_DIR := target/assurance
CONFORMANCE_RESULT := $(ASSURANCE_DIR)/reference-conformance.jsonl
DIFFERENTIAL_RESULT := $(ASSURANCE_DIR)/r2u2-differential.jsonl
CLI_RESULT := $(ASSURANCE_DIR)/cli-conformance.jsonl
CENSUS_RESULT := $(ASSURANCE_DIR)/test-census.json
QUIRE_EXPORT := $(ASSURANCE_DIR)/quire-static-export.json
MSRV_RESULT := $(ASSURANCE_DIR)/msrv.jsonl
SHARED_CORPUS_MANIFEST_COPY := $(ASSURANCE_DIR)/shared-corpus-manifest.json
REVISION ?= $(shell git rev-parse HEAD)

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test plus the shared-assurance tests"
	@echo "  make kani-check       - Verify the bounded Kani horizon proof"
	@echo "  make check-corpus     - Verify R2U2 corpus bytes"
	@echo "  make conformance      - Replay the shared corpus through the evaluator"
	@echo "  make differential     - Replay the retained R2U2 exchange"
	@echo "  make cli-conformance  - Drive the built CLI over its declared requests"
	@echo "  make test-census      - Bind requirement-tagged tests to compiled tests"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make msrv             - Check all targets and features with Rust 1.98.1"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean and drop the assurance environment"
	@echo "  make assurance-inputs - Write the structured results the assurance chain reads"
	@echo "  make pins             - Classify the toolchain through the shared matrix"
	@echo "  make assurance-chain  - Seal, retain, and verify through Quoin"
	@echo "  make assurance        - pins + assurance-chain"
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

# The traced tests invoke the assurance gates, so the producers must already have
# run. They are a prerequisite rather than something a test creates for itself: a
# test that can produce its own inputs can produce a green run out of nothing.
.PHONY: test
test: assurance-inputs
	$(CARGO) test --all-targets --all-features
	$(CI_GUARD) record test

# =============================================================================
# MLTL domain
# =============================================================================

.PHONY: check-corpus
check-corpus:
	cd corpus/r2u2-v4.2 && sha256sum --check SHA256SUMS
	$(CI_GUARD) record check-corpus

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

.PHONY: test-census
test-census:
	$(PYTHON) scripts/rust_test_census.py
	$(CI_GUARD) record test-census

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

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh
	$(CI_GUARD) record audit-unsafe

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
# Shared assurance
# =============================================================================

# The only target that WRITES THE CHAIN'S INPUTS. `conformance`,
# `differential`, `cli-conformance`, `test-census` and `spec` run producers
# too — they are the same producers, run as ordinary gates. What is unique
# here is that these are the bytes the chain reads, and everything
# downstream consumes them and refuses to create them.
.PHONY: assurance-inputs
assurance-inputs:
	mkdir -p $(ASSURANCE_DIR)
	$(CARGO) run --quiet --example emit_shared_corpus_manifest > $(SHARED_CORPUS_MANIFEST_COPY)
	$(CARGO) run --quiet --example reference_conformance > $(CONFORMANCE_RESULT)
	$(CARGO) run --quiet --example r2u2_differential -- \
		--manifest corpus/r2u2-v4.2/manifest.json > $(DIFFERENTIAL_RESULT)
	$(CARGO) build --quiet --bin tl-mltl
	$(CARGO) run --quiet --example cli_conformance -- \
		--requests tests/fixtures/cli-requests/manifest.json > $(CLI_RESULT)
	$(PYTHON) scripts/rust_test_census.py --json > $(CENSUS_RESULT)
	$(QUIRE) coverage --scope . --json > $(QUIRE_EXPORT)
	rustup run 1.98.1 $(CARGO) check --locked --all-targets --all-features \
		--message-format=json > $(MSRV_RESULT)

.PHONY: pins
pins:
	$(PYTHON) scripts/check_shared_pins.py

.PHONY: assurance-chain
assurance-chain: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --candidate-revision $(REVISION)

.PHONY: assurance
assurance: pins assurance-chain
	$(CI_GUARD) record assurance

# An operator target, not a CI gate. It writes into this repository's own Quoin
# evidence store, which is a reviewed change to spec/evidence/ rather than
# something a gate should do on every run.
.PHONY: assurance-record
assurance-record: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --adapt $(DIFFERENTIAL_RESULT) \
		> $(ASSURANCE_DIR)/entries.json
	$(QUOIN) evidence record \
		--repo . \
		--suite SUITE-001 \
		--commit $(REVISION) \
		--tool "tl-mltl-r2u2-differential 0.1.0" \
		--adapter entries \
		--kind Integration \
		--results $(ASSURANCE_DIR)/entries.json

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint kani-check test check-corpus conformance differential cli-conformance \
	test-census deny audit-unsafe spec msrv rustdoc assurance

# The assured entry point (NFR-006). Builds and runs the guard, which refuses
# to invoke `make ci` at all if this file's execution controls or the calling
# environment's MAKEFLAGS could suppress a prerequisite's failure, then
# reconciles the gates that actually completed against the declared list
# above regardless of Make's own exit code. `make ci` alone still does
# neither of those — see the header comment.
.PHONY: guarded-ci
guarded-ci:
	$(CI_GUARD) ci
