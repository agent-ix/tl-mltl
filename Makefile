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
# This file is not a trust root and no longer tries to be one. The parse-time
# guards that used to police Make's own execution controls — MAKEFLAGS,
# PYTHONOPTIMIZE, the rustup and loader overrides, CARGO_TARGET_DIR, the
# CARGO/PYTHON/QUIRE/SHA256SUM/BASH origin checks, and the static
# check_failure_propagation.py inspection — went with the collector they were
# protecting.
#
# Read this before trusting a green `make ci`. Measured in this repository, not
# assumed: with a syntax error introduced into src/lib.rs, `make -k ci` exits 2
# and 10 of the 14 `ci` prerequisites do not complete — fmt-check, lint, test,
# conformance, differential, cli-conformance, test-census, msrv, rustdoc and
# assurance. Adding a single `.IGNORE:` line to this file makes all 10 report
# success and `make ci` exits 0. Nothing here notices.
#
# The structural backstop only goes so far. Quoin binds each retained input by
# digest and the chain derives every attested result from the producer's own
# bytes, so a producer that did not run yields an absent or empty input that the
# chain names and refuses. That covers the seven proofs re-run inside
# `assurance-inputs`. It does not cover fmt-check, lint, check-corpus, deny,
# audit-unsafe, rustdoc, or the `quire validate` half of spec: those feed no
# input and are simply neutered. And under `.IGNORE:` the chain's own refusal is
# suppressed too — the measurement above recorded `assurance-chain` exiting 2 and
# being ignored — so the backstop narrows the class rather than closing it.
#
# Tracked as agent-ix/tl-mltl#14. Do not read the structural replacement as a
# fix for this.

CARGO ?= cargo
PYTHON ?= python3
QUIRE ?= quire
QUOIN ?= quoin

# The shared-assurance lane runs in its own interpreter. Nothing in this
# repository imports jsonschema once the local evidence machinery is gone, so
# there is no version conflict left to resolve; the environment exists because
# engineering-assurance is pinned as a git tag, and resolving a git dependency
# into the system interpreter would make the pin depend on whatever else that
# interpreter happens to have.
ASSURANCE_VENV ?= .venv-assurance
ASSURANCE_PYTHON ?= $(ASSURANCE_VENV)/bin/python

ASSURANCE_DIR := target/assurance
CONFORMANCE_RESULT := $(ASSURANCE_DIR)/reference-conformance.jsonl
DIFFERENTIAL_RESULT := $(ASSURANCE_DIR)/r2u2-differential.jsonl
CLI_RESULT := $(ASSURANCE_DIR)/cli-conformance.jsonl
CENSUS_RESULT := $(ASSURANCE_DIR)/test-census.json
QUIRE_EXPORT := $(ASSURANCE_DIR)/quire-static-export.json
COMPAT_RESULT := $(ASSURANCE_DIR)/legacy-compatibility.json
MSRV_RESULT := $(ASSURANCE_DIR)/msrv.jsonl
REVISION ?= $(shell git rev-parse HEAD)

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test plus the shared-assurance tests"
	@echo "  make check-corpus     - Verify shared and R2U2 corpus bytes"
	@echo "  make conformance      - Replay the shared corpus through the evaluator"
	@echo "  make differential     - Replay the retained R2U2 exchange"
	@echo "  make cli-conformance  - Drive the built CLI over its declared requests"
	@echo "  make test-census      - Bind requirement-tagged tests to compiled tests"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make spec             - Validate specification and coverage with Quire"
	@echo "  make msrv             - Check all targets and features with Rust 1.75"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean and drop the assurance environment"
	@echo "  make assurance-env    - Create the pinned shared-assurance interpreter"
	@echo "  make assurance-inputs - Run the producers and write their structured results"
	@echo "  make pins             - Classify the toolchain through the shared matrix"
	@echo "  make compat-view      - Read retained evidence through the shared mapping"
	@echo "  make assurance-chain  - Seal, retain, and verify through Quoin"
	@echo "  make assurance        - pins + compat-view + assurance-chain"
	@echo "  make ci               - All CI gates locally (hosted CI is manual-only)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets --all-features -- -D warnings

