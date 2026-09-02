#!/usr/bin/env python3
"""Bind requirement-tagged Rust tests to the tests Cargo actually compiles.

A `// Trace:` comment is a claim that a test backs a requirement. Quire reads
those comments out of the source; Cargo compiles a set of tests. Nothing
otherwise checks that the two sets are the same, so a tag above a function that
is `#[cfg(...)]`-ed out, renamed, or `#[ignore]`d would keep backing a matrix row
that no longer runs.

This is a producer. It reports; it computes no aggregate verdict beyond its own
obligation and retains nothing.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
TRACE_TEST = re.compile(
    r"//\s*Trace:[^\n]*\n(?:\s*#\[[^\n]+\]\n)*\s*fn\s+([A-Za-z_][A-Za-z0-9_]*)\s*\(",
    re.MULTILINE,
)
TEST_LINE = re.compile(r"^(.+): test$")
SCHEMA = "tl-mltl.test-census/v1"


def tagged_test_names(root: Path) -> set[str]:
    names: set[str] = set()
    for source in root.rglob("*.rs"):
        relative = source.relative_to(root)
        if relative.parts and relative.parts[0] in {".git", "target"}:
            continue
        for name in TRACE_TEST.findall(source.read_text(encoding="utf-8")):
            if name in names:
                raise ValueError(f"duplicate requirement-tagged Rust test name: {name}")
            names.add(name)
    return names


def listed_test_names(output: str) -> tuple[set[str], list[str]]:
    qualified: list[str] = []
    names: set[str] = set()
    for line in output.splitlines():
        match = TEST_LINE.fullmatch(line.strip())
        if match is None or " - (line " in match.group(1):
            continue
        qualified.append(match.group(1))
        name = match.group(1).rsplit("::", 1)[-1]
        if name in names:
            raise ValueError(f"compiled Rust test names are not unique: {name}")
        names.add(name)
    return names, qualified


def cargo_list(ignored: bool = False) -> str:
    arguments = ["cargo", "test", "--all-targets", "--all-features", "--"]
    if ignored:
        arguments.append("--ignored")
    arguments.append("--list")
    result = subprocess.run(arguments, cwd=ROOT, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise RuntimeError((result.stdout + result.stderr).strip())
    return result.stdout


def main() -> int:
    as_json = sys.argv[1:] == ["--json"]
    if sys.argv[1:] and not as_json:
        print("usage: rust_test_census.py [--json]", file=sys.stderr)
        return 2
    try:
        expected = tagged_test_names(ROOT)
        observed, _ = listed_test_names(cargo_list())
        ignored, qualified_ignored = listed_test_names(cargo_list(ignored=True))
    except (OSError, RuntimeError, ValueError) as error:
        # A census that could not be taken is unavailable. It is not a pass, and
        # it is not silently a failure of the code under test either.
        print(f"cannot derive compiled Rust test census: {error}", file=sys.stderr)
        if as_json:
            print(
                json.dumps(
                    {
                        "schemaVersion": SCHEMA,
                        "entries": [
                            {
                                "symbol": "rust-test-census",
                                "outcome": "unavailable",
                                "detail": f"census could not be derived: {error}",
                                "traceIds": ["TC-017"],
                            }
                        ],
                        "matched": False,
                    },
                    indent=2,
                    sort_keys=True,
                )
            )
        return 1

    problems: list[str] = []
    if observed != expected:
        problems.append(
            "compiled Rust test census disagrees with requirement-tagged source tests: "
            f"missing={sorted(expected - observed)}, extra={sorted(observed - expected)}"
        )
    if ignored:
        problems.append(f"compiled Rust tests are ignored: {qualified_ignored}")

    # A census over an empty tagged set passes every comparison it makes and
    # asserts nothing. It is vacuous, and vacuous is not passed.
    outcome = "fail" if problems else ("vacuous" if not observed else "pass")

    if as_json:
        print(
            json.dumps(
                {
                    "schemaVersion": SCHEMA,
                    "entries": [
                        {
                            "symbol": "rust-test-census",
                            "outcome": outcome,
                            "detail": "; ".join(problems)
                            or f"{len(observed)} requirement-tagged compiled tests, none ignored",
                            "traceIds": ["TC-017"],
                        }
                    ],
                    "tagged": sorted(expected),
                    "compiled": sorted(observed),
                    "ignored": sorted(qualified_ignored),
                    "matched": outcome == "pass",
                },
                indent=2,
                sort_keys=True,
            )
        )
        return 0 if outcome == "pass" else 1

    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        return 1
    if not observed:
        print("the requirement-tagged Rust test set is empty", file=sys.stderr)
        return 1
    print(f"compiled Rust test census passed: {len(observed)} requirement-tagged tests")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
