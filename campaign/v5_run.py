#!/usr/bin/env python3
"""FR-047: run fixed cargo-mutants selections and retain their complete raw output."""

from __future__ import annotations

import argparse
import io
import json
import os
import subprocess
import tarfile
from pathlib import Path
from typing import Any

import v5_mutation as v5


def command(source_file: str, expression: str, output: Path, tail: list[str]) -> list[str]:
    if not source_file or not expression or not tail:
        raise ValueError("mutation file, selection regex and tests must be fixed")
    return ["cargo", "mutants", "--no-config", "--all-features", "--file", source_file,
            "--re", expression, "--output", str(output), "--timeout", "120",
            "--jobs", "1", "--", *tail]


def add_bytes(archive: tarfile.TarFile, name: str, data: bytes) -> None:
    member = tarfile.TarInfo(name)
    member.size = len(data)
    member.mode = 0o644
    member.mtime = 0
    archive.addfile(member, io.BytesIO(data))


def run_one(entry: dict[str, Any], output: Path) -> dict[str, Any]:
    crate = entry["crate"]
    source = Path(entry["source_path"]).resolve()
    revision = entry["source_revision"]
    v5.exact_source(source, revision)
    output.mkdir(parents=True, exist_ok=False)
    environment = os.environ.copy()
    environment["CARGO_NET_OFFLINE"] = "true"
    if crate == "tl-mltl":
        environment["TL_MLTL_SOURCE_REVISION"] = revision
        environment["TL_MLTL_SOURCE_STATE"] = "clean"
    discovery_command = ["cargo", "mutants", "--no-config", "--all-features", "--list",
                         "--json", "--file", entry["source_file"]]
    discovered = subprocess.run(discovery_command, cwd=source, env=environment,
                                capture_output=True, check=True)
    discovery_path = output / "discovery.json"
    discovery_path.write_bytes(discovered.stdout)
    raw_output = output / "native"
    argv = command(entry["source_file"], entry["selection_regex"], raw_output,
                   entry["test_tail"])
    process = subprocess.run(argv, cwd=source, env=environment, capture_output=True)
    v5.exact_source(source, revision)
    native_path = raw_output / "mutants.out"
    if not native_path.is_dir():
        raise ValueError(f"{crate}: cargo-mutants emitted no native output")
    archive_path = output / "native.tar.gz"
    with tarfile.open(archive_path, "w:gz") as archive:
        archive.add(native_path, arcname="mutants.out")
        add_bytes(archive, "invocation.stdout", process.stdout)
        add_bytes(archive, "invocation.stderr", process.stderr)
        add_bytes(archive, "invocation.json", json.dumps({
            "discovery_command": discovery_command,
            "mutation_command": argv,
            "exit_code": process.returncode,
            "source_revision": revision,
        }, sort_keys=True).encode())
    return {
        "crate": crate,
        "source_path": str(source),
        "source_revision": revision,
        "source_file": entry["source_file"],
        "selection_regex": entry["selection_regex"],
        "critical_scope": entry["critical_scope"],
        "test_tail": entry["test_tail"],
        "survivor_reviews": entry.get("survivor_reviews", []),
        "discovery_path": str(discovery_path),
        "discovery_sha256": v5.digest(discovery_path.read_bytes()),
        "archive_path": str(archive_path),
        "archive_sha256": v5.digest(archive_path.read_bytes()),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--selection", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    args = parser.parse_args()
    selection = json.loads(args.selection.read_bytes())
    if selection.get("schema") != "tl-mltl.v5-mutation-selection/v1":
        raise ValueError("wrong V5 selection schema")
    args.output_dir.mkdir(parents=True, exist_ok=False)
    entries = [run_one(entry, args.output_dir / entry["crate"])
               for entry in selection["runs"]]
    manifest = {"schema": "tl-mltl.v5-mutation-manifest/v1", "runs": entries}
    manifest_path = args.output_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")
    result = v5.report(manifest, manifest_path.parent)
    (args.output_dir / "report.json").write_text(json.dumps(result, sort_keys=True, indent=2) + "\n")
    return 0 if result["status"] == "passed" else 1


if __name__ == "__main__":
    raise SystemExit(main())
