#!/usr/bin/env python3
"""Construct a candidate V1 manifest from exact local source and corpus bytes."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from v1_campaign import COMMAND_CONTRACTS, REQUIRED, SOURCE_NAMES, git_revision, sha256


def corpus_paths(repo: Path) -> list[Path]:
    result = subprocess.run(
        ["git", "ls-files", "-z"], cwd=repo, capture_output=True, check=True
    )
    selected = []
    for encoded in result.stdout.split(b"\0"):
        if not encoded:
            continue
        relative = Path(encoded.decode())
        parts = relative.parts
        if "corpus" in parts or parts[:2] == ("benches", "inputs"):
            selected.append(repo / relative)
    return sorted(selected)


def make_manifest(repos_root: Path) -> dict:
    sources = {}
    inputs = {}
    for name in SOURCE_NAMES:
        repo = (repos_root / name).resolve()
        sources[name] = {"path": str(repo), "revision": git_revision(repo)}
        for path in corpus_paths(repo):
            relative = path.relative_to(repo).as_posix()
            inputs[f"{name}/{relative}"] = {
                "path": str(path), "sha256": sha256(path.read_bytes()),
            }
    lanes = []
    for milestone, lane_ids in REQUIRED.items():
        for lane_id in lane_ids:
            if lane_id not in COMMAND_CONTRACTS:
                continue
            repo, parser, argv = COMMAND_CONTRACTS[lane_id]
            lanes.append({
                "id": lane_id, "milestone": milestone, "mode": "command",
                "repo": repo, "parser": parser, "argv": argv,
                "seed": {"kind": "none", "reason": "deterministic_cargo_test"},
            })
    return {
        "schema": "tl-mltl.v1-campaign-manifest/v1",
        "sources": sources,
        "inputs": dict(sorted(inputs.items())),
        "lanes": lanes,
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repos-root", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    manifest = make_manifest(args.repos_root)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")


if __name__ == "__main__":
    main()
