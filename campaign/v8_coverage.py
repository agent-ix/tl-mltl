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
    "mltl": ("src/future/evaluate.rs", "src/past/evaluate.rs",
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
    "mltl": {"default": ("src/future/evaluate.rs", "src/past/evaluate.rs",
                         "src/mapping/past.rs", "src/wire/"),
             "infinite": CRITICAL_PREFIXES["mltl"]},
    "rewrite": {"default": ("src/engine/future.rs", "src/engine/past.rs",
                            "src/report.rs", "src/replay.rs", "src/disposition.rs"),
                "infinite": CRITICAL_PREFIXES["rewrite"]},
}
TOOLCHAIN = "nightly"


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
        if not all(isinstance(v, int) and v >= 0 for v in
                   (lines["count"], lines["covered"], branches["count"], branches["covered"])):
            raise ValueError(f"malformed coverage counts: {relative}")
        if lines["covered"] > lines["count"] or branches["covered"] > branches["count"]:
            raise ValueError(f"impossible coverage counts: {relative}")
        if branches["count"] and not item["branches"]:
            raise ValueError(f"missing detailed branch population: {relative}")
        uncovered = []
        for branch in item["branches"]:
            if len(branch) != 9 or any(type(value) is not int for value in branch):
                raise ValueError(f"malformed branch location: {relative}")
            # LLVM export: line/column span, true and false counts, file IDs, kind.
            if branch[4] == 0 or branch[5] == 0:
                location = {"line": branch[0], "column": branch[1],
                            "true_count": branch[4], "false_count": branch[5]}
                if location not in uncovered:
                    uncovered.append(location)
        if branches["covered"] < branches["count"] and not uncovered:
            raise ValueError(f"unlocated uncovered branch: {relative}")
        files[relative] = {"lines": {"count": lines["count"], "covered": lines["covered"]},
                           "branches": {"count": branches["count"],
                                        "covered": branches["covered"]},
                           "uncovered_branch_locations": uncovered}
    if not files or not any(item["branches"]["count"] for item in files.values()):
        raise ValueError("no production branches were measured")
    return {"files": files, "totals": {
        metric: {key: sum(file[metric][key] for file in files.values())
                 for key in ("count", "covered")}
        for metric in ("lines", "branches")}}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    for name in CRATES:
        parser.add_argument(f"--{name}", type=Path, required=True)
    parser.add_argument("--cargo-home", type=Path, required=True)
    parser.add_argument("--raw-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=1800)
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
            lane_env = env | {"CARGO_TARGET_DIR": str(args.raw_dir / f"{key}.target")}
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
            if code == 0 and export.is_file():
                result["raw"]["export"] = {"path": str(export.resolve()),
                                             "sha256": digest(export)}
                try:
                    result["coverage"] = classify_export(json.loads(export.read_text()), roots[name])
                    critical = []
                    critical_files = {}
                    for file, details in result["coverage"]["files"].items():
                        if any(file.startswith(prefix) for prefix in CRITICAL_PREFIXES[name]):
                            critical_files[file] = details["branches"]
                            critical.extend({"file": file, **location} for location in
                                            details["uncovered_branch_locations"])
                    missing = [prefix for prefix in EXPECTED_CRITICAL[name][feature]
                               if not any(file.startswith(prefix) for file in critical_files)]
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
                    elif critical:
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
    report = {"schema": "tl-mltl.v8-coverage/v1", "source_revisions": revisions,
              "cargo_lock_sha256": cargo_locks,
              "tools": tools, "profile": "test", "test_selection": ["lib", "tests"],
              "host": {"system": platform.system(), "machine": platform.machine()},
              "runs": results, "status": "passed" if len(results) == 8 and all(
                  item["status"] == "passed" for item in results) else "incomplete"}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")
    print(json.dumps({"status": report["status"], "runs": len(results),
                      "output": str(args.output.resolve())}, sort_keys=True))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
