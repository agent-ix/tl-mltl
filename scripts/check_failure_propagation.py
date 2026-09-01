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
ASSIGNMENT_OPERATOR = r"(?:\+=|\?=|!=|:::=|::=|:=|=)"
CONTROL_NAMES = r"MAKEFLAGS|MAKE|SHELL|\.SHELLFLAGS"
CONTROL_ASSIGNMENT = re.compile(
    rf"^\s*(?:(?:export|override|unexport|private)\s+)*"
    rf"({CONTROL_NAMES})\s*{ASSIGNMENT_OPERATOR}\s*(.*)$"
)
CONTROL_DEFINE = re.compile(
    rf"^\s*(?:(?:export|override|unexport|private)\s+)*define\s+"
    rf"({CONTROL_NAMES})(?:\s|$)"
)
CONTROL_DIRECTIVE = re.compile(r"^\s*\.(IGNORE|SILENT|ONESHELL|DEFAULT)\s*(?::|$)")
CONTROL_EVAL = re.compile(r"\$\s*[({]\s*eval\b")
TARGET_SCOPED_CONTROL = re.compile(
    rf"^\s*([^#=\n]+?):\s*(?:(?:export|override|unexport|private)\s+)*"
    rf"({CONTROL_NAMES})\s*{ASSIGNMENT_OPERATOR}"
)
MAKEFILE_IMPORT = re.compile(r"^\s*(?:-?include|sinclude)\b")
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
        if current in dependencies:
            raise ValueError(f"duplicate target rule can overwrite policy state: {current}")
        dependencies[current] = (match.group(2) or "").split()
    return dependencies, recipes


def makeflags_ignore_errors(value: str) -> bool:
    """Return whether GNU Make flags can change what mandatory CI executes."""
    try:
        tokens = shlex.split(value)
    except ValueError:
        return True
    for token in tokens:
        if token.startswith(("--jobs", "--jobserver-", "--load-average", "--output-sync")):
            continue
        if token in {"--print-directory", "--no-print-directory"}:
            continue
        if re.fullmatch(r"-(?:j|l|O)(?:[0-9.]+|[A-Za-z]+)?", token):
            continue
        if token == "-w":
            continue
        if token:
            return True
    return False


def command_parts(command: str) -> tuple[str, str]:
    stripped = command.lstrip()
    modifiers = ""
    while stripped[:1] in {"@", "+", "-"}:
        modifiers += stripped[0]
        stripped = stripped[1:].lstrip()
    return modifiers, stripped


def inspect_execution_controls(text: str) -> list[str]:
    """Reject global, scoped, generated, and imported Make execution controls."""
    errors: list[str] = []
    for number, line in enumerate(text.splitlines(), start=1):
        if line.startswith("\t"):
            continue
        target_scoped = TARGET_SCOPED_CONTROL.match(line)
        if target_scoped is not None:
            targets, name = target_scoped.groups()
            errors.append(
                f"Makefile:{number} assigns target-scoped execution control {name} "
                f"for {targets.strip()}"
            )
        directive = CONTROL_DIRECTIVE.match(line)
        if directive is not None:
            errors.append(f"Makefile:{number} declares .{directive.group(1)}")
        assignment = CONTROL_ASSIGNMENT.match(line)
        if assignment is not None:
            name, value = assignment.groups()
            if name != "MAKEFLAGS" or makeflags_ignore_errors(value):
                errors.append(f"Makefile:{number} assigns execution control {name}")
        define = CONTROL_DEFINE.match(line)
        if define is not None:
            errors.append(f"Makefile:{number} defines execution control {define.group(1)}")
        if CONTROL_EVAL.search(line):
            errors.append(f"Makefile:{number} uses eval, which can hide execution controls")
        if MAKEFILE_IMPORT.match(line):
            errors.append(f"Makefile:{number} includes an uninspected Make fragment")
    return errors


def inspect(makefile: Path, root: Path = ROOT) -> list[str]:
    try:
        text = makefile.read_text(encoding="utf-8")
    except OSError as error:
        return [f"cannot read Makefile {makefile}: {error}"]
    errors = inspect_execution_controls(text)
    try:
        dependencies, recipes = parse_makefile(text)
    except ValueError as error:
        errors.append(f"Makefile structure is ambiguous: {error}")
        return errors
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


def inspect_expanded_recipes(makefile: Path, root: Path = ROOT) -> list[str]:
    """Inspect recipes after Make variable expansion for hidden shell controls."""
    errors: list[str] = []
    clean_env = dict(os.environ)
    clean_env.pop("MAKEFLAGS", None)
    clean_env.pop("PYTHONOPTIMIZE", None)
    for target in sorted(PROBES | {QUALIFICATION_TARGET}):
        try:
            result = subprocess.run(
                ["/usr/bin/make", "--no-print-directory", "-n", "-f", str(makefile), target],
                cwd=root,
                check=False,
                capture_output=True,
                text=True,
                env=clean_env,
            )
        except FileNotFoundError:
            return ["required Make executable is unavailable: /usr/bin/make"]
        if result.returncode != 0:
            errors.append(f"cannot expand mandatory target {target}: {result.stderr.strip()}")
            continue
        for command in result.stdout.splitlines():
            if FORBIDDEN_SHELL_CONTROL.search(command):
                errors.append(
                    f"expanded mandatory target {target} uses a forbidden shell control "
                    f"operator: {command}"
                )
    return errors


def probe_command_positions(makefile: Path) -> list[str]:
    """Substitute false at every mandatory recipe position and require Make to fail."""
    text = makefile.read_text(encoding="utf-8")
    control_errors = inspect_execution_controls(text)
    if control_errors:
        return [
            "command-position probe refuses a Makefile with execution controls: " + error
            for error in control_errors
        ]
    try:
        _, recipes = parse_makefile(text)
    except ValueError as error:
        return [f"command-position probe refuses an ambiguous Makefile: {error}"]
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
    if makeflags_ignore_errors(os.environ.get("MAKEFLAGS", "")):
        errors.append("ambient MAKEFLAGS can change mandatory local CI execution")
    if os.environ.get("MAKE"):
        errors.append("ambient MAKE override is not permitted")
    if os.environ.get("PYTHONOPTIMIZE") or sys.flags.optimize:
        errors.append("optimized Python disables policy checks")
    if not args.static_only and not errors:
        errors.extend(inspect_expanded_recipes(args.makefile, ROOT))
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
