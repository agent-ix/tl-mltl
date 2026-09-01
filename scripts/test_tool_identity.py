#!/usr/bin/env python3
"""Behavior tests for the qualified executable lock."""

from __future__ import annotations

import hashlib
import subprocess
import tempfile
from pathlib import Path

import tool_identity


def expect(condition: bool, message: str) -> None:
    if not condition:
        raise RuntimeError(message)


def main() -> int:
    lock_value, locked_tools = tool_identity.load_lock()
    expect(set(locked_tools) == set(tool_identity.REQUIRED), "mandatory tool census drifted")

    # Exercise the collector's exact clean-environment CLI route. The lookup
    # reads the source lock but need not possess the qualification host's paths.
    cli = subprocess.run(
        ["/usr/bin/python3", "scripts/tool_identity.py", "--tool-path", "cargo"],
        cwd=tool_identity.ROOT,
        env={"HOME": "/tmp", "PATH": "/usr/bin", "LANG": "C.UTF-8", "LC_ALL": "C.UTF-8"},
        check=False,
        capture_output=True,
        text=True,
    )
    expect(cli.returncode == 0, "clean-environment tool lookup was unreachable")
    expect(cli.stdout.strip() == locked_tools["cargo"]["path"], "tool lookup returned the wrong path")

    # Prove live verification without depending on the checked-in host profile.
    python = Path("/usr/bin/python3")
    digest = hashlib.sha256(python.read_bytes()).hexdigest()
    with tempfile.TemporaryDirectory(prefix="tl-mltl-tools-") as directory:
        root = Path(directory)
        tools: dict[str, dict[str, str]] = {}
        for name in tool_identity.REQUIRED:
            path = root / name
            path.symlink_to(python)
            tools[name] = {"path": str(path), "sha256": digest}
        value = {
            "schemaVersion": "tl-mltl.qualified-tools/v1",
            "environment": {"home": "/qualified", "cargoTargetDir": "/qualified/target"},
            "tools": tools,
        }
        validated = tool_identity.validate_lock(value)
        unavailable, mismatches = tool_identity.verify_live(value, validated)
        expect(not unavailable and not mismatches, "synthetic qualified tool identities disagreed")
        value["tools"]["cargo"]["sha256"] = "0" * 64
        changed = tool_identity.validate_lock(value)
        _, changed_mismatches = tool_identity.verify_live(value, changed)
        expect(any("cargo" in item for item in changed_mismatches), "digest mutation was not detected")
    print("qualified tool identity behavior is valid")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
