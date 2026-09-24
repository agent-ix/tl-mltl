#!/usr/bin/env python3
"""Measure production Rust line and branch coverage with pinned llvm-cov tools."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import subprocess
import sys
from pathlib import Path

CRATES = ("syntax", "parse", "mltl", "rewrite")
FEATURES = {
    "syntax": (("core", ("--no-default-features",)),
               ("alloc", ("--no-default-features", "--features", "alloc")),
               ("serde", ("--all-features",))),
    "parse": (("default", ("--all-features",)),),
    "mltl": (("default", ()), ("infinite", ("--features", "infinite-trace"))),
    "rewrite": (("default", ()), ("infinite", ("--features", "infinite-trace"))),
}
CRITICAL_PREFIXES = {
    "syntax": ("src/future.rs", "src/formula/infinite.rs", "src/contracts/reader.rs"),
    "parse": ("src/parser.rs", "src/formatter.rs", "src/dialect/v4.rs",
              "src/infinite.rs", "src/lexer.rs"),
    "mltl": ("src/future/evaluate.rs", "src/past/mod.rs",
             "src/infinite/periodic.rs", "src/infinite/export.rs",
             "src/mapping/past.rs", "src/wire/"),
    "rewrite": ("src/engine/future.rs", "src/engine/past.rs",
                "src/infinite.rs", "src/report.rs", "src/replay.rs",
                "src/disposition.rs"),
}
EXPECTED_CRITICAL = {
    "syntax": {"core": CRITICAL_PREFIXES["syntax"][:2],
               "alloc": CRITICAL_PREFIXES["syntax"][:2],
               "serde": CRITICAL_PREFIXES["syntax"]},
    "parse": {"default": CRITICAL_PREFIXES["parse"]},
    "mltl": {"default": ("src/future/evaluate.rs", "src/past/mod.rs",
                         "src/mapping/past.rs", "src/wire/command.rs",
                         "src/wire/common.rs", "src/wire/trace.rs"),
             "infinite": ("src/future/evaluate.rs", "src/past/mod.rs",
                          "src/infinite/periodic.rs", "src/infinite/export.rs",
                          "src/mapping/past.rs", "src/wire/command.rs",
                          "src/wire/common.rs", "src/wire/trace.rs")},
    "rewrite": {"default": ("src/engine/future.rs", "src/engine/past.rs",
                            "src/report.rs", "src/replay.rs", "src/disposition.rs"),
                "infinite": CRITICAL_PREFIXES["rewrite"]},
}
TOOLCHAIN = "nightly"
ZERO_BRANCH_POLICY_FILES = {"src/dialect/v4.rs", "src/disposition.rs"}
REVIEW_FIELDS = {"run", "file", "line", "column", "true_count", "false_count",
                 "source_file_sha256", "reason", "reviewer"}


def unique_pairs(pairs: list[tuple[str, object]]) -> dict:
    """Reject duplicate JSON keys, including in a supplied review record."""
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate V8 review key: {key}")
        result[key] = value
    return result


def gap_key(gap: dict) -> tuple:
    """Bind a review to one run, source location, and measured missing side."""
    return tuple(gap[field] for field in
                 ("run", "file", "line", "column", "true_count", "false_count"))


def validate_reviews(reviews: object, gaps: list[dict], roots: dict[str, Path]) -> set[tuple]:
    """Check explicit human infeasibility decisions against the live source graph."""
    if type(reviews) is not list:
        raise ValueError("V8 reviews must be a list")
    observed = {gap_key(gap) for gap in gaps}
    accepted = set()
    for review in reviews:
        if type(review) is not dict or set(review) != REVIEW_FIELDS:
            raise ValueError("V8 review shape mismatch")
        if (type(review["run"]) is not str or type(review["file"]) is not str or
                any(type(review[field]) is not int or review[field] < 0 for field in
                    ("line", "column", "true_count", "false_count"))):
            raise ValueError("V8 review location malformed")
        key = gap_key(review)
        if key not in observed:
            raise ValueError("V8 review names unknown or stale gap")
        if key in accepted:
            raise ValueError("duplicate V8 review")
        reason = review["reason"]
        reviewer = review["reviewer"]
        if (type(reason) is not str or reason != reason.strip() or len(reason) < 40 or
                len(reason.split()) < 6 or
                any(marker in reason.lower() for marker in ("tbd", "todo", "placeholder"))):
            raise ValueError("V8 review needs a substantive infeasibility reason")
        if (type(reviewer) is not str or reviewer != reviewer.strip() or
                len(reviewer) < 3 or len(reviewer) > 128 or
                reviewer.lower() in ("unknown", "anonymous", "reviewer", "tbd")):
            raise ValueError("V8 review needs a named reviewer")
        source = roots[review["run"].split("-", 1)[0]] / review["file"]
        source_digest = hashlib.sha256(source.read_bytes()).hexdigest()
        if review["source_file_sha256"] != source_digest:
            raise ValueError("V8 review source file digest is stale")
        accepted.add(key)
    return accepted


def parse_prep_command() -> list[str]:
    """Build the example invoked by parse's shared-assurance tests."""
    return ["cargo", "build", "--example", "fuzz_campaign", "--all-features",
            "--locked", "--offline"]