# The traced tests invoke the assurance gates, so the producers must already have
# run. They are a prerequisite rather than something a test creates for itself: a
# test that can produce its own inputs can produce a green run out of nothing.
.PHONY: test
test: assurance-inputs
	$(CARGO) test --all-targets --all-features

# =============================================================================
# MLTL domain
# =============================================================================

.PHONY: check-corpus
check-corpus:
	sha256sum --check corpus/tl-syntax-v1.sha256
	cd corpus/r2u2-v4.2 && sha256sum --check SHA256SUMS

.PHONY: conformance
conformance:
	$(CARGO) run --quiet --example reference_conformance -- \
		--manifest corpus/tl-syntax-v1/manifest.json

.PHONY: differential
differential:
	$(CARGO) run --quiet --example r2u2_differential -- \
		--manifest corpus/r2u2-v4.2/manifest.json

# The example drives the CLI binary, so the binary has to exist. It is built
# here rather than located by the example, because an example that can build its
# own subject can report a green run against a stale one.
.PHONY: cli-conformance
cli-conformance:
	$(CARGO) build --quiet --bin tl-mltl
	$(CARGO) run --quiet --example cli_conformance -- \
		--requests tests/fixtures/cli-requests/manifest.json

.PHONY: test-census
test-census:
	$(PYTHON) scripts/rust_test_census.py

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean
	rm -rf $(ASSURANCE_VENV)

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses
	$(CARGO) deny check sources

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

.PHONY: spec
spec:
	$(QUIRE) validate --scope . 'spec/**/*.md'
	$(QUIRE) coverage --scope . --strict

.PHONY: msrv
msrv:
	rustup run 1.75.0 $(CARGO) check --locked --all-targets --all-features

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features

# =============================================================================
# Shared assurance
# =============================================================================

# Rebuilt when the pin changes. Without this prerequisite, editing the pinned
# release never rebuilds the environment and the toolchain keeps whatever it
# already had.
$(ASSURANCE_PYTHON): requirements-assurance.txt
	rm -rf $(ASSURANCE_VENV)
	$(PYTHON) -m venv $(ASSURANCE_VENV)
	$(ASSURANCE_VENV)/bin/pip install --quiet --disable-pip-version-check \
		-r requirements-assurance.txt

.PHONY: assurance-env
assurance-env: $(ASSURANCE_PYTHON)

# The only target that runs a producer. Everything downstream consumes these
# files and refuses to create them.
.PHONY: assurance-inputs
assurance-inputs: assurance-env
	mkdir -p $(ASSURANCE_DIR)
	$(CARGO) run --quiet --example reference_conformance -- \
		--manifest corpus/tl-syntax-v1/manifest.json > $(CONFORMANCE_RESULT)
	$(CARGO) run --quiet --example r2u2_differential -- \
		--manifest corpus/r2u2-v4.2/manifest.json > $(DIFFERENTIAL_RESULT)
	$(CARGO) build --quiet --bin tl-mltl
	$(CARGO) run --quiet --example cli_conformance -- \
		--requests tests/fixtures/cli-requests/manifest.json > $(CLI_RESULT)
	$(PYTHON) scripts/rust_test_census.py --json > $(CENSUS_RESULT)
	$(QUIRE) coverage --scope . --json > $(QUIRE_EXPORT)
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py --json > $(COMPAT_RESULT)
	rustup run 1.75.0 $(CARGO) check --locked --all-targets --all-features \
		--message-format=json > $(MSRV_RESULT)

.PHONY: pins
pins: assurance-env
	$(ASSURANCE_PYTHON) scripts/check_shared_pins.py

.PHONY: compat-view
compat-view: assurance-env
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py
	$(ASSURANCE_PYTHON) scripts/legacy_evidence_view.py --mutation-probes

.PHONY: assurance-chain
assurance-chain: assurance-inputs
	$(PYTHON) scripts/assurance_chain.py --candidate-revision $(REVISION)

.PHONY: assurance
assurance: pins compat-view assurance-chain

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
ci: fmt-check lint test check-corpus conformance differential cli-conformance \
	test-census deny audit-unsafe spec msrv rustdoc assurance
