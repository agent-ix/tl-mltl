#!/usr/bin/env python3
"""Define the single source-file census used by parameter digests."""

from __future__ import annotations

from pathlib import Path


FIXED = {
    "Cargo.toml",
    "Cargo.lock",
    "build.rs",
    "Makefile",
    "deny.toml",
    "rust-toolchain.toml",
    ".github/workflows/ci.yml",
    "tools.lock",
    "evidence/RETRACTIONS.json",
    "corpus/tl-syntax-v1.sha256",
    "corpus/r2u2-v4.2/SHA256SUMS",
    "corpus/r2u2-v4.2/manifest.json",
    "corpus/r2u2-v4.2/differential-report.json",
    "schemas/tl-mltl-evidence-input-v1.schema.json",
    "schemas/tl-mltl-evidence-manifest-v1.schema.json",
}


def parameter_names(tree: set[str]) -> list[str]:
    missing = FIXED - tree
    if missing:
        raise OSError(f"source revision lacks parameter files: {sorted(missing)}")
    dynamic = {
        path
        for path in tree
        if ((path.startswith("src/") or path.startswith("tests/")) and path.endswith(".rs"))
        or (path.startswith("scripts/") and Path(path).suffix in {".py", ".sh"})
    }
    return sorted(FIXED | dynamic)
