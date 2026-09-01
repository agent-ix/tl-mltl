#!/usr/bin/env python3
"""Behavior tests for local-CI failure propagation policy."""

from __future__ import annotations

import hashlib
import os
import subprocess
import sys
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
    for value in ("i", "ik", "-i", "--ignore-errors", "-t", "-n", "--eval=.IGNORE:"):
        expect(policy.makeflags_ignore_errors(value), f"MAKEFLAGS={value!r} escaped inspection")
    for value in ("-j4", "--jobs=4 --jobserver-auth=3,4", "-l2 -Otarget", "-w"):
        expect(not policy.makeflags_ignore_errors(value), f"safe MAKEFLAGS={value!r} was rejected")

    mutations = [
        makefile.replace(
            "\tcargo test --all-targets --all-features",
            "\t-cargo test --all-targets --all-features",
            1,
        ),
        makefile.replace(
            "\tcargo test --all-targets --all-features",
            "\tcargo test --all-targets --all-features || true",
            1,
        ),
        makefile + "\n.IGNORE:\n",
        makefile + "\n.SILENT:\n",
        makefile + "\n.ONESHELL:\n",
        makefile + "\n.DEFAULT:\n",
        makefile + "\nSHELL := /usr/bin/true\n",
        makefile + "\n.SHELLFLAGS != printf '%s' '-c true'\n",
        makefile + "\nMAKE ::= /usr/bin/true\n",
        makefile + "\nprivate MAKE :::= /usr/bin/true\n",
        makefile + "\ndefine MAKEFLAGS\n-i\nendef\n",
        makefile + "\noverride define SHELL\n/usr/bin/true\nendef\n",
        makefile + "\n$(eval MAKEFLAGS := -i)\n",
        makefile + "\n${eval SHELL := /usr/bin/true}\n",
        makefile + "\ninclude imported-controls.mk\n",
        makefile + "\n-include optional-controls.mk\n",
        makefile + "\nsinclude optional-controls.mk\n",
        makefile + "\ntest: SHELL := /usr/bin/true\n",
        makefile + "\ntest lint: .SHELLFLAGS ::= -c true\n",
        makefile + "\n%.policy: private MAKE :::= /usr/bin/true\n",
        makefile + "\npolicy-targets: %.policy: override MAKEFLAGS += -i\n",
    ]
    target_scoped_start = len(mutations) - 4
    for operator in ("=", ":=", "::=", ":::=", "+=", "?=", "!="):
        mutations.append(makefile + f"\nexport override MAKEFLAGS {operator} -i\n")
    with tempfile.TemporaryDirectory() as directory:
        root = Path(directory)
        for index, mutated in enumerate(mutations):
            path = root / f"Makefile.{index}"
            path.write_text(mutated, encoding="utf-8")
            inspection = policy.inspect(path, root)
            expect(bool(inspection), f"Make execution-control mutation {index} escaped inspection")
            if target_scoped_start <= index < target_scoped_start + 4:
                expect(
                    any("target-scoped execution control" in item for item in inspection),
                    f"scoped mutation {index} was rejected for the wrong reason: {inspection}",
                )
            result = subprocess.run(
                [
                    sys.executable,
                    str(policy.ROOT / "scripts" / "check_failure_propagation.py"),
                    "--makefile",
                    str(path),
                    "--static-only",
                ],
                cwd=policy.ROOT,
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            expect(result.returncode != 0, f"checker exit contract accepted mutation {index}")

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

    with tempfile.TemporaryDirectory() as directory:
        behavior = Path(directory)
        multi_target = behavior / "multi-target.mk"
        multi_target.write_text(
            ".PHONY: ci ci-for-evidence\n"
            "ci ci-for-evidence:\n\tfalse\n"
            "ci ci-for-evidence: SHELL := /usr/bin/true\n",
            encoding="utf-8",
        )
        pattern = behavior / "pattern.mk"
        pattern.write_text(
            ".PHONY: ci\nci:\n\tfalse\n%: SHELL := /usr/bin/true\n",
            encoding="utf-8",
        )
        imported = behavior / "imported.mk"
        (behavior / "controls.mk").write_text(
            "SHELL := /usr/bin/true\n", encoding="utf-8"
        )
        imported.write_text(
            "include controls.mk\n.PHONY: ci\nci:\n\tfalse\n", encoding="utf-8"
        )
        for path in (multi_target, pattern, imported):
            result = subprocess.run(
                ["/usr/bin/make", "--no-print-directory", "-f", str(path), "ci"],
                cwd=behavior,
                check=False,
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            expect(
                result.returncode == 0,
                f"GNU Make fixture did not demonstrate swallowed execution: {path.name}",
            )
            expect(
                bool(policy.inspect_execution_controls(path.read_text(encoding="utf-8"))),
                f"behavioral control escaped source inspection: {path.name}",
            )

        hidden = behavior / "hidden-expanded.mk"
        hidden.write_text(
            makefile.replace(
                "cargo test --all-targets --all-features",
                "$(POLICY_TEST)",
                1,
            )
            + "\nPOLICY_TEST = cargo test --all-targets --all-features || true\n",
            encoding="utf-8",
        )
        expect(
            bool(policy.inspect_expanded_recipes(hidden, policy.ROOT)),
            "expanded recipe control escaped inspection",
        )
        expect(
            bool(policy.probe_command_positions(multi_target)),
            "command-position probe accepted a controlled Makefile",
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
