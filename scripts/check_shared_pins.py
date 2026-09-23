#!/usr/bin/env python3
"""Observe the local toolchain and let Engineering Assurance classify it (FR-006-AC-1).

Four things this file deliberately is not.

It is not a copy of the compatibility matrix. It never says which version of
anything is correct. It observes what is installed and hands every verdict to
the pinned `engineering-assurance compatibility` command, because a second copy of the rule is a
second authority, and two authorities drift.

It does not create human acceptance. The pinned command reports whether its
embedded matrix contains an attributed human decision. This script preserves
that fact and requires it for the migration gate.

It is not a network probe. It does not ask a registry whether a release landed.
`npm.ix` in particular is a mirror that lags the public registry and is not an
oracle for anything; the only thing this script does about it is refuse to find
it written down anywhere in this repository.

It is not an envelope. It prints a report and exits. It retains nothing.

Exit status: 0 when the EA classifier's accepted matrix and local checks pass,
1 when the matrix withholds approval or a local check fails, 2 when Engineering
Assurance cannot classify — a different fact from a failing check.
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parent.parent
PINS_PATH = ROOT / "assurance" / "pins.json"

FORBIDDEN_REGISTRY = "npm.ix"

# Files a mirror reference could realistically hide in. Read line by line rather
# than grepped as a blob, so that pins.json's own prose about the mirror does not
# match itself and report a violation that is actually the rule being written down.
MIRROR_SCAN_FILES = (
    ".npmrc",
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    ".github/workflows/ci.yml",
)


class PinError(RuntimeError):
    """The pinned assurance distribution could not be used."""


def observe(argv: list[str]) -> str | None:
    """Run a version probe. An absent tool is None, which upstream calls unknown."""
    try:
        result = subprocess.run(argv, capture_output=True, text=True, check=False)
    except (OSError, ValueError):
        return None
    if result.returncode != 0:
        return None
    value = result.stdout.strip()
    return value or None


def observe_quire() -> str | None:
    """Read the CLI version from quire's own provenance record, not its banner."""
    raw = observe(["quire", "provenance"])
    if raw is None:
        return None
    try:
        return str(json.loads(raw)["cli"]["version"])
    except (json.JSONDecodeError, KeyError, TypeError):
        return None


def observe_engineering_assurance() -> str | None:
    raw = observe(["engineering-assurance", "--version"])
    match = re.fullmatch(r"engineering-assurance (\d+\.\d+\.\d+)", raw or "")
    return match.group(1) if match else None


def classify_with_ea(observed: dict[str, str | None]) -> dict[str, Any]:
    """Ask the pinned native classifier; a withheld result still has useful rows."""
    request = {"protocol": "engineering-assurance.compatibility-request/v1",
               "observed": [{"component": name, "version": version}
                            for name, version in observed.items()]}
    try:
        result = subprocess.run(
            ["engineering-assurance", "compatibility"],
            input=json.dumps(request), capture_output=True, text=True, check=False,
        )
    except OSError as error:
        raise PinError(f"the pinned assurance command is unavailable: {error}") from error
    if result.returncode not in (0, 1):
        raise PinError(f"the pinned assurance classifier failed: {result.stderr.strip()}")
    try:
        report = json.loads(result.stdout)
    except json.JSONDecodeError as error:
        raise PinError("the pinned assurance classifier returned invalid JSON") from error
    expected_keys = {"protocol", "matrix_version", "outcome", "versions_compatible",
                     "human_acceptance_recorded", "gate_satisfied", "components"}
    if (type(report) is not dict or set(report) != expected_keys or
            report["protocol"] != "engineering-assurance.compatibility-result/v1" or
            report["matrix_version"] != "engineering-assurance.compatibility-matrix/v1" or
            type(report["versions_compatible"]) is not bool or
            type(report["human_acceptance_recorded"]) is not bool or
            type(report["gate_satisfied"]) is not bool or
            type(report["components"]) is not list or
            len(report["components"]) != len(observed)):
        raise PinError("the pinned assurance classifier returned an invalid result")
    classified = []
    for item in report["components"]:
        if (type(item) is not dict or
                set(item) != {"component", "observed", "expected", "verdict", "reason"} or
                type(item["component"]) is not str or
                item["component"] not in observed or
                item["observed"] != observed[item["component"]] or
                item["verdict"] not in {"compatible", "incompatible", "unknown"} or
                type(item["expected"]) is not str or not item["expected"] or
                type(item["reason"]) is not str or not item["reason"]):
            raise PinError("the pinned assurance classifier returned an invalid component")
        classified.append(item["component"])
    if len(set(classified)) != len(observed):
        raise PinError("the pinned assurance classifier omitted or duplicated a component")
    versions_ok = all(item["verdict"] == "compatible" for item in report["components"])
    gate_ok = versions_ok and report["human_acceptance_recorded"]
    if (report["versions_compatible"] != versions_ok or
            report["gate_satisfied"] != gate_ok or
            report["outcome"] != ("compatible" if gate_ok else "withheld") or
            result.returncode != (0 if gate_ok else 1)):
        raise PinError("the pinned assurance classifier returned inconsistent outcomes")
    return report


