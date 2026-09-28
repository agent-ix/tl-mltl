#!/usr/bin/env python3
"""Run and compare exact same-host V9 Criterion baseline/candidate pairs."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import platform
import random
import shutil
import statistics
import subprocess
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
GROUPS = {
    "parse": ("parser_roundtrip", "parser_roundtrip", (
        "bounded_small", "past_small", "infinite_small", "infinite_fairness",
        "bounded_median", "past_median", "infinite_median",
        "bounded_near_node_cap", "past_near_node_cap", "infinite_near_node_cap",
    )),
    "rewrite": ("rewrite_rules", "rewrite_rules", (
        "small_1", "median_24", "near_cap_64",
    )),
    "mltl": ("v9_workloads", "v9_workloads", tuple(
        f"{family}_{scale}" for scale in ("small", "median", "near_cap")
        for family in ("closed", "prefix", "lasso", "fairness", "c2po")
    ) + tuple(
        f"{family}_{scale}" for scale in ("small", "median", "near_cap")
        for family in ("closed_trace", "closed_width")
    )),
}
SAMPLES = 20
THRESHOLD = 0.20
BOOTSTRAPS = 2_000


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(argv: list[str], cwd: Path, env: dict[str, str] | None = None,
            timeout: int = 30) -> str:
    result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True,
                            text=True, timeout=timeout, check=False)
    if result.returncode:
        raise ValueError(f"{' '.join(argv)} failed: {result.stderr.strip()[:200]}")
    return result.stdout.strip()


def optional(argv: list[str]) -> str | None:
    try:
        return command(argv, ROOT, timeout=10)
    except (OSError, ValueError, subprocess.TimeoutExpired):
        return None


def host() -> dict[str, str | None]:
    return {
        "machine": platform.machine(), "system": platform.system(),
        "release": platform.release(), "node": platform.node(),
        "cpu": (optional(["sysctl", "-n", "machdep.cpu.brand_string"])
                or optional(["sysctl", "-n", "hw.model"])
                or platform.processor() or platform.machine()),
        "cpu_count": str(os.cpu_count() or ""),
        "logical_cpus": optional(["sysctl", "-n", "hw.logicalcpu"]),
        "memory_bytes": optional(["sysctl", "-n", "hw.memsize"]),
        "rustc": optional(["rustup", "run", "1.98.1", "rustc", "-Vv"]),
        "cargo": optional(["rustup", "run", "1.98.1", "cargo", "-V"]),
    }


def source(path: Path, name: str) -> dict[str, str]:
    path = path.resolve()
    if command(["git", "status", "--porcelain"], path):
        raise ValueError(f"dirty {name} source: {path}")
    revision = command(["git", "rev-parse", "HEAD"], path)
    if len(revision) != 40:
        raise ValueError(f"non-exact {name} revision")
    return {"path": str(path), "revision": revision,
            "src_tree": command(["git", "rev-parse", "HEAD:src"], path),
            "manifest_sha256": sha256((path / "Cargo.toml").read_bytes()),
            "lock_sha256": sha256((path / "Cargo.lock").read_bytes())}


def harness(candidate: Path, name: str) -> dict[str, str]:
    bench, _, _ = GROUPS[name]
    paths = [candidate / "benches" / f"{bench}.rs"]
    if name == "parse":
        paths += [candidate / "benches" / "inputs" / "SHA256SUMS"]
        paths += sorted((candidate / "benches" / "inputs").glob("*.txt"))
    else:
        paths += [candidate / "benches" / "input-digests.json"]
    return {str(path.relative_to(candidate)): sha256(path.read_bytes()) for path in paths}


def stage_digest(stage: Path) -> str:
    """Bind every staged baseline source byte, including the copied harness."""
    digest = hashlib.sha256()
    for path in sorted(file for file in stage.rglob("*") if file.is_file()):
        relative = path.relative_to(stage).as_posix().encode()
        content = path.read_bytes()
        digest.update(len(relative).to_bytes(8, "little"))
        digest.update(relative)
        digest.update(len(content).to_bytes(8, "little"))
        digest.update(content)
    return digest.hexdigest()


def stage_baseline(baseline: Path, candidate: Path, stage: Path, name: str) -> None:
    bench, _, _ = GROUPS[name]
    manifest = tomllib.loads((baseline / "Cargo.toml").read_text())
    if ("criterion" not in manifest.get("dev-dependencies", {}) or
            not any(item.get("name") == bench and item.get("harness") is False
                    for item in manifest.get("bench", []))):
        raise ValueError(
            f"{name} baseline predates its Criterion workload/API; "
            "select an exact feature-compatible baseline revision"
        )
    if stage.exists():
        raise ValueError(f"baseline stage already exists: {stage}")
    stage.mkdir(parents=True)
    tracked = subprocess.run(["git", "ls-files", "-z"], cwd=baseline,
                             capture_output=True, check=True).stdout
    for encoded in tracked.split(b"\0"):
        if not encoded:
            continue
        relative = Path(os.fsdecode(encoded))
        target = stage / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(baseline / relative, target)
    shutil.rmtree(stage / "benches", ignore_errors=True)
    shutil.copytree(candidate / "benches", stage / "benches")
    if harness(stage, name) != harness(candidate, name):
        raise ValueError(f"{name} copied benchmark harness or inputs changed")
    for file in ("Cargo.toml", "Cargo.lock"):
        if sha256((stage / file).read_bytes()) != sha256((baseline / file).read_bytes()):
            raise ValueError(f"{name} baseline {file} changed while staging")


def samples(path: Path) -> dict:
    raw = json.loads((path / "sample.json").read_text())
    counts, times = raw["iters"], raw["times"]
    if (len(counts) != SAMPLES or len(times) != SAMPLES or
            any(type(n) not in (int, float) or n <= 0 or not math.isfinite(n)
                for n in counts + times)):
        raise ValueError(f"missing or vacuous Criterion samples: {path}")
    values = [total / count for total, count in zip(times, counts, strict=True)]
    return {"sample_ns": values, "sample_count": SAMPLES,
            "median_ns": statistics.median(values),
            "variance_ns2": statistics.variance(values),
            "sample_sha256": sha256((path / "sample.json").read_bytes()),
            "estimates_sha256": sha256((path / "estimates.json").read_bytes())}


def bootstrap_change(baseline: list[float], candidate: list[float], seed: str) -> tuple[float, float]:
    rng = random.Random(int(sha256(seed.encode())[:16], 16))
    ratios = []
    for _ in range(BOOTSTRAPS):
        base = statistics.median(rng.choices(baseline, k=len(baseline)))
        current = statistics.median(rng.choices(candidate, k=len(candidate)))
        ratios.append(current / base - 1)
    ratios.sort()
    return ratios[int(BOOTSTRAPS * 0.025)], ratios[int(BOOTSTRAPS * 0.975)]


def classify_pair(baseline: dict, candidate: dict, case_id: str) -> dict:
    ratio = candidate["median_ns"] / baseline["median_ns"] - 1
    low, high = bootstrap_change(baseline["sample_ns"], candidate["sample_ns"], case_id)
    status = ("repeat_required_above_20pct" if low > THRESHOLD else
              "inconclusive_threshold_overlap" if high > THRESHOLD else
              "below_20pct_threshold")
    return {"baseline": baseline, "candidate": candidate,
            "median_change_ratio": ratio, "change_95pct_ci_ratio": [low, high],
            "status": status}


def validate_pair(path: Path, pair: dict) -> None:
    if pair.get("schema") != "tl-mltl.v9-criterion-pair/v1":
        raise ValueError(f"invalid V9 pair schema: {path}")
    if set(pair.get("crates", {})) != set(GROUPS):
        raise ValueError(f"incomplete V9 crate population: {path}")
    for name, (bench, _, cases) in GROUPS.items():
        crate = pair["crates"][name]
        staged = path / name / "baseline_source"
        if stage_digest(staged) != crate["baseline_stage_sha256"]:
            raise ValueError(f"V9 staged baseline source changed: {name}")
        if harness(staged, name) != crate["copied_harness_sha256"]:
            raise ValueError(f"V9 staged harness changed: {name}")
        baseline = crate["baseline_source"]
        for file, field in (("Cargo.toml", "manifest_sha256"),
                            ("Cargo.lock", "lock_sha256")):
            if sha256((staged / file).read_bytes()) != baseline[field]:
                raise ValueError(f"V9 baseline {file} changed: {name}")
        if set(crate["cases"]) != set(cases):
            raise ValueError(f"incomplete V9 case population: {name}")
        for side in ("baseline", "candidate"):
            run = crate[f"{side}_run"]
            expected = ["cargo", "bench", "--locked", "--offline",
                        *(["--features", "infinite-trace"] if name == "mltl" else []),
                        "--bench", bench, "--", "--noplot"]
            if run["argv"] != expected or run["exit_code"] != 0:
                raise ValueError(f"invalid V9 benchmark invocation: {name}/{side}")
            log = path / name / f"{side}.log"
            if sha256(log.read_bytes()) != run["log_sha256"]:
                raise ValueError(f"V9 raw log changed: {name}/{side}")


def measure(args: argparse.Namespace) -> int:
    pair_dir = args.pair_dir.resolve()
    if pair_dir.exists():
        raise ValueError("pair output must be fresh; stale Criterion data cannot be reused")
    pair_dir.mkdir(parents=True)
    cargo_home = args.cargo_home.resolve()
    if not cargo_home.is_dir():
        raise ValueError("a provisioned writable Cargo home is required")
    rustc = command(["rustup", "which", "rustc", "--toolchain", "1.98.1"], ROOT)
    environment = os.environ.copy()
    environment.update({"CARGO_HOME": str(cargo_home), "RUSTC": rustc,
                        "PATH": f"{Path(rustc).parent}:{environment['PATH']}",
                        "TMPDIR": "/private/tmp"})
    before = host()
    entries = {}
    for name, (bench, group, cases) in GROUPS.items():
        baseline = getattr(args, f"baseline_{name}").resolve()
        candidate = getattr(args, f"candidate_{name}").resolve()
        base_source, current_source = source(baseline, name), source(candidate, name)
        if baseline == candidate:
            raise ValueError(f"{name} baseline and candidate paths must differ")
        staged = pair_dir / name / "baseline_source"
        stage_baseline(baseline, candidate, staged, name)
        source_sets = {"baseline": (staged, base_source),
                       "candidate": (candidate, current_source)}
        row = {"baseline_source": base_source, "candidate_source": current_source,
               "copied_harness_sha256": harness(candidate, name),
               "baseline_stage_sha256": stage_digest(staged), "cases": {}}
        for side, (source_dir, source_id) in source_sets.items():
            target = pair_dir / name / f"{side}_target"
            run_env = environment | {"CARGO_TARGET_DIR": str(target)}
            if name == "mltl":
                run_env.update({"TL_MLTL_SOURCE_REVISION": source_id["revision"],
                                "TL_MLTL_SOURCE_STATE": "clean"})
            argv = ["cargo", "bench", "--locked", "--offline",
                    *(["--features", "infinite-trace"] if name == "mltl" else []),
                    "--bench", bench, "--", "--noplot"]
            result = subprocess.run(argv, cwd=source_dir, env=run_env,
                                    capture_output=True, timeout=args.timeout, check=False)
            log = pair_dir / name / f"{side}.log"
            log.write_bytes(result.stdout + result.stderr)
            row[f"{side}_run"] = {"argv": argv, "exit_code": result.returncode,
                                  "log_sha256": sha256(log.read_bytes())}
            if result.returncode:
                entries[name] = row
                (pair_dir / "pair.json").write_text(json.dumps({
                    "schema": "tl-mltl.v9-criterion-pair/v1", "host_before": before,
                    "host_after": host(), "crates": entries, "status": "incomplete",
                    "reason": f"{name}_{side}_benchmark_failed",
                }, indent=2, sort_keys=True) + "\n")
                return 1
            for case in cases:
                source_sample = target / "criterion" / group / case / "new"
                retained = pair_dir / "samples" / name / case / side
                retained.mkdir(parents=True)
                for file in ("sample.json", "estimates.json"):
                    shutil.copy2(source_sample / file, retained / file)
                row["cases"].setdefault(case, {})[side] = samples(retained)
        entries[name] = row
    after = host()
    result = {"schema": "tl-mltl.v9-criterion-pair/v1", "host_before": before,
              "host_after": after, "crates": entries,
              "status": "measured" if before == after and all(before[key] for key in (
                  "machine", "system", "release", "node", "cpu", "cpu_count",
                  "rustc", "cargo")) else "inconclusive_host"}
    (pair_dir / "pair.json").write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": result["status"], "pair": str(pair_dir)}, sort_keys=True))
    return 0 if result["status"] == "measured" else 1


def report(args: argparse.Namespace) -> int:
    paths = [path.resolve() for path in args.pair_dir]
    if len(set(paths)) != len(paths):
        raise ValueError("paired directories must be distinct")
    pairs = [json.loads((path / "pair.json").read_text()) for path in paths]
    for path, pair in zip(paths, pairs, strict=True):
        if pair.get("status") != "incomplete":
            validate_pair(path, pair)
    if any(pair.get("status") == "incomplete" for pair in pairs):
        result = {"schema": "tl-mltl.v9-criterion-report/v1", "status": "incomplete",
                  "reason": "a_baseline_or_candidate_benchmark_failed",
                  "pair_dirs": [str(path) for path in paths], "cases": []}
    elif not pairs:
        result = {"schema": "tl-mltl.v9-criterion-report/v1", "status": "incomplete",
                  "reason": "baseline_candidate_pairs_not_measured", "pairs": [], "cases": []}
    else:
        first = pairs[0]
        source_ids = {name: (first["crates"][name]["baseline_source"]["revision"],
                             first["crates"][name]["candidate_source"]["revision"])
                      for name in GROUPS}
        comparable = all(pair.get("status") == "measured" and
                         pair.get("host_before") == pair.get("host_after") == first["host_before"]
                         and all(pair["crates"][name]["copied_harness_sha256"] ==
                                 first["crates"][name]["copied_harness_sha256"] and
                                 (pair["crates"][name]["baseline_source"]["revision"],
                                  pair["crates"][name]["candidate_source"]["revision"]) ==
                                 source_ids[name] for name in GROUPS)
                         for pair in pairs)
        cases = []
        if comparable:
            for name, (_, _, expected_cases) in GROUPS.items():
                for case in expected_cases:
                    runs = []
                    for path, pair in zip(paths, pairs, strict=True):
                        row = pair["crates"][name]["cases"][case]
                        retained = path / "samples" / name / case
                        for side in ("baseline", "candidate"):
                            if samples(retained / side) != row[side]:
                                raise ValueError(f"raw Criterion distribution changed: {name}/{case}/{side}")
                        runs.append(classify_pair(row["baseline"], row["candidate"],
                                                  f"{name}/{case}/{path.name}"))
                    confirmed = sum(run["status"] == "repeat_required_above_20pct"
                                    for run in runs)
                    status = ("confirmed_above_20pct" if confirmed >= 2 else
                              "repeat_required" if confirmed == 1 and len(runs) < 3 else
                              "one_run_spike" if confirmed == 1 else
                              "inconclusive_overlap" if any(run["status"] ==
                                  "inconclusive_threshold_overlap" for run in runs) else
                              "below_20pct_threshold")
                    cases.append({"crate": name, "case": case, "runs": runs, "status": status})
        status = ("inconclusive_host" if not comparable else
                  "incomplete" if len(pairs) < 2 else
                  "finding_required" if any(row["status"] == "confirmed_above_20pct"
                                            for row in cases) else
                  "repeat_required" if any(row["status"] == "repeat_required"
                                            for row in cases) else
                  "inconclusive_noise" if any(row["status"] in {
                      "inconclusive_overlap", "one_run_spike"}
                                              for row in cases) else "passed")
        result = {"schema": "tl-mltl.v9-criterion-report/v1", "status": status,
                  "threshold": THRESHOLD, "required_pairs": 2,
                  "pair_dirs": [str(path) for path in paths],
                  "source_revisions": source_ids, "host": first.get("host_before"),
                  "cases": cases, "pair_count": len(pairs),
                  "reason": "host_or_toolchain_mismatch" if not comparable else
                  "repeat_pair_missing" if len(pairs) < 2 else None}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": result["status"], "output": str(args.output)}, sort_keys=True))
    return 0 if result["status"] == "passed" else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="action", required=True)
    measure_parser = sub.add_parser("measure")
    for name in GROUPS:
        measure_parser.add_argument(f"--baseline-{name}", type=Path, required=True)
        measure_parser.add_argument(f"--candidate-{name}", type=Path, required=True)
    measure_parser.add_argument("--cargo-home", type=Path, required=True)
    measure_parser.add_argument("--pair-dir", type=Path, required=True)
    measure_parser.add_argument("--timeout", type=int, default=900)
    report_parser = sub.add_parser("report")
    report_parser.add_argument("--pair-dir", type=Path, action="append", default=[])
    report_parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    return measure(args) if args.action == "measure" else report(args)


if __name__ == "__main__":
    sys.exit(main())
