#!/usr/bin/env python3
"""Rederive the post-envelope summary from retained qualified evidence."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

import evidence_profile
import parameter_identity
import rust_test_census
import tool_identity


ROOT = Path(__file__).resolve().parent.parent
CHECKS = (
    "candidate-gates",
    "make-spec",
    "quire-coverage",
    "rustdoc",
    "default-dependencies",
    "diff-integrity",
    "input-schema",
    "manifest-schema",
    "pgm01-schema",
    "pgm01-validator",
    "sealed-pgm01-schema",
    "sealed-pgm01-validator",
)
CONTRADICTION = re.compile(
    r"test result: FAILED|Error [0-9]+ \(ignored\)|\b[1-9][0-9]* ignored\b"
)
TEST_SUCCESS = re.compile(
    r"^test result: ok\. ([0-9]+) passed; 0 failed; 0 ignored", re.MULTILINE
)
MAKE_CI_SIGNATURES = (
    "fmt-check gate passed",
    "lint gate passed",
    "Rust test gate passed",
    "rust-test-census gate passed",
    "corpus-integrity gate passed",
    "deny gate passed",
    "audit-unsafe gate passed",
    "evidence-tool gate passed",
    "spec gate passed",
    "rustdoc gate passed",
    "candidate CI gate passed",
    "qualified tool identities passed",
)


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def source_revision(evidence_dir: Path) -> str:
    return (evidence_dir / "source-revision.txt").read_text(encoding="utf-8").strip()


def git_text(revision: str, relative: str) -> str:
    return subprocess.run(
        ["/usr/bin/git", "show", f"{revision}:{relative}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
        text=True,
    ).stdout


def git_bytes(revision: str, relative: str) -> bytes:
    return subprocess.run(
        ["/usr/bin/git", "show", f"{revision}:{relative}"],
        cwd=ROOT,
        check=True,
        capture_output=True,
    ).stdout


def parameter_paths_at_revision(revision: str) -> list[str]:
    tree = set(
        subprocess.run(
            ["/usr/bin/git", "ls-tree", "-r", "--name-only", revision],
            cwd=ROOT,
            check=True,
            capture_output=True,
            text=True,
        ).stdout.splitlines()
    )
    return parameter_identity.parameter_names(tree)


def historical_parameters_digest(revision: str) -> str:
    state = hashlib.sha256()
    for relative in parameter_paths_at_revision(revision):
        state.update(relative.encode("utf-8"))
        state.update(b"\0")
        state.update(git_bytes(revision, relative))
        state.update(b"\0")
    return state.hexdigest()


def expected_rust_tests(evidence_dir: Path) -> int:
    revision = source_revision(evidence_dir)
    count = len(rust_test_census.git_tagged_test_names(ROOT, revision))
    if count <= 0:
        raise ValueError("cannot derive a non-empty Rust test census from the source revision")
    return count


def complete_fraction(output: str, prefix: str) -> bool:
    for backed, total in re.findall(prefix + r"\s*([0-9]+)/([0-9]+)", output):
        if int(total) > 0 and backed == total:
            return True
    return False


def positive_output(evidence_dir: Path, name: str) -> bool:
    output = "\n".join(
        path.read_text(encoding="utf-8", errors="replace")
        for path in (evidence_dir / f"{name}.stdout", evidence_dir / f"{name}.stderr")
        if path.exists()
    )
    if name == "diff-integrity":
        return not output.strip()
    if name == "candidate-gates":
        passed = [int(value) for value in TEST_SUCCESS.findall(output)]
        return (
            sum(passed) == expected_rust_tests(evidence_dir)
            and "all 12 mandatory local/candidate-CI targets propagate failures" in output
            and "all 3 evidence-policy behavior tests passed" in output
            and complete_fraction(output, r"Coverage:")
            and "licenses ok" in output
            and "sources ok" in output
            and all(signature in output for signature in MAKE_CI_SIGNATURES)
        )
    if name == "make-spec":
        return complete_fraction(output, r"Coverage:") and "spec gate passed" in output
    if name == "quire-coverage":
        return complete_fraction(output, r"Coverage:")
    if name == "rustdoc":
        return "Generated " in output and "/doc/tl_mltl/index.html" in output
    if name == "default-dependencies":
        lines = [line.strip() for line in output.splitlines() if line.strip()]
        return len(lines) == 1 and lines[0].startswith("tl-mltl v0.1.0 ")
    if name in {
        "input-schema",
        "manifest-schema",
        "pgm01-schema",
        "pgm01-validator",
        "sealed-pgm01-schema",
        "sealed-pgm01-validator",
    }:
        return re.search(r'"errors"\s*:\s*\[\]\s*,?\s*"valid"\s*:\s*true', output) is not None
    return False


def derive_outcomes(evidence_dir: Path, require_positive: bool) -> list[dict[str, object]]:
    outcomes: list[dict[str, object]] = []
    observed = {
        path.name[: -len(".status.txt")]
        for path in evidence_dir.glob("*.status.txt")
        if path.is_file()
    }
    for name in list(CHECKS) + sorted(observed - set(CHECKS)):
        status_path = evidence_dir / f"{name}.status.txt"
        if not status_path.exists():
            outcomes.append({"name": name, "status": "inconclusive", "exitCode": None})
            continue
        exit_code = int(status_path.read_text(encoding="utf-8").strip())
        stdout_path = evidence_dir / f"{name}.stdout"
        stderr_path = evidence_dir / f"{name}.stderr"
        explicit_skip = (
            exit_code == 125
            and stdout_path.exists()
            and stdout_path.read_text(encoding="utf-8", errors="replace").strip()
            == "skipped-unavailable"
            and stderr_path.exists()
            and not stderr_path.read_text(encoding="utf-8", errors="replace").strip()
        )
        malformed_skip = exit_code == 125 and not explicit_skip
        validator_error = (
            exit_code == 0
            and name in {"pgm01-validator", "sealed-pgm01-validator"}
            and stderr_path.exists()
            and bool(stderr_path.read_text(encoding="utf-8").strip())
        )
        contradiction = any(
            path.exists()
            and CONTRADICTION.search(path.read_text(encoding="utf-8", errors="replace"))
            for path in (evidence_dir / f"{name}.stdout", evidence_dir / f"{name}.stderr")
        )
        positive_missing = exit_code == 0 and require_positive and not positive_output(evidence_dir, name)
        outcomes.append(
            {
                "name": name,
                "status": (
                    "skipped-unavailable"
                    if explicit_skip
                    else "failed"
                    if malformed_skip or validator_error or contradiction or positive_missing
                    else "passed"
                    if exit_code == 0
                    else "failed"
                ),
                "exitCode": exit_code,
            }
        )
    return outcomes


def summary(evidence_dir: Path) -> dict[str, object]:
    if evidence_profile.resolve_profile(evidence_dir) != "v2":
        raise ValueError("active evidence lacks the qualified v2 profile and source tool lock")
    outcomes = derive_outcomes(evidence_dir, require_positive=True)
    statuses = {item["status"] for item in outcomes}
    if "failed" in statuses:
        overall = "failed"
    elif "skipped-unavailable" in statuses or "inconclusive" in statuses:
        overall = "inconclusive"
    else:
        overall = "passed"
    envelope = evidence_dir / "evidence-envelope.json"
    return {
        "schemaVersion": "tl-mltl.collection-summary/v1",
        "overallStatus": overall,
        "finalEnvelopeSha256": sha256(envelope),
        "finalEnvelopeValidated": all(
            item["status"] == "passed"
            for item in outcomes
            if str(item["name"]).startswith("sealed-")
        ),
        "outcomes": outcomes,
    }


def validate_tool_identity(evidence_dir: Path) -> list[str]:
    revision = source_revision(evidence_dir)
    try:
        source_lock = json.loads(git_text(revision, "tools.lock"))
        source_tools = source_lock.get("tools")
        required = (
            tool_identity.REQUIRED
            if isinstance(source_tools, dict)
            and set(source_tools) == set(tool_identity.REQUIRED)
            else tool_identity.LEGACY_REQUIRED
        )
        expected = tool_identity.validate_lock(
            source_lock,
            required=required,
            # Verification is portable: only a new collection must reproduce
            # this checkout's host-local absolute target path.
            expected_target=None,
        )
        collection_input = json.loads(
            (evidence_dir / "collection-input.json").read_text(encoding="utf-8")
        )
        observed = collection_input["tools"]["identities"]
    except (KeyError, OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        return [f"cannot rederive retained tool identities: {error}"]
    return [] if observed == expected else [
        f"retained tool identities disagree with source tools.lock: {evidence_dir}"
    ]


def validate_parameter_identity(evidence_dir: Path) -> list[str]:
    revision = source_revision(evidence_dir)
    try:
        envelope = json.loads(
            (evidence_dir / "evidence-envelope.json").read_text(encoding="utf-8")
        )
        observed = envelope["parametersDigest"]["value"]
        expected = historical_parameters_digest(revision)
    except (
        KeyError,
        OSError,
        ValueError,
        json.JSONDecodeError,
        subprocess.CalledProcessError,
    ) as error:
        return [f"cannot rederive retained parameter identity: {error}"]
    return [] if observed == expected else [
        f"retained parameters digest disagrees with source revision: {evidence_dir}"
    ]


def validate_envelope_result(evidence_dir: Path, value: dict[str, object]) -> list[str]:
    try:
        envelope = json.loads((evidence_dir / "evidence-envelope.json").read_text(encoding="utf-8"))
        actual = envelope["result"]["status"]
    except (KeyError, OSError, json.JSONDecodeError, TypeError) as error:
        return [f"cannot derive retained envelope result: {error}"]
    outcomes = value["outcomes"]
    if not isinstance(outcomes, list):
        return ["collection summary outcomes are not a list"]
    sealed_failed = any(
        str(item["name"]).startswith("sealed-") and item["status"] != "passed"
        for item in outcomes
    )
    expected = "error" if value["overallStatus"] == "failed" or sealed_failed else "inconclusive"
    return [] if actual == expected else [f"envelope result {actual!r} disagrees with {expected!r}"]


def main() -> int:
    check = len(sys.argv) == 3 and sys.argv[1] == "--check"
    write = len(sys.argv) == 2 and sys.argv[1] != "--check"
    if not check and not write:
        print("usage: finalize_collection.py [--check] EVIDENCE_DIR", file=sys.stderr)
        return 2
    evidence_dir = Path(sys.argv[2] if check else sys.argv[1])
    try:
        profile = evidence_profile.resolve_profile(evidence_dir)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"cannot resolve evidence qualification profile: {error}", file=sys.stderr)
        return 2
    if profile == "retracted":
        if not check:
            print(f"refusing to rewrite explicitly retracted evidence: {evidence_dir}", file=sys.stderr)
            return 2
        print(f"retained evidence is explicitly retracted: {evidence_dir}")
        return 3
    if profile != "v2":
        print(f"active evidence is inconclusive without qualification-v2: {evidence_dir}", file=sys.stderr)
        return 3
    try:
        value = summary(evidence_dir)
    except (OSError, ValueError, json.JSONDecodeError, subprocess.CalledProcessError) as error:
        print(f"cannot derive retained collection summary: {error}", file=sys.stderr)
        return 2
    errors = (
        validate_envelope_result(evidence_dir, value)
        + validate_tool_identity(evidence_dir)
        + validate_parameter_identity(evidence_dir)
    )
    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        return 1
    summary_path = evidence_dir / "collection-summary.json"
    if check:
        try:
            actual = json.loads(summary_path.read_text(encoding="utf-8"))
        except (OSError, json.JSONDecodeError) as error:
            print(f"cannot read retained summary: {error}", file=sys.stderr)
            return 2
        if actual != value:
            print(f"retained summary disagrees with status files: {evidence_dir}", file=sys.stderr)
            return 1
        if value["overallStatus"] == "inconclusive":
            print(f"retained evidence is inconclusive: {evidence_dir}", file=sys.stderr)
            return 3
        if value["overallStatus"] != "passed":
            print(f"retained evidence failed qualification: {evidence_dir}", file=sys.stderr)
            return 1
        return 0
    summary_path.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
