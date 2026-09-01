#!/usr/bin/env python3
"""Prove every mandatory local-CI recipe propagates command failures."""

from __future__ import annotations

import argparse
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBES = {
    "fmt-check", "lint", "test", "check-corpus", "deny", "audit-unsafe",
    "evidence-tool", "spec", "rustdoc", "verify-evidence", "rust-test-census",
}
COLLECTION_PROBES = PROBES - {"verify-evidence"}
QUALIFICATION_TARGET = "check-tool-identities"
GUARD_TARGET = "check-failure-propagation"
TARGET = re.compile(r"^([A-Za-z0-9_.-]+):(?:\s+(.*?))?\s*$")
IGNORE_ATTRIBUTE = re.compile(r"#\s*\[[^\]]*\bignore\b[^\]]*\]")
DISABLED_CRATE_OR_MODULE = re.compile(r"#!\s*\[\s*cfg\s*\([^\]]*\)\s*\]")
MAKEFLAGS_ASSIGNMENT = re.compile(
    r"^\s*(?:(?:export|override|unexport|private)\s+)*MAKEFLAGS\s*[:+?!]*=\s*(.*)$"
)
SHELL_ASSIGNMENT = re.compile(
    r"^\s*(?:(?:export|override|unexport|private)\s+)*(?:SHELL|\.SHELLFLAGS)\s*[:+?!]*="
)
FORBIDDEN_SHELL_CONTROL = re.compile(r"\|\||(?<!&)\&(?!&)|(?<!\|)\|(?!\|)|;|(?:^|\s)set\s+\+e(?:\s|$)")


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
    ci_required = PROBES | {GUARD_TARGET}
    required = ci_required | {QUALIFICATION_TARGET}
    observed = set(dependencies.get("ci", []))
    if observed != ci_required:
        errors.append(
            "ci prerequisite census drift: "
            f"missing={sorted(ci_required - observed)}, extra={sorted(observed - ci_required)}"
        )
    candidate_required = COLLECTION_PROBES | {GUARD_TARGET, QUALIFICATION_TARGET}
    candidate_observed = set(dependencies.get("ci-for-evidence", []))
    if candidate_observed != candidate_required:
        errors.append(
            "candidate CI prerequisite census drift: "
            f"missing={sorted(candidate_required - candidate_observed)}, "
            f"extra={sorted(candidate_observed - candidate_required)}"
        )
    for number, line in enumerate(text.splitlines(), start=1):
        if re.match(r"^\s*\.(?:IGNORE|SILENT|ONESHELL|DEFAULT)\s*(?::|$)", line):
            errors.append(f"Makefile:{number} declares a global recipe-control directive")
        if SHELL_ASSIGNMENT.match(line):
            errors.append(f"Makefile:{number} overrides the mandatory recipe shell")
        assignment = MAKEFLAGS_ASSIGNMENT.match(line)
        if assignment is not None and makeflags_ignore_errors(assignment.group(1)):
            errors.append(f"Makefile:{number} enables MAKEFLAGS ignore-errors")
    for target in sorted(required):
        commands = recipes.get(target, [])
        if not commands:
            errors.append(f"mandatory target {target} has no recipe")
            continue
        for command in commands:
            modifiers, executable = command_parts(command)
            if "-" in modifiers:
                errors.append(f"mandatory target {target} ignores a recipe failure: {command}")
            if FORBIDDEN_SHELL_CONTROL.search(executable):
                errors.append(
                    f"mandatory target {target} uses a forbidden shell control operator: {command}"
                )
    for source in root.rglob("*.rs"):
        if any(part in {".git", "target", ".qualification-target"} for part in source.parts):
            continue
        source_text = source.read_text(encoding="utf-8")
        if IGNORE_ATTRIBUTE.search(source_text):
            errors.append(f"{source.relative_to(root)} disables a Rust test with #[ignore]")
        if DISABLED_CRATE_OR_MODULE.search(source_text):
            errors.append(f"{source.relative_to(root)} has a crate/module-level cfg exclusion")
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
        for target in sorted(PROBES | {QUALIFICATION_TARGET}):
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
    if not args.inspect_only and not args.static_only and not errors:
        errors.extend(probe_command_positions(args.makefile))
    for error in errors:
        print(error, file=sys.stderr)
    if errors:
        return 1
    print(
        f"all {len(PROBES) + 1} mandatory local/candidate-CI targets propagate failures"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