def mirror_references(pins: dict[str, Any]) -> list[str]:
    """Find any place this repository would resolve a component from the mirror."""
    offenders: list[str] = []
    for name in MIRROR_SCAN_FILES:
        path = ROOT / name
        if not path.is_file():
            continue
        try:
            text = path.read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        for number, line in enumerate(text.splitlines(), start=1):
            if FORBIDDEN_REGISTRY in line:
                offenders.append(f"{name}:{number}")
    # pins.json is inspected structurally: its prose says the mirror's name on
    # purpose, and matching that would be the check reporting its own statement.
    requirement = pins["engineering_assurance"]["requirement"]
    if FORBIDDEN_REGISTRY in requirement:
        offenders.append("assurance/pins.json:engineering_assurance.requirement")
    module_install = pins["engineering_assurance"]["module_install"]
    if FORBIDDEN_REGISTRY in module_install:
        offenders.append("assurance/pins.json:engineering_assurance.module_install")
    return offenders


def upstream_pin_mismatches(pins: dict[str, Any]) -> list[str]:
    """Check the tl-syntax pin this repository actually depends on.

    The COMPILED revision is what Cargo resolves and what the C2PO mapping
    manifest reports as `syntaxRevision`. Every current-facing record that
    names it must agree: a lockfile that drifted from `Cargo.toml`, or a
    `TL_SYNTAX_REVISION` constant that still names the old pin, would make the
    crate report a dependency identity it is not actually built from. The
    shared temporal corpus and the future-operator corpus are both read
    straight out of this same compiled dependency via `tl_syntax::CORPUS_DIR`,
    so there is no separate corpus basis to track for either any more.
    """
    compiled = pins["upstream_dependency"]["compiled_revision"]
    problems: list[str] = []
    checks = {
        "Cargo.toml": f'rev = "{compiled}"',
        "Cargo.lock": f"#{compiled}",
        "src/lib.rs": f'TL_SYNTAX_REVISION: &str = "{compiled}"',
        "README.md": f"`{compiled}`",
        "corpus/README.md": f"`{compiled}`",
        "assurance/change-assurance.json": f"The compiled dependency moved to {compiled[:8]}",
    }
    for name, needle in checks.items():
        path = ROOT / name.strip()
        if not path.is_file():
            problems.append(f"{name}: absent")
            continue
        if needle not in path.read_text(encoding="utf-8"):
            problems.append(f"{name.strip()}: does not name the expected revision")
    current_records = ("README.md", "corpus/README.md", "assurance/change-assurance.json")
    for superseded in pins["upstream_dependency"].get("superseded_compiled_revisions", []):
        for name in current_records:
            path = ROOT / name
            if path.is_file() and superseded in path.read_text(encoding="utf-8"):
                problems.append(f"{name}: still names superseded compiled revision {superseded}")
    return problems


def build_report() -> dict[str, Any]:
    pins = json.loads(PINS_PATH.read_text(encoding="utf-8"))
    observed = {
        "quire-cli": observe_quire(),
        "quoin": observe(["quoin", "--version"]),
        "ix-flow": observe(["ix-flow", "--version"]),
        "engineering-assurance": observe_engineering_assurance(),
    }
    classification = classify_with_ea(observed)
    offenders = mirror_references(pins)
    upstream = upstream_pin_mismatches(pins)
    versions_ok = classification["versions_compatible"]
    acceptance_recorded = classification["human_acceptance_recorded"]
    return {
        "schemaVersion": "tl-mltl.shared-pin-report/v1",
        "matrix_version": classification["matrix_version"],
        "acceptance_state": "accepted" if acceptance_recorded else "pending_human_acceptance",
        "acceptance_recorded_here": False,
        "acceptance_authority": (
            "The matrix embedded in the pinned engineering-assurance CLI. "
            "This repository reports it and is not a second acceptance authority."
        ),
        "versions_compatible": versions_ok,
        "mirror_references": offenders,
        "upstream_pin_mismatches": upstream,
        "accepted": classification["gate_satisfied"] and not offenders and not upstream,
        "components": classification["components"],
    }


def main(argv: list[str]) -> int:
    as_json = argv[1:] == ["--json"]
    if argv[1:] and not as_json:
        print("usage: check_shared_pins.py [--json]", file=sys.stderr)
        return 2
    try:
        report = build_report()
    except PinError as error:
        print(str(error), file=sys.stderr)
        return 2
    if as_json:
        print(json.dumps(report, indent=2, sort_keys=True))
    else:
        for item in report["components"]:
            observed = item["observed"] if item["observed"] is not None else "not observed"
            print(f"{item['component']}: {observed} -> {item['verdict']} ({item['reason']})")
        for offender in report["mirror_references"]:
            print(f"mirror registry reference: {offender}", file=sys.stderr)
        for problem in report["upstream_pin_mismatches"]:
            print(f"upstream pin disagreement: {problem}", file=sys.stderr)
        print(
            f"acceptance state recorded by the pinned release: {report['acceptance_state']} "
            "(required for this gate)"
        )
        print(
            "shared pins accepted" if report["accepted"] else "shared pins NOT accepted",
            file=sys.stderr if not report["accepted"] else sys.stdout,
        )
    return 0 if report["accepted"] else 1


if __name__ == "__main__":
    raise SystemExit(main(sys.argv))
