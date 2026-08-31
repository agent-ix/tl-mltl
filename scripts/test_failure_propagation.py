#!/usr/bin/env python3
"""Behavior tests for local-CI failure propagation policy."""

from __future__ import annotations

import os
import subprocess
import tempfile
from pathlib import Path

import check_failure_propagation as policy


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

    base_env = dict(os.environ)
    base_env.pop("MAKEFLAGS", None)
    base_env.pop("MAKELEVEL", None)
    for name, value in (("CARGO", "true"), ("PYTHONOPTIMIZE", "1"), ("MAKEFLAGS", "-i")):
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
        shim = Path(directory) / "cargo"
        shim.write_text("#!/bin/sh\nexit 0\n", encoding="utf-8")
        shim.chmod(0o755)
        attacked = dict(base_env)
        attacked["PATH"] = f"{directory}:{base_env['PATH']}"
        result = subprocess.run(
            ["/usr/bin/make", "--no-print-directory", "-n", "MAKEFLAGS=", "ci"],
            cwd=policy.ROOT,
            env=attacked,
            check=False,
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        expect(result.returncode != 0, "ambient PATH shim attack was accepted")
    print("failure propagation behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