def parse_prep_binary(target: Path) -> Path:
    return target / "debug" / "examples" / ("fuzz_campaign.exe" if os.name == "nt" else
                                           "fuzz_campaign")


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def tool_path(binary: str) -> Path:
    result = subprocess.run(("rustup", "which", "rustc", "--toolchain", TOOLCHAIN),
                            capture_output=True, text=True, check=True)
    rustc = Path(result.stdout.strip()).resolve()
    if binary == "rustc":
        return rustc
    matches = list((rustc.parent.parent / "lib" / "rustlib").glob(f"*/bin/{binary}"))
    if len(matches) != 1:
        raise ValueError(f"expected one {binary} in {TOOLCHAIN} toolchain")
    return matches[0]


def source(path: Path, name: str) -> str:
    if f'name = "tl-{name}"' not in (path / "Cargo.toml").read_text():
        raise ValueError(f"wrong Cargo package: {path}")
    revision = subprocess.run(("git", "rev-parse", "HEAD"), cwd=path,
                              capture_output=True, text=True, check=True).stdout.strip()
    status = subprocess.run(("git", "status", "--porcelain"), cwd=path,
                            capture_output=True, text=True, check=True).stdout.strip()
    if status:
        raise ValueError(f"dirty source: {path}")
    return revision


def covered_file(path: str, root: Path) -> str | None:
    file = Path(path).resolve()
    try:
        relative = file.relative_to(root.resolve())
    except ValueError:
        return None
    if len(relative.parts) < 2 or relative.parts[0] != "src" or file.suffix != ".rs":
        return None
    return relative.as_posix()


