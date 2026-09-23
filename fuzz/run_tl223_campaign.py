#!/usr/bin/env python3
"""Run or verify the two source-pinned TL-223 libFuzzer populations."""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import re
import shutil
import subprocess
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TARGETS = ("lasso_differential", "partial_valuation_monotone")
MIN_EXECUTIONS = 1_000_000
SOURCE_PATHS = (
    "Cargo.toml",
    "Cargo.lock",
    "src/infinite/mod.rs",
    "fuzz/Cargo.toml",
    "fuzz/Cargo.lock",
    "fuzz/support/infinite.rs",
    "fuzz/run_tl223_campaign.py",
)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(argv: list[str]) -> str:
    return subprocess.check_output(argv, cwd=ROOT, text=True).strip()


def source_digests(target: str) -> dict[str, str]:
    paths = (*SOURCE_PATHS, f"fuzz/fuzz_targets/{target}.rs")
    return {path: digest((ROOT / path).read_bytes()) for path in paths}


def corpus_digests(target: str) -> dict[str, str]:
    directory = ROOT / "fuzz" / "corpus" / target
    records = (directory / "SHA256SUMS").read_text().splitlines()
    result: dict[str, str] = {}
    for record in records:
        match = re.fullmatch(r"([0-9a-f]{64})  ([A-Za-z0-9_.-]+)", record)
        if match is None or match[2] in result:
            raise ValueError("invalid or duplicate corpus digest")
        result[match[2]] = match[1]
    if not result or {path.name for path in directory.iterdir()} != set(result) | {
        "SHA256SUMS"
    }:
        raise ValueError("missing or undeclared corpus member")
    for name, expected in result.items():
        if digest((directory / name).read_bytes()) != expected:
            raise ValueError(f"corpus digest mismatch: {name}")
    return dict(sorted(result.items()))


def classify(
    exit_code: int, log: bytes, requested: int, artifacts: dict[str, str]
) -> tuple[str, int | None, str]:
    matches = re.findall(rb"(?m)^#(\d+)\s+DONE\b", log)
    actual = int(matches[0]) if len(matches) == 1 else None
    if artifacts:
        return "crash_requires_replay", actual, "artifact_present"
    if exit_code != 0:
        return "engine_failed", actual, "nonzero_exit"
    if len(matches) != 1 or actual != requested:
        return "incomplete", actual, "missing_or_short_done_count"
    if actual < MIN_EXECUTIONS:
        return "smoke_only", actual, "below_linear_exit_budget"
    return "complete", actual, "execution_budget"


def run(target: str, output: Path, runs: int, seconds: int, seed: int) -> int:
    if target not in TARGETS or runs <= 0 or seconds <= 0 or seed <= 0:
        raise ValueError("invalid target or budget")
    if output.exists():
        raise ValueError("output directory already exists")
    if command(["git", "status", "--porcelain"]):
        raise ValueError("measurement requires a clean source worktree")
    revision = command(["git", "rev-parse", "HEAD"])
    rustc = command(["rustc", "-Vv"])
    if "nightly" not in rustc:
        raise ValueError("libFuzzer requires the recorded nightly toolchain")
    sources = source_digests(target)
    seeds = corpus_digests(target)
    versions = {
        "rustc": rustc,
        "cargo": command(["cargo", "-V"]),
        "cargo_fuzz": command(["cargo", "fuzz", "-V"]),
    }
    output.mkdir(parents=True)
    artifacts_dir = output / "artifacts"
    artifacts_dir.mkdir()
    with tempfile.TemporaryDirectory(prefix=f"tl223-{target}-") as temporary:
        corpus = Path(temporary) / "corpus"
        corpus.mkdir()
        for name in seeds:
            shutil.copyfile(ROOT / "fuzz" / "corpus" / target / name, corpus / name)
        argv = [
            "cargo", "fuzz", "run", "--sanitizer", "address", target, str(corpus), "--",
            f"-runs={runs}", f"-seed={seed}", f"-max_total_time={seconds}",
            "-max_len=256", f"-artifact_prefix={artifacts_dir}/",
        ]
        start = time.monotonic()
        try:
            result = subprocess.run(
                argv, cwd=ROOT, capture_output=True, timeout=seconds + 300, check=False
            )
            stdout, stderr, exit_code = result.stdout, result.stderr, result.returncode
        except subprocess.TimeoutExpired as error:
            stdout, stderr, exit_code = error.stdout or b"", error.stderr or b"", 124
        elapsed = time.monotonic() - start
    raw = {"stdout.log.gz": gzip.compress(stdout, mtime=0),
           "stderr.log.gz": gzip.compress(stderr, mtime=0)}
    for name, data in raw.items():
        (output / name).write_bytes(data)
    artifacts = {
        path.name: digest(path.read_bytes())
        for path in sorted(artifacts_dir.iterdir()) if path.is_file()
    }
    status, actual, stop = classify(exit_code, stdout + b"\n" + stderr, runs, artifacts)
    report = {
        "schema": "tl-mltl.tl223-fuzz/v1",
        "target": target,
        "source_revision": revision,
        "source_sha256": sources,
        "corpus_sha256": seeds,
        "corpus_manifest_sha256": digest(
            (ROOT / "fuzz" / "corpus" / target / "SHA256SUMS").read_bytes()
        ),
        "engine": "libFuzzer",
        "sanitizer": "address",
        "tool_versions": versions,
        "command": argv,
        "budget": {"runs": runs, "seconds": seconds, "seed": seed},
        "observed": {"executions": actual, "exit_code": exit_code,
                     "elapsed_seconds": round(elapsed, 3), "stop_reason": stop},
        "raw_sha256": {name: digest(data) for name, data in raw.items()},
        "artifact_sha256": artifacts,
        "replay": {"required": bool(artifacts), "confirmed": False},
        "linear_min_executions": MIN_EXECUTIONS,
        "status": status,
    }
    (output / "report.json").write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")
    print(f"{target}: {status}, executions={actual}, exit={exit_code}")
    return 0 if status in {"complete", "smoke_only"} else 1


