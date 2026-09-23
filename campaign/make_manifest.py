#!/usr/bin/env python3
"""Construct a candidate V1 manifest from exact local source and corpus bytes."""

from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path

from v1_campaign import COMMAND_CONTRACTS, NATIVE_CONTRACTS, REQUIRED, SOURCE_NAMES, git_revision, sha256


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


def make_manifest(repos_root: Path, live_r2u2_source: Path | None = None,
                  v7_cargo_home: Path | None = None,
                  v8_cargo_home: Path | None = None,
                  v5_selection: Path | None = None,
                  v9_pair_dirs: list[Path] | None = None) -> dict:
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
    if v5_selection is not None:
        selected = v5_selection.resolve()
        inputs["v5_selection"] = {"path": str(selected),
                                  "sha256": sha256(selected.read_bytes())}
    for index, pair in enumerate(v9_pair_dirs or []):
        path = pair.resolve() / "pair.json"
        inputs[f"v9_pair_{index}"] = {"path": str(path.resolve()),
                                      "sha256": sha256(path.read_bytes())}
    lanes = []
    for milestone, lane_ids in REQUIRED.items():
        for lane_id in lane_ids:
            if lane_id in NATIVE_CONTRACTS:
                seed = ({"kind": "fixed", "value": 181} if lane_id == "fuzz_replay"
                        else {"kind": "none", "reason": "symbolic_no_random_seed"})
                lanes.append({
                    "id": lane_id, "milestone": milestone, "mode": "native",
                    "seed": seed,
                })
                continue
            if lane_id not in COMMAND_CONTRACTS:
                continue
            if lane_id == "live_r2u2" and live_r2u2_source is None:
                continue
            if lane_id == "embedded_miri_limits" and v7_cargo_home is None:
                continue
            if lane_id == "coverage" and v8_cargo_home is None:
                continue
            if lane_id == "mutation_population" and v5_selection is None:
                continue
            if lane_id == "performance" and not v9_pair_dirs:
                continue
            repo, parser, argv = COMMAND_CONTRACTS[lane_id]
            lane = {
                "id": lane_id, "milestone": milestone, "mode": "command",
                "repo": repo, "parser": parser, "argv": argv,
                "seed": {"kind": "none", "reason": "deterministic_cargo_test"},
            }
            if lane_id == "live_r2u2":
                lane["target_source"] = str(live_r2u2_source.resolve())
            if lane_id == "embedded_miri_limits":
                lane["cargo_home"] = str(v7_cargo_home.resolve())
                lane["timeout_seconds"] = 3600
            if lane_id == "coverage":
                lane["cargo_home"] = str(v8_cargo_home.resolve())
                lane["timeout_seconds"] = 7200
            if lane_id == "mutation_population":
                lane["selection_path"] = str(v5_selection.resolve())
                lane["timeout_seconds"] = 7200
                lane["seed"] = {"kind": "none", "reason": "fixed_mutant_selection"}
            if lane_id == "performance":
                lane["pair_dirs"] = [str(path.resolve()) for path in v9_pair_dirs]
                lane["timeout_seconds"] = 600
                lane["seed"] = {"kind": "none", "reason": "fixed_criterion_pairs"}
            lanes.append(lane)
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
    parser.add_argument(
        "--live-r2u2-source", type=Path,
        help="Explicitly opt into the exact-pin foreign target run",
    )
    parser.add_argument(
        "--v7-cargo-home", type=Path,
        help="Explicitly opt into fresh embedded/Miri probes with a provisioned Cargo home",
    )
    parser.add_argument(
        "--v8-cargo-home", type=Path,
        help="Explicitly opt into fresh four-crate llvm-cov with a provisioned Cargo home",
    )
    parser.add_argument(
        "--v5-selection", type=Path,
        help="Opt into fresh four-crate mutation with a fixed source-pinned selection JSON",
    )
    parser.add_argument(
        "--v9-pair-dir", type=Path, action="append", default=[],
        help="Include one retained same-host baseline/candidate Criterion pair",
    )
    args = parser.parse_args()
    manifest = make_manifest(args.repos_root, args.live_r2u2_source,
                             args.v7_cargo_home, args.v8_cargo_home, args.v5_selection,
                             args.v9_pair_dir)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(manifest, sort_keys=True, indent=2) + "\n")


if __name__ == "__main__":
    main()