def classify_export(raw: dict, root: Path) -> dict:
    if raw.get("type") != "llvm.coverage.json.export" or not isinstance(raw.get("data"), list):
        raise ValueError("not an llvm-cov JSON export")
    if len(raw["data"]) != 1:
        raise ValueError("expected one coverage compilation unit")
    files = {}
    for item in raw["data"][0]["files"]:
        relative = covered_file(item["filename"], root)
        if relative is None:
            continue
        if relative in files:
            raise ValueError(f"duplicate production coverage file: {relative}")
        summary = item["summary"]
        lines = summary["lines"]
        branches = summary["branches"]
        functions = summary.get("functions", {})
        regions = summary.get("regions", {})
        if not all(isinstance(v, int) and v >= 0 for v in
                   (lines["count"], lines["covered"], branches["count"], branches["covered"])):
            raise ValueError(f"malformed coverage counts: {relative}")
        if lines["covered"] > lines["count"] or branches["covered"] > branches["count"]:
            raise ValueError(f"impossible coverage counts: {relative}")
        for metric, counts in (("functions", functions), ("regions", regions)):
            if counts and (not all(type(counts.get(key)) is int and counts[key] >= 0
                                   for key in ("count", "covered")) or
                           counts["covered"] > counts["count"]):
                raise ValueError(f"impossible {metric} coverage counts: {relative}")
        if branches["count"] and not item["branches"]:
            raise ValueError(f"missing detailed branch population: {relative}")
        branch_sites = {}
        for branch in item["branches"]:
            if len(branch) != 9 or any(type(value) is not int for value in branch):
                raise ValueError(f"malformed branch location: {relative}")
            if branch[4] < 0 or branch[5] < 0:
                raise ValueError(f"negative branch execution count: {relative}")
            # LLVM repeats a source span for distinct instantiations. Sum those
            # counts before deciding whether either side is uncovered.
            site = tuple(branch[:4] + branch[6:])
            counts = branch_sites.setdefault(site, [0, 0])
            counts[0] += branch[4]
            counts[1] += branch[5]
        # One source span can represent multiple monomorphized branches. LLVM's
        # summary counts those instances, so unique source sites are not an
        # upper bound on the summary population.
        raw_hit_sides = sum(int(branch[4] > 0) + int(branch[5] > 0)
                            for branch in item["branches"])
        if (branches["count"] > 2 * len(item["branches"]) or
                branches["covered"] > raw_hit_sides):
            raise ValueError(f"branch summary exceeds detailed population: {relative}")
        uncovered = []
        if branches["count"]:
            for site, (true_count, false_count) in branch_sites.items():
                if true_count == 0 or false_count == 0:
                    location = {"line": site[0], "column": site[1],
                                "true_count": true_count, "false_count": false_count}
                    if location not in uncovered:
                        uncovered.append(location)
        # LLVM's summary counts monomorphized branches separately. When both
        # sides were exercised at each source span but an instance is still
        # missing a side, the export has no function identity that can bind
        # the deficit to one reviewable source location. Keep the summary gap
        # open rather than assigning it to every zero-count duplicate record.
        unattributed_missing_sides = (
            branches["count"] - branches["covered"] if not uncovered else 0
        )
        files[relative] = {"lines": {"count": lines["count"], "covered": lines["covered"]},
                           "branches": {"count": branches["count"],
                                        "covered": branches["covered"]},
                           "functions": {key: functions.get(key, 0)
                                         for key in ("count", "covered")},
                           "regions": {key: regions.get(key, 0)
                                       for key in ("count", "covered")},
                           "uncovered_branch_locations": uncovered,
                           "unattributed_missing_sides": unattributed_missing_sides}
    if not files or not any(item["branches"]["count"] for item in files.values()):
        raise ValueError("no production branches were measured")
    return {"files": files, "totals": {
        metric: {key: sum(file[metric][key] for file in files.values())
                 for key in ("count", "covered")}
        for metric in ("lines", "branches")}}


