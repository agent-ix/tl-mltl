# =============================================================================
# TL MLTL Makefile
# =============================================================================

CARGO ?= cargo

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make check-corpus     - Verify shared and R2U2 corpus bytes"
	@echo "  make verify-evidence  - Verify every retained evidence SHA-256 manifest"
	@echo "  make spec             - Validate and cover specification artifacts"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make evidence-tool    - Syntax-check evidence tooling and schemas"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + audit-unsafe)"

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

.PHONY: test
test:
	$(CARGO) test --all-targets --all-features

.PHONY: check-corpus
check-corpus:
	sha256sum --check corpus/tl-syntax-v1.sha256
	cd corpus/r2u2-v4.2 && sha256sum --check SHA256SUMS

.PHONY: verify-evidence
verify-evidence:
	bash scripts/verify_evidence.sh

.PHONY: spec
spec:
	quire validate --scope . 'spec/**/*.md'
	quire coverage --scope . --strict

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings $(CARGO) doc --no-deps --all-features

.PHONY: evidence-tool
evidence-tool:
	python3 -m py_compile scripts/build_evidence_envelope.py scripts/finalize_collection.py scripts/test_evidence_tool.py scripts/validate_json_schema.py
	python3 scripts/test_evidence_tool.py

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

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

.PHONY: audit-unsafe
audit-unsafe:
	bash scripts/check_unsafe_comments.sh

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test check-corpus deny audit-unsafe evidence-tool spec rustdoc verify-evidence
