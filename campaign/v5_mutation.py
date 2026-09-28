#!/usr/bin/env python3
"""FR-047: classify exact cargo-mutants populations from retained native bytes."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
import tarfile
from pathlib import Path
from typing import Any

SCHEMA = "tl-mltl.v5-mutation-report/v1"
OUTCOMES = {
    "CaughtMutant": "caught",
    "MissedMutant": "missed",
    "Timeout": "timed_out",
    "Unviable": "unviable",
}
REVIEW_DISPOSITIONS = {"counterexample", "proof_candidate", "reviewed_limitation"}


def failed(status: Any) -> bool:
    return (isinstance(status, dict) and set(status) == {"Failure"}
            and type(status["Failure"]) is int and status["Failure"] != 0)


def timed_out(status: Any) -> bool:
    return status == "Timeout" or (isinstance(status, dict) and set(status) == {"Timeout"})


def validate_phases(kind: str, phases: list[dict[str, Any]],
                    build_argv: list[str], test_argv: list[str]) -> None:
    if not phases or phases[0].get("phase") != "Build":
        raise ValueError(f"{kind}: missing native build phase")
    if phases[0].get("argv") != build_argv:
        raise ValueError(f"{kind}: build selection changed")
    if len(phases) == 2:
        if phases[1].get("phase") != "Test" or phases[1].get("argv") != test_argv:
            raise ValueError(f"{kind}: test selection changed")
    elif len(phases) != 1:
        raise ValueError(f"{kind}: unexpected native phase count")
    build = phases[0]["process_status"]
    test = phases[1]["process_status"] if len(phases) == 2 else None
    valid = {
        "caught": build == "Success" and len(phases) == 2 and failed(test),
        "missed": build == "Success" and len(phases) == 2 and test == "Success",
        "unviable": len(phases) == 1 and failed(build),
        "timed_out": (len(phases) == 1 and timed_out(build)) or
                     (len(phases) == 2 and build == "Success" and timed_out(test)),
    }
    if not valid[kind]:
        raise ValueError(f"{kind}: native phase result contradicts summary")


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def exact_source(path: Path, revision: str) -> None:
    if len(revision) != 40 or any(char not in "0123456789abcdef" for char in revision):
        raise ValueError("source revision must be a full lower-case Git SHA")
    actual = subprocess.run(["git", "rev-parse", "HEAD"], cwd=path, capture_output=True,
                            text=True, check=True).stdout.strip()
    if actual != revision:
        raise ValueError(f"source revision changed: expected {revision}, got {actual}")
    dirty = subprocess.run(["git", "status", "--porcelain"], cwd=path,
                           capture_output=True, text=True, check=True).stdout.strip()
    if dirty:
        raise ValueError(f"source restoration is dirty: {path}")


def native_archive(
    path: Path,
) -> tuple[dict[str, Any], list[dict[str, Any]], dict[str, Any], dict[str, Any]]:
    with tarfile.open(path, "r:gz") as archive:
        names = set(archive.getnames())
        for required in ("mutants.out/outcomes.json", "mutants.out/mutants.json",
                         "mutants.out/log/baseline.log", "restored_control.json",
                         "restored_control.stdout", "restored_control.stderr",
                         "invocation.json", "invocation.stdout", "invocation.stderr"):
            if required not in names:
                raise ValueError(f"native archive lacks {required}")
        def read(name: str) -> bytes:
            member = archive.extractfile(name)
            if member is None:
                raise ValueError(f"native archive member is not a file: {name}")
            return member.read()

        outcomes = json.loads(read("mutants.out/outcomes.json"))
        selected = json.loads(read("mutants.out/mutants.json"))
        restored = json.loads(read("restored_control.json"))
        invocation = json.loads(read("invocation.json"))
        for outcome in outcomes["outcomes"]:
            for field in ("log_path", "diff_path"):
                relative = outcome.get(field)
                if relative and f"mutants.out/{relative}" not in names:
                    raise ValueError(f"missing native {field}: {relative}")
        return outcomes, selected, restored, invocation


def classify(
    discovered: list[dict[str, Any]],
    selected: list[dict[str, Any]],
    native: dict[str, Any],
    test_tail: list[str],
    reviews: list[dict[str, str]],
) -> dict[str, Any]:
    """Reconcile every selected outcome without silently shrinking the viable set."""
    discovered_names = [item["name"] for item in discovered]
    selected_names = [item["name"] for item in selected]
    if not discovered_names or not selected_names:
        raise ValueError("empty discovery or selection")
    if len(set(discovered_names)) != len(discovered_names):
        raise ValueError("duplicate discovered mutant")
    if len(set(selected_names)) != len(selected_names):
        raise ValueError("duplicate selected mutant")
    if not set(selected_names) <= set(discovered_names):
        raise ValueError("selected mutant was not discovered")
    outcomes = native["outcomes"]
    if not outcomes or outcomes[0].get("scenario") != "Baseline":
        raise ValueError("missing native baseline")
    baseline = outcomes[0]
    if baseline.get("summary") != "Success":
        raise ValueError("unmutated baseline did not pass")
    baseline_phases = baseline["phase_results"]
    if ([phase.get("phase") for phase in baseline_phases] != ["Build", "Test"]
            or any(phase["process_status"] != "Success" for phase in baseline_phases)):
        raise ValueError("unmutated baseline build/test did not both pass")
    if not test_tail:
        raise ValueError("fixed test selection is empty")
    build_argv = baseline_phases[0]["argv"]
    test_argv = baseline_phases[1]["argv"]
    if test_argv[-len(test_tail):] != test_tail:
        raise ValueError("baseline test selection changed")
    counts = {value: 0 for value in OUTCOMES.values()}
    outcome_names = []
    review_needed = {}
    for item in outcomes[1:]:
        mutant = item.get("scenario", {}).get("Mutant")
        if not isinstance(mutant, dict):
            raise ValueError("non-mutant scenario after baseline")
        name = mutant["name"]
        outcome_names.append(name)
        summary = item.get("summary")
        if summary not in OUTCOMES:
            raise ValueError(f"unknown native outcome: {summary}")
        kind = OUTCOMES[summary]
        counts[kind] += 1
        if kind in ("missed", "timed_out"):
            review_needed[name] = kind
        validate_phases(kind, item["phase_results"], build_argv, test_argv)
    if len(outcome_names) != len(selected_names) or set(outcome_names) != set(selected_names):
        raise ValueError("native outcomes do not exactly cover selected mutants")
    if native["total_mutants"] != len(selected_names):
        raise ValueError("native total disagrees with selection")
    for native_key, local_key in (("caught", "caught"), ("missed", "missed"),
                                  ("timeout", "timed_out"), ("unviable", "unviable")):
        if native[native_key] != counts[local_key]:
            raise ValueError(f"native {native_key} summary disagrees with outcomes")
    by_name = {}
    for review in reviews:
        name = review["name"]
        if name in by_name or review["disposition"] not in REVIEW_DISPOSITIONS:
            raise ValueError("duplicate or invalid survivor review")
        if not review.get("detail", "").strip():
            raise ValueError("survivor review lacks a concrete detail")
        by_name[name] = review
    if set(by_name) != set(review_needed):
        raise ValueError("missed/timed-out survivor review population is incomplete")
    viable = counts["caught"] + counts["missed"] + counts["timed_out"]
    if viable == 0:
        raise ValueError("no viable semantic mutants measured")
    result = {
        "discovered": len(discovered_names),
        "selected": len(selected_names),
        "not_run": len(discovered_names) - len(selected_names),
        "viable": viable,
        **counts,
        "kill_rate": {"caught": counts["caught"], "viable": viable},
        "threshold_percent": 90,
        "survivor_reviews": reviews,
        "cargo_mutants_version": native["cargo_mutants_version"],
        "test_argv": test_argv,
    }
    if counts["timed_out"]:
        result["status"] = "incomplete"
        result["reason"] = "timed_out_mutants_are_not_completed_outcomes"
    elif counts["caught"] * 10 < viable * 9:
        result["status"] = "failed"
        result["reason"] = "below_90_percent_viable_caught"
    else:
        result["status"] = "passed"
    return result


def exact_selection(discovered: list[dict[str, Any]], selected: list[dict[str, Any]],
                    source_file: str, expression: str) -> None:
    if not source_file or not expression:
        raise ValueError("missing fixed source file or mutant regex")
    expected = {item["name"] for item in discovered
                if item["file"] == source_file and re.search(expression, item["name"])}
    if expected != {item["name"] for item in selected}:
        raise ValueError("native selected mutants differ from fixed discovery filter")


def verify_restored_control(restored: dict[str, Any], revision: str,
                            test_tail: list[str]) -> None:
    if restored.get("source_revision") != revision:
        raise ValueError("restored control source revision differs")
    if restored.get("argv") != ["cargo", "test", "--locked", "--all-features",
                                *test_tail]:
        raise ValueError("restored control test selection differs")
    if type(restored.get("exit_code")) is not int or restored["exit_code"] != 0:
        raise ValueError("restored green control failed")


def verify_invocation(invocation: dict[str, Any], entry: dict[str, Any],
                      archive_path: Path, native: dict[str, Any]) -> None:
    if invocation.get("source_revision") != entry["source_revision"]:
        raise ValueError("mutation invocation source revision differs")
    discovery_command = ["cargo", "mutants", "--no-config", "--all-features", "--list",
                         "--json", "--file", entry["source_file"]]
    if invocation.get("discovery_command") != discovery_command:
        raise ValueError("mutation discovery command differs")
    mutation_command = [
        "cargo", "mutants", "--no-config", "--all-features", "--file",
        entry["source_file"], "--re", entry["selection_regex"], "--output",
        str(archive_path.parent / "native"), "--timeout", "120", "--jobs", "1", "--",
        *entry["test_tail"],
    ]
    if invocation.get("mutation_command") != mutation_command:
        raise ValueError("mutation test selection or output command differs")
    expected_code = 3 if native["timeout"] else (2 if native["missed"] else 0)
    if type(invocation.get("exit_code")) is not int or invocation["exit_code"] != expected_code:
        raise ValueError("mutation invocation exit code contradicts native outcomes")


def report(manifest: dict[str, Any], base: Path) -> dict[str, Any]:
    if manifest.get("schema") != "tl-mltl.v5-mutation-manifest/v1":
        raise ValueError("wrong V5 manifest schema")
    runs = {}
    for entry in manifest["runs"]:
        crate = entry["crate"]
        if crate in runs:
            raise ValueError(f"duplicate crate: {crate}")
        source = (base / entry["source_path"]).resolve()
        exact_source(source, entry["source_revision"])
        discovery_path = (base / entry["discovery_path"]).resolve()
        archive_path = (base / entry["archive_path"]).resolve()
        discovery_bytes = discovery_path.read_bytes()
        archive_bytes = archive_path.read_bytes()
        if digest(discovery_bytes) != entry["discovery_sha256"]:
            raise ValueError(f"{crate}: stale discovery bytes")
        if digest(archive_bytes) != entry["archive_sha256"]:
            raise ValueError(f"{crate}: stale native archive bytes")
        native, selected, restored, invocation = native_archive(archive_path)
        verify_restored_control(restored, entry["source_revision"], entry["test_tail"])
        verify_invocation(invocation, entry, archive_path, native)
        discovered = json.loads(discovery_bytes)
        exact_selection(discovered, selected, entry["source_file"], entry["selection_regex"])
        outcome = classify(discovered, selected, native,
                           entry["test_tail"], entry.get("survivor_reviews", []))
        runs[crate] = outcome | {
            "source_revision": entry["source_revision"],
            "critical_scope": entry["critical_scope"],
            "source_file": entry["source_file"],
            "selection_regex": entry["selection_regex"],
            "discovery_sha256": entry["discovery_sha256"],
            "archive_sha256": entry["archive_sha256"],
        }
    if not runs:
        raise ValueError("no mutation runs")
    aggregate = "failed" if any(run["status"] == "failed" for run in runs.values()) else (
        "incomplete" if any(run["status"] == "incomplete" for run in runs.values()) else "passed"
    )
    return {"schema": SCHEMA, "status": aggregate, "runs": runs}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest_path = args.manifest.resolve()
    try:
        result = report(json.loads(manifest_path.read_bytes()), manifest_path.parent)
    except (OSError, KeyError, TypeError, ValueError, subprocess.CalledProcessError,
            tarfile.TarError) as error:
        result = {"schema": SCHEMA, "status": "failed", "reason": str(error)}
    args.output.write_text(json.dumps(result, sort_keys=True, indent=2) + "\n")
    return 0 if result["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