def critical_census(coverage: dict, name: str, feature: str) -> tuple[dict, list, list]:
    """Name each required critical source file and every uncovered branch."""
    files = {file: details["branches"] for file, details in coverage["files"].items()
             if any(file.startswith(prefix) for prefix in CRITICAL_PREFIXES[name])}
    missing = []
    for file in EXPECTED_CRITICAL[name][feature]:
        detail = coverage["files"].get(file)
        if detail is None:
            missing.append(file)
        elif detail["branches"]["count"] == 0:
            # LLVM emits no branch records for these const match policy files.
            # Require executed functions, regions, and lines instead of treating
            # an unmeasured file as complete.
            if file not in ZERO_BRANCH_POLICY_FILES or any(
                detail[metric]["covered"] == 0 for metric in ("lines", "functions", "regions")
            ):
                missing.append(file)
    gaps = [{"file": file, **location}
            for file, details in coverage["files"].items() if file in files
            for location in details["uncovered_branch_locations"]]
    return files, missing, gaps


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in CRATES:
        parser.add_argument(f"--{name}", type=Path, required=True)
    parser.add_argument("--cargo-home", type=Path, required=True)
    parser.add_argument("--raw-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=1800)
    parser.add_argument("--reviews", type=Path,
                        help="JSON list of named, source-bound infeasibility reviews")
    parser.add_argument("--only", choices=[f"{name}-{feature}" for name in CRATES
                                            for feature, _ in FEATURES[name]])
    args = parser.parse_args()
    roots = {name: getattr(args, name).resolve() for name in CRATES}
    revisions = {name: source(root, name) for name, root in roots.items()}
    cargo_locks = {name: digest(root / "Cargo.lock") for name, root in roots.items()}
    rustc = tool_path("rustc")
    llvm_cov = tool_path("llvm-cov")
    llvm_profdata = tool_path("llvm-profdata")
    if not args.cargo_home.is_dir() or not all(path.is_file() for path in
                                               (rustc, llvm_cov, llvm_profdata)):
        raise ValueError("pinned toolchain or writable Cargo home unavailable")
    tools = {"rustc": subprocess.run((str(rustc), "--version"), capture_output=True,
                                     text=True, check=True).stdout.strip(),
             "llvm_cov": subprocess.run((str(llvm_cov), "--version"), capture_output=True,
                                        text=True, check=True).stdout.strip(),
             "llvm_profdata": subprocess.run((str(llvm_profdata), "--version"),
                                             capture_output=True, text=True,
                                             check=True).stdout.strip(),
             "cargo_llvm_cov": subprocess.run(("cargo", "llvm-cov", "--version"),
                                              capture_output=True, text=True,
                                              check=True).stdout.strip()}
    args.raw_dir.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    env.update({"CARGO_HOME": str(args.cargo_home.resolve()),
                "RUSTC": str(rustc), "LLVM_COV": str(llvm_cov),
                "LLVM_PROFDATA": str(llvm_profdata), "TMPDIR": "/private/tmp",
                "PATH": f"{rustc.parent}:{env['PATH']}"})
    # Git dependencies resolve from the exact local source graph in offline mode.
    env["GIT_CONFIG_COUNT"] = "4"
    for index, (name, root) in enumerate(roots.items()):
        env[f"GIT_CONFIG_KEY_{index}"] = f"url.file://{root}.insteadOf"
        env[f"GIT_CONFIG_VALUE_{index}"] = f"https://github.com/agent-ix/tl-{name}.git"
    results = []
    for name in CRATES:
        for feature, flags in FEATURES[name]:
            key = f"{name}-{feature}"
            if args.only and args.only != key:
                continue
            export = args.raw_dir / f"{key}.json"
            stdout = args.raw_dir / f"{key}.stdout"
            stderr = args.raw_dir / f"{key}.stderr"
            command = ("cargo", "llvm-cov", "--branch", "--json", "--output-path",
                       str(export), "--lib", "--tests", *flags, "--locked", "--offline")
            target = args.raw_dir / f"{key}.target"
            lane_env = env | {"CARGO_TARGET_DIR": str(target)}
            prep = None
            if name == "parse":
                prep_argv = parse_prep_command()
                prep_stdout = args.raw_dir / f"{key}.prep.stdout"
                prep_stderr = args.raw_dir / f"{key}.prep.stderr"
                try:
                    prepared = subprocess.run(prep_argv, cwd=roots[name], env=lane_env,
                                              capture_output=True, timeout=args.timeout)
                    prep_code, prep_out, prep_err = (prepared.returncode,
                                                     prepared.stdout, prepared.stderr)
                except subprocess.TimeoutExpired as failure:
                    prep_code, prep_out, prep_err = (124, failure.stdout or b"",
                                                    failure.stderr or b"")
                prep_stdout.write_bytes(prep_out)
                prep_stderr.write_bytes(prep_err)
                prep = {"argv": prep_argv, "exit_code": prep_code,
                        "raw": {kind: {"path": str(path.resolve()),
                                       "sha256": digest(path)}
                                for kind, path in (("stdout", prep_stdout),
                                                   ("stderr", prep_stderr))}}
                binary = parse_prep_binary(target)
                if prep_code == 0 and binary.is_file():
                    prep["binary"] = {"path": str(binary.resolve()),
                                      "sha256": digest(binary)}
                else:
                    stdout.write_bytes(b"")
                    stderr.write_bytes(b"")
                    results.append({"id": key, "repo": name, "feature": feature,
                                    "source_revision": revisions[name], "argv": command,
                                    "exit_code": 125,
                                    "raw": {kind: {"path": str(path.resolve()),
                                                   "sha256": digest(path)}
                                            for kind, path in (("stdout", stdout),
                                                               ("stderr", stderr))},
                                    "prep": prep, "status": "failed",
                                    "reason": "parse_example_prep_failed"})
                    continue
            try:
                run = subprocess.run(command, cwd=roots[name], env=lane_env,
                                     capture_output=True, timeout=args.timeout)
                code, out, err = run.returncode, run.stdout, run.stderr
            except subprocess.TimeoutExpired as failure:
                code, out, err = 124, failure.stdout or b"", failure.stderr or b""
            stdout.write_bytes(out)
            stderr.write_bytes(err)
            result = {"id": key, "repo": name, "feature": feature,
                      "source_revision": revisions[name], "argv": command,
                      "exit_code": code, "raw": {kind: {"path": str(path.resolve()),
                                                        "sha256": digest(path)}
                                                     for kind, path in (("stdout", stdout),
                                                                        ("stderr", stderr))}}
            if prep is not None:
                result["prep"] = prep
            if code == 0 and export.is_file():
                result["raw"]["export"] = {"path": str(export.resolve()),
                                             "sha256": digest(export)}
                try:
                    result["coverage"] = classify_export(json.loads(export.read_text()), roots[name])
                    critical_files, missing, critical = critical_census(
                        result["coverage"], name, feature)
                    result["critical_uncovered"] = critical
                    result["critical_branch_census"] = {
                        "files": critical_files,
                        "missing_files": missing,
                        "count": sum(item["count"] for item in critical_files.values()),
                        "covered": sum(item["covered"] for item in critical_files.values())}
                    result["status"] = "passed" if not missing and not critical and all(
                        item["covered"] == item["count"] for item in critical_files.values()
                    ) else "incomplete"
                    if missing:
                        result["reason"] = "critical_branches_not_instrumented"
                    elif critical or any(
                        item["covered"] < item["count"] for item in critical_files.values()
                    ):
                        result["reason"] = "critical_branch_target_open"
                except (ValueError, KeyError, TypeError, json.JSONDecodeError) as failure:
                    result["status"] = "failed"
                    result["reason"] = f"invalid_export:{failure}"
            else:
                result["status"] = "incomplete" if code == 124 else "failed"
                result["reason"] = "timeout" if code == 124 else "coverage_run_failed"
            results.append(result)
    if any(source(root, name) != revisions[name] or
           digest(root / "Cargo.lock") != cargo_locks[name]
           for name, root in roots.items()):
        raise ValueError("source graph changed during coverage run")
    reviews = (json.loads(args.reviews.read_text(), object_pairs_hook=unique_pairs)
               if args.reviews else [])
    gaps = [{"run": row["id"], **gap} for row in results
            for gap in row.get("critical_uncovered", [])]
    reviewed = validate_reviews(reviews, gaps, roots)
    for row in results:
        if "critical_branch_census" not in row:
            continue
        missing = row["critical_branch_census"]["missing_files"]
        unreviewed = any(gap_key({"run": row["id"], **gap}) not in reviewed
                         for gap in row["critical_uncovered"])
        complete = all(item["count"] == item["covered"] or any(
            gap["file"] == file for gap in row["critical_uncovered"]
        ) for file, item in row["critical_branch_census"]["files"].items())
        if not missing and not unreviewed and complete:
            row["status"] = "passed"
            row.pop("reason", None)
    report = {"schema": "tl-mltl.v8-coverage/v1", "source_revisions": revisions,
              "cargo_lock_sha256": cargo_locks,
              "tools": tools, "profile": "test", "test_selection": ["lib", "tests"],
              "host": {"system": platform.system(), "machine": platform.machine()},
              "reviewed_infeasibility": reviews,
              "runs": results, "status": "passed" if len(results) == 8 and all(
                  item["status"] == "passed" for item in results) else "incomplete"}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "runs": len(results),
                      "output": str(args.output.resolve())}, sort_keys=True))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
