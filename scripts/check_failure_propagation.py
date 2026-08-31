#!/usr/bin/env python3
"""Prove every mandatory local-CI recipe propagates command failures."""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

import tool_identity


ROOT = Path(__file__).resolve().parent.parent
PROBES = {
    "fmt-check", "lint", "test", "check-corpus", "deny", "audit-unsafe",
    "evidence-tool", "spec", "rustdoc", "verify-evidence",
}
COLLECTION_PROBES = PROBES - {"verify-evidence"}
GUARD_TARGET = "check-failure-propagation"
TARGET = re.compile(r"^([A-Za-z0-9_.-]+):(?:\s+(.*?))?\s*$")
IGNORE_ATTRIBUTE = re.compile(r"#\s*\[[^\]]*\bignore\b[^\]]*\]")
MAKEFLAGS_ASSIGNMENT = re.compile(
    r"^\s*(?:(?:export|override|unexport)\s+)*MAKEFLAGS\s*(?::|\+|\?)?=\s*(.*)$"
)


def parse_makefile(text: str) -> tuple[dict[str, list[str]], dict[str, list[str]]]:
    dependencies: dict[str, list[str]] = {}
    recipes: dict[str, list[str]] = {}
    current: str | None = None
    for line in text.splitlines():
        if line.startswith("\t"):
            if current is not None:
                recipes.setdefault(current, []).append(line[1:])
            continue
        current = None
        if not line or line[0].isspace() or line.startswith("#"):
            continue
        match = TARGET.fullmatch(line)
        if match is None or match.group(1).startswith("."):
            continue
        current = match.group(1)
        dependencies[current] = (match.group(2) or "").split()
    return dependencies, recipes


def makeflags_ignore_errors(value: str) -> bool:
    try:
        tokens = shlex.split(value)
    except ValueError:
        return True
    return any(
        token == "--ignore-errors"
        or (token.startswith("-") and not token.startswith("--") and "i" in token[1:])
        or (token and not token.startswith("-") and "=" not in token and "i" in token)
        for token in tokens
    )


def command_parts(command: str) -> tuple[str, str]:
    stripped = command.lstrip()
    modifiers = ""
    while stripped[:1] in {"@", "+", "-"}:
        modifiers += stripped[0]
        stripped = stripped[1:].lstrip()
    return modifiers, stripped


def inspect(makefile: Path, root: Path = ROOT) -> list[str]:
    text = makefile.read_text(encoding="utf-8")
    dependencies, recipes = parse_makefile(text)
    errors: list[str] = []
    required = PROBES | {GUARD_TARGET}
    observed = set(dependencies.get("ci", []))
    if observed != required:
        errors.append(
            "ci prerequisite census drift: "
            f"missing={sorted(required - observed)}, extra={sorted(observed - required)}"
        )
    candidate_required = COLLECTION_PROBES | {GUARD_TARGET}
    candidate_observed = set(dependencies.get("ci-for-evidence", []))
    if candidate_observed != candidate_required:
        errors.append(
            "candidate CI prerequisite census drift: "
            f"missing={sorted(candidate_required - candidate_observed)}, "
            f"extra={sorted(candidate_observed - candidate_required)}"
        )
    for number, line in enumerate(text.splitlines(), start=1):
        if re.match(r"^\s*\.(?:IGNORE|SILENT)\s*(?::|$)", line):
            errors.append(f"Makefile:{number} declares a global recipe-control directive")
        assignment = MAKEFLAGS_ASSIGNMENT.match(line)
        if assignment is not None and makeflags_ignore_errors(assignment.group(1)):
            errors.append(f"Makefile:{number} enables MAKEFLAGS ignore-errors")
    for target in sorted(required):
        commands = recipes.get(target, [])
        if not commands:
            errors.append(f"mandatory target {target} has no recipe")
            continue
        for command in commands:
            modifiers, _ = command_parts(command)
            if "-" in modifiers:
                errors.append(f"mandatory target {target} ignores a recipe failure: {command}")
    for source in root.rglob("*.rs"):
        if ".git" in source.parts or "target" in source.parts:
            continue
        if IGNORE_ATTRIBUTE.search(source.read_text(encoding="utf-8")):
            errors.append(f"{source.relative_to(root)} disables a Rust test with #[ignore]")
    return errors


def probe_command_positions(makefile: Path) -> list[str]:
    """Substitute false at every mandatory recipe position and require Make to fail."""
    _, recipes = parse_makefile(makefile.read_text(encoding="utf-8"))
    errors: list[str] = []
    make = shutil.which("make")
    if make != "/usr/bin/make":
        return [f"Make must resolve to /usr/bin/make, got {make}"]
    clean_env = dict(os.environ)
    clean_env.pop("MAKEFLAGS", None)
    with tempfile.TemporaryDirectory() as directory:
        probe = Path(directory) / "Makefile"
        for target in sorted(PROBES):
            commands = recipes.get(target, [])
            for selected in range(len(commands)):
                lines = [f".PHONY: {target}", f"{target}:"]
                for index, command in enumerate(commands):
                    modifiers, _ = command_parts(command)
                    lines.append(f"\t{modifiers}{'false' if index == selected else 'true'}")
                probe.write_text("\n".join(lines) + "\n", encoding="utf-8")
                result = subprocess.run(
                    [make, "--no-print-directory", "-f", str(probe), target],
                    check=False,
                    stdout=subprocess.DEVNULL,
                    stderr=subprocess.DEVNULL,
                    env=clean_env,
                )
                if result.returncode == 0:
                    errors.append(
                        f"mandatory target {target} swallowed failure at recipe position {selected + 1}"
                    )
    return errors


def inspect_toolchain() -> list[str]:
    try:
        value, tools = tool_identity.load_lock()
    except (OSError, ValueError, json.JSONDecodeError) as error:
        return [f"qualified tool lock is unavailable: {error}"]
    unavailable, mismatches = tool_identity.verify_live(value, tools)
    return unavailable + mismatches


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--makefile", type=Path, default=ROOT / "Makefile")
    parser.add_argument("--inspect-only", action="store_true")
    parser.add_argument("--static-only", action="store_true")
    args = parser.parse_args()
    errors = inspect(args.makefile)
    if os.environ.get("MAKEFLAGS"):
        errors.append("ambient MAKEFLAGS is not permitted for local CI")
    if os.environ.get("MAKE"):
        errors.append("ambient MAKE override is not permitted")
    if os.environ.get("PYTHONOPTIMIZE") or sys.flags.optimize:
        errors.append("optimized Python disables policy checks")
    if not args.static_only:
        errors.extend(inspect_toolchain())
    if not args.inspect_only and not args.static_only and not errors:
        errors.extend(probe_command_positions(args.makefile))
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print(f"all {len(PROBES)} mandatory local-CI targets propagate failures")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
