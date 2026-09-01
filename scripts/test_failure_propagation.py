#!/usr/bin/env python3
"""Behavior tests for local-CI failure propagation policy."""

from __future__ import annotations

import hashlib
import os
import subprocess
import tempfile
from pathlib import Path

import check_failure_propagation as policy
import tool_identity


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def inspect_text(text: str) -> list[str]:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "Makefile"
        path.write_text(text, encoding="utf-8")
        return policy.inspect(path, Path(directory))


def main() -> int:
    makefile = (policy.ROOT / "Makefile").read_text(encoding="utf-8")
    expect(not policy.inspect(policy.ROOT / "Makefile"), "current Makefile must pass static inspection")
    expect(policy.makeflags_ignore_errors("-i"), "short ignore-errors flag must be rejected")
    expect(policy.makeflags_ignore_errors("--ignore-errors"), "long ignore-errors flag must be rejected")
    expect(not policy.makeflags_ignore_errors("--no-print-directory"), "safe flag was rejected")
    mutated = makefile.replace("\tcargo test --all-targets --all-features", "\t-cargo test --all-targets --all-features", 1)
    expect(any("ignores a recipe failure" in item for item in inspect_text(mutated)), "ignored recipe failure was missed")
    mutated = makefile.replace("ci:", ".IGNORE:\n\nci:", 1)
    expect(any("global recipe-control" in item for item in inspect_text(mutated)), "global ignore was missed")
    for directive in (".ONESHELL:", ".DEFAULT:"):
        mutated = makefile.replace("ci:", f"{directive}\n\nci:", 1)
        expect(
            any("global recipe-control" in item for item in inspect_text(mutated)),
            f"global directive was missed: {directive}",
        )
    for assignment in (
        "SHELL := /usr/bin/true",
        "SHELL != printf /usr/bin/true",
        ".SHELLFLAGS := -c true",
    ):
        mutated = makefile.replace("ci:", f"{assignment}\n\nci:", 1)
        expect(
            any("mandatory recipe shell" in item for item in inspect_text(mutated)),
            f"shell assignment was missed: {assignment}",
        )
    for suffix in (" || true", " ; true", " &", " | true", " ; set +e"):
        mutated = makefile.replace(
            "\tcargo test --all-targets --all-features",
            "\tcargo test --all-targets --all-features" + suffix,
            1,
        )
        expect(
            any("forbidden shell control" in item for item in inspect_text(mutated)),
            f"shell failure-control route was missed: {suffix}",
        )

    base_env = dict(os.environ)
    base_env.pop("MAKEFLAGS", None)
    base_env.pop("MAKELEVEL", None)
    for name, value in (
        ("CARGO", "true"),
        ("CARGO_TARGET_DIR", "/tmp/unqualified-target"),
        ("PYTHONOPTIMIZE", "1"),
        ("MAKEFLAGS", "-i"),
    ):
        attacked = dict(base_env)
        attacked[name] = value
        assignment = "MAKEFLAGS=-i" if name == "MAKEFLAGS" else "MAKEFLAGS="
        result = subprocess.run(
            ["/usr/bin/make", "--no-print-directory", "-n", assignment, "ci"],
            cwd=policy.ROOT,
            env=attacked,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        expect(result.returncode != 0, f"ambient {name} attack was accepted")
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        python = Path("/usr/bin/python3")
        digest = hashlib.sha256(python.read_bytes()).hexdigest()
        tools = {}
        qualified = root / "qualified"
        qualified.mkdir()
        for tool_name in tool_identity.REQUIRED:
            path = qualified / tool_name
            path.symlink_to(python)
            tools[tool_name] = {"path": str(path), "sha256": digest}
        value = {
            "schemaVersion": "tl-mltl.qualified-tools/v1",
            "environment": {"home": "/home/peter", "cargoTargetDir": str(policy.ROOT / ".qualification-target")},
            "tools": tools,
        }
        validated = tool_identity.validate_lock(value)
        shim_dir = root / "attack"
        shim_dir.mkdir()
        shim = shim_dir / "cargo"
        shim.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        shim.chmod(0o755)
        unavailable, mismatches = tool_identity.verify_live(
            value, validated, search_path=f"{shim_dir}:{qualified}"
        )
        expect(not unavailable and bool(mismatches), "ambient PATH shim escaped qualification")
    print("failure propagation behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