def verify(directories: list[Path]) -> None:
    if len(directories) != len(TARGETS):
        raise ValueError("both target receipts are required")
    if command(["git", "status", "--porcelain"]):
        raise ValueError("verification requires a clean source worktree")
    revision = command(["git", "rev-parse", "HEAD"])
    seen: set[str] = set()
    for directory in directories:
        report = json.loads((directory / "report.json").read_text())
        target = report["target"]
        if target not in TARGETS or target in seen:
            raise ValueError("duplicate or unknown target")
        seen.add(target)
        if report["schema"] != "tl-mltl.tl223-fuzz/v1":
            raise ValueError("wrong receipt schema")
        if report["source_revision"] != revision or report["source_sha256"] != source_digests(target):
            raise ValueError("source identity mismatch")
        if report["corpus_sha256"] != corpus_digests(target) or report[
            "corpus_manifest_sha256"
        ] != digest((ROOT / "fuzz" / "corpus" / target / "SHA256SUMS").read_bytes()):
            raise ValueError("corpus identity mismatch")
        if report["linear_min_executions"] != MIN_EXECUTIONS:
            raise ValueError("minimum execution count changed")
        for name in ("stdout.log.gz", "stderr.log.gz"):
            if digest((directory / name).read_bytes()) != report["raw_sha256"][name]:
                raise ValueError("raw stream digest mismatch")
        artifacts_dir = directory / "artifacts"
        actual_artifacts = {
            path.name: digest(path.read_bytes())
            for path in sorted(artifacts_dir.iterdir()) if path.is_file()
        }
        if actual_artifacts != report["artifact_sha256"]:
            raise ValueError("artifact identity mismatch")
        stdout = gzip.decompress((directory / "stdout.log.gz").read_bytes())
        stderr = gzip.decompress((directory / "stderr.log.gz").read_bytes())
        requested = report["budget"]["runs"]
        if requested < MIN_EXECUTIONS:
            raise ValueError("budget below Linear exit criterion")
        status, actual, stop = classify(
            report["observed"]["exit_code"], stdout + b"\n" + stderr,
            requested, actual_artifacts
        )
        if (status, actual, stop) != (
            report["status"], report["observed"]["executions"],
            report["observed"]["stop_reason"]
        ) or status != "complete" or report["replay"] != {
            "required": False, "confirmed": False
        }:
            raise ValueError("fuzz execution did not meet the complete gate")
    if seen != set(TARGETS):
        raise ValueError("missing target receipt")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    subcommands = parser.add_subparsers(dest="mode", required=True)
    run_parser = subcommands.add_parser("run")
    run_parser.add_argument("target", choices=TARGETS)
    run_parser.add_argument("--output", required=True, type=Path)
    run_parser.add_argument("--runs", type=int, default=MIN_EXECUTIONS)
    run_parser.add_argument("--seconds", type=int, default=3600)
    run_parser.add_argument("--seed", type=int, default=223)
    verify_parser = subcommands.add_parser("verify")
    verify_parser.add_argument("receipts", nargs=2, type=Path)
    args = parser.parse_args()
    if args.mode == "run":
        raise SystemExit(run(args.target, args.output.resolve(), args.runs,
                             args.seconds, args.seed))
    verify([path.resolve() for path in args.receipts])
    print("TL-223 complete: two million-execution targets, pinned and replayable")


if __name__ == "__main__":
    main()
