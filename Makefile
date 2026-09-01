# =============================================================================
# TL MLTL Makefile
# =============================================================================

ifneq ($(filter ci ci-for-evidence,$(MAKECMDGOALS)),)
ifneq ($(strip $(MAKEFLAGS)),)
$(error local CI refuses non-empty MAKEFLAGS)
endif
ifneq ($(strip $(PYTHONOPTIMIZE)),)
$(error local CI refuses optimized Python policy execution)
endif
ifneq ($(strip $(RUSTUP_TOOLCHAIN)$(RUSTUP_HOME)$(CARGO_HOME)$(RUSTC)$(RUSTDOC)$(RUSTC_WRAPPER)$(RUSTC_WORKSPACE_WRAPPER)$(RUSTFLAGS)$(CARGO_ENCODED_RUSTFLAGS)$(RUSTDOCFLAGS)$(LD_PRELOAD)$(LD_LIBRARY_PATH)$(PYTHONPATH)),)
$(error local CI refuses ambient compiler, loader, or Python-path overrides)
endif
ifneq ($(origin CARGO),undefined)
$(error local CI refuses a CARGO override)
endif
ifneq ($(origin PYTHON),undefined)
$(error local CI refuses a PYTHON override)
endif
ifneq ($(origin QUIRE),undefined)
$(error local CI refuses a QUIRE override)
endif
ifneq ($(origin SHA256SUM),undefined)
$(error local CI refuses a SHA256SUM override)
endif
ifneq ($(origin BASH),undefined)
$(error local CI refuses a BASH override)
endif
tl_ci_static_status := $(shell /usr/bin/env -u PYTHONOPTIMIZE MAKEFLAGS= /usr/bin/python3 scripts/check_failure_propagation.py --makefile '$(firstword $(MAKEFILE_LIST))' --static-only >/dev/null; echo $$?)
ifneq ($(tl_ci_static_status),0)
$(error local CI refuses unsafe Make recipe controls)
endif
endif

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make check-failure-propagation - prove required command failures reach CI"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses and sources"
	@echo "  make audit-unsafe     - Enforce // SAFETY: comments on unsafe blocks"
	@echo "  make check-corpus     - Verify shared and R2U2 corpus bytes"
	@echo "  make verify-evidence  - Verify every retained evidence SHA-256 manifest"
	@echo "  make spec             - Validate and cover specification artifacts"
	@echo "  make rustdoc          - Build warning-free public documentation"
	@echo "  make evidence-tool    - Syntax-check evidence tooling and schemas"
	@echo "  make ci-for-evidence  - Candidate gates before evidence can self-anchor"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + audit-unsafe)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	cargo fmt --all

.PHONY: fmt-check
fmt-check:
	cargo fmt --all -- --check
	@/usr/bin/printf 'fmt-check gate passed\n'

.PHONY: lint
lint:
	cargo clippy --all-targets --all-features -- -D warnings
	@/usr/bin/printf 'lint gate passed\n'

.PHONY: test
test:
	cargo test --all-targets --all-features
	@/usr/bin/printf 'Rust test gate passed\n'

.PHONY: check-failure-propagation
check-failure-propagation:
	/usr/bin/python3 scripts/check_failure_propagation.py

.PHONY: check-tool-identities
check-tool-identities:
	/usr/bin/python3 scripts/tool_identity.py --verify-live
	@/usr/bin/printf 'qualified tool identities passed\n'

.PHONY: check-corpus
check-corpus:
	sha256sum --check corpus/tl-syntax-v1.sha256
	cd corpus/r2u2-v4.2 && sha256sum --check SHA256SUMS
	@/usr/bin/printf 'corpus-integrity gate passed\n'

.PHONY: verify-evidence
verify-evidence:
	/usr/bin/bash scripts/verify_evidence.sh
	@/usr/bin/printf 'verify-evidence gate passed\n'

.PHONY: spec
spec:
	quire validate --scope . 'spec/**/*.md'
	quire coverage --scope . --strict
	@/usr/bin/printf 'spec gate passed\n'

.PHONY: rustdoc
rustdoc:
	RUSTDOCFLAGS=-Dwarnings cargo doc --no-deps --all-features
	@/usr/bin/printf 'rustdoc gate passed\n'

.PHONY: evidence-tool
evidence-tool:
	/usr/bin/python3 -m compileall -q scripts
	/usr/bin/python3 scripts/run_policy_tests.py
	@/usr/bin/printf 'evidence-tool gate passed\n'

.PHONY: build
build:
	cargo build --release

.PHONY: clean
clean:
	cargo clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	cargo deny check licenses
	cargo deny check sources
	@/usr/bin/printf 'deny gate passed\n'

.PHONY: cargo-audit
cargo-audit:
	cargo audit

.PHONY: audit-unsafe
audit-unsafe:
	/usr/bin/bash scripts/check_unsafe_comments.sh
	@/usr/bin/printf 'audit-unsafe gate passed\n'

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci ci-for-evidence
ci-for-evidence: fmt-check lint test check-corpus deny audit-unsafe evidence-tool spec rustdoc check-failure-propagation check-tool-identities
	@/usr/bin/printf 'candidate CI gate passed\n'

ci: fmt-check lint test check-corpus deny audit-unsafe evidence-tool spec rustdoc verify-evidence check-failure-propagation
	@/usr/bin/printf 'full local CI gate passed\n'
