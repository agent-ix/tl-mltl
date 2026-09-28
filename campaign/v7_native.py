#!/usr/bin/env python3
"""Run the bounded V7 embedded/Miri probes and retain their native output.

This is a producer, not the V1 campaign verdict. Boundary counts below name
reviewed assertions in the pinned test sources; the process results show which
of those test bodies actually completed under Miri.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tomllib
from dataclasses import dataclass
from pathlib import Path

TARGET = "thumbv7em-none-eabi"
MIRI_FLAGS = "-Zmiri-strict-provenance -Zmiri-symbolic-alignment-check"
RESULT = re.compile(
    r"test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out"
)


@dataclass(frozen=True)
class Probe:
    name: str
    repo: str
    kind: str
    args: tuple[str, ...]
    test_name: str | None = None
    paired_edges: int = 0
    refusal_only_edges: int = 0


PROBES = (
    *(Probe(f"embedded_{feature}", "syntax", "build", (
        "rustup", "run", "1.98.1", "cargo", "build", "--locked", "--offline",
        "--target", TARGET, "--no-default-features",
        *(("--features", feature) if feature != "core" else ()), "--lib",
    )) for feature in ("core", "alloc", "serde")),
    Probe("syntax_formula_limits", "syntax", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--features", "serde", "--test", "infinite_formula",
        "strict_unbounded_reader_honors_lowered_node_and_depth_limits",
        "--", "--exact",
    ), "strict_unbounded_reader_honors_lowered_node_and_depth_limits", 3),
    Probe("syntax_borrowed_ownership", "syntax", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--features", "serde", "--test", "typed_signal_context",
        "borrowed_and_owned_catalogs_round_trip_with_distinct_identities",
        "--", "--exact",
    ), "borrowed_and_owned_catalogs_round_trip_with_distinct_identities"),
    Probe("parse_limits_utf8", "parse", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--test", "versioned_dialects",
        "parse_and_format_limits_are_exact_and_hostile_utf8_never_unwinds",
        "--", "--exact",
    ), "parse_and_format_limits_are_exact_and_hostile_utf8_never_unwinds", 8),
    Probe("mltl_lasso_limits", "mltl", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--features", "infinite-trace", "--test", "infinite_trace",
        "every_lasso_resource_dimension_refuses_one_over_without_panic",
        "--", "--exact",
    ), "every_lasso_resource_dimension_refuses_one_over_without_panic", 10),
    Probe("mltl_prefix_limits", "mltl", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--features", "infinite-trace", "--test", "infinite_trace",
        "prefix_resource_dimensions_refuse_one_over_without_panic",
        "--", "--exact",
    ), "prefix_resource_dimensions_refuse_one_over_without_panic", 6),
    Probe("rewrite_budgets", "rewrite", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--test", "rewrite",
        "iteration_application_and_work_budgets_fail_closed", "--", "--exact",
    ), "iteration_application_and_work_budgets_fail_closed", 0, 3),
    Probe("rewrite_record_limits", "rewrite", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--test", "tc_053_profile_subsystems",
        "tc_053_report_replay_and_all_work_limits_are_exact_and_fail_closed",
        "--", "--exact",
    ), "tc_053_report_replay_and_all_work_limits_are_exact_and_fail_closed", 3),
    Probe("oracle_limits", "oracle", "miri", (
        "rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
        "--offline", "--test", "reference", "tc_188_oracle_limit_edges_are_typed",
        "--", "--exact",
    ), "tc_188_oracle_limit_edges_are_typed", 4),
)


def classify(probe: Probe, code: int, output: str) -> str:
    if code != 0:
        return "failed"
    if probe.kind == "miri":
        if (f"test {probe.test_name} ... ok" not in output or
                len(RESULT.findall(output)) != 1 or "running 1 test" not in output):
            return "unavailable"
    elif not re.search(r"Finished `(?:dev|test)` profile", output):
        return "unavailable"
    return "passed"


def checked(command: tuple[str, ...], cwd: Path, env: dict[str, str], timeout: int) -> str:
    result = subprocess.run(command, cwd=cwd, env=env, capture_output=True,
                            text=True, timeout=timeout, check=False)
    if result.returncode:
        raise RuntimeError(f"{' '.join(command)} failed: {result.stderr.strip()}")
    return result.stdout.strip()


def source(path: Path, expected_name: str) -> str:
    manifest = tomllib.loads((path / "Cargo.toml").read_text())
    if manifest["package"]["name"] != f"tl-{expected_name}":
        raise ValueError(f"wrong Cargo package at {path}")
    revision = checked(("git", "rev-parse", "HEAD"), path, os.environ.copy(), 10)
    if checked(("git", "status", "--porcelain"), path, os.environ.copy(), 10):
        raise ValueError(f"dirty source checkout: {path}")
    return revision


def main() -> int:
    parser = argparse.ArgumentParser()
    for name in ("syntax", "parse", "mltl", "rewrite", "oracle"):
        parser.add_argument(f"--{name}", type=Path, required=True)
    parser.add_argument("--cargo-home", type=Path, required=True)
    parser.add_argument("--raw-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--timeout", type=int, default=300)
    args = parser.parse_args()
    paths = {name: getattr(args, name).resolve() for name in
             ("syntax", "parse", "mltl", "rewrite", "oracle")}
    sources = {name: source(path, name) for name, path in paths.items()}
    if len(set(paths.values())) != len(paths):
        raise ValueError("each crate needs a distinct checkout")
    syntax_root = (paths["syntax"] / "src/lib.rs").read_text()
    if not syntax_root.startswith("#![no_std]"):
        raise ValueError("tl-syntax lost its no_std crate root")
    for name in ("parse", "mltl", "rewrite", "oracle"):
        if (paths[name] / "src/lib.rs").read_text().startswith("#![no_std]"):
            raise ValueError(f"{name} has an unclassified no_std claim")
    if not args.cargo_home.is_dir():
        raise ValueError("a writable, locally provisioned CARGO_HOME is required")
    env = os.environ.copy()
    env.update({"CARGO_HOME": str(args.cargo_home.resolve()), "TMPDIR": "/private/tmp",
                "CARGO_NET_GIT_FETCH_WITH_CLI": "true", "MIRIFLAGS": MIRI_FLAGS,
                "GIT_CONFIG_COUNT": "5"})
    for index, (name, path) in enumerate(paths.items()):
        env[f"GIT_CONFIG_KEY_{index}"] = f"url.file://{path}.insteadOf"
        env[f"GIT_CONFIG_VALUE_{index}"] = f"https://github.com/agent-ix/tl-{name}.git"
    tools = {
        "rustc": checked(("rustup", "run", "1.98.1", "rustc", "--version"),
                         paths["syntax"], env, 10),
        "nightly_rustc": checked(("rustup", "run", "nightly", "rustc", "--version"),
                                 paths["syntax"], env, 10),
        "miri": checked(("rustup", "run", "nightly", "cargo", "miri", "--version"),
                        paths["syntax"], env, 10),
    }
    build_rustc = checked(("rustup", "which", "rustc", "--toolchain", "1.98.1"),
                          paths["syntax"], env, 10)
    tools["build_rustc"] = build_rustc
    targets = checked(("rustup", "target", "list", "--installed"),
                      paths["syntax"], env, 10).splitlines()
    if TARGET not in targets:
        raise ValueError(f"{TARGET} target is not installed")
    args.raw_dir.mkdir(parents=True, exist_ok=True)
    outcomes = []
    for probe in PROBES:
        probe_env = env.copy()
        if probe.kind == "build":
            probe_env["RUSTC"] = build_rustc
            probe_env["PATH"] = f"{Path(build_rustc).parent}:{env['PATH']}"
        try:
            process = subprocess.run(probe.args, cwd=paths[probe.repo], env=probe_env,
                                     capture_output=True, timeout=args.timeout, check=False)
            stdout, stderr, code = process.stdout, process.stderr, process.returncode
            status = classify(probe, code, (stdout + stderr).decode(errors="replace"))
        except subprocess.TimeoutExpired as error:
            stdout, stderr, code, status = error.stdout or b"", error.stderr or b"", None, "timeout"
        raw = stdout + stderr
        raw_path = args.raw_dir / f"{probe.name}.log"
        raw_path.write_bytes(raw)
        outcomes.append({"name": probe.name, "repo": probe.repo, "kind": probe.kind,
                         "argv": probe.args, "exit_code": code, "status": status,
                         "raw_path": os.path.relpath(raw_path.resolve(),
                                                     args.output.parent.resolve()),
                         "raw_sha256": hashlib.sha256(raw).hexdigest(),
                         "paired_edges": probe.paired_edges,
                         "refusal_only_edges": probe.refusal_only_edges,
                         "feature": probe.name.removeprefix("embedded_") if probe.kind == "build"
                         else "serde" if probe.repo == "syntax"
                         else "infinite-trace" if probe.repo == "mltl" else "default"})
    passed = [item for item in outcomes if item["status"] == "passed"]
    report = {
        "schema": "tl-mltl.v7-native/v1", "target": TARGET, "source_revisions": sources,
        "tool_versions": tools, "miri_flags": MIRI_FLAGS,
        "embedded_scope": {"syntax_features": ["core", "alloc", "serde"],
                           "std_only_crates": ["parse", "mltl", "rewrite", "oracle"]},
        "counts": {"probes": len(outcomes), "passed": len(passed),
                   "miri_passed": sum(item["kind"] == "miri" for item in passed),
                   "paired_edges": sum(item["paired_edges"] for item in passed),
                   "refusal_only_edges": sum(item["refusal_only_edges"] for item in passed)},
        "not_run_under_miri": [
            {"path": f"tl-{name}/tests/shared_assurance.rs",
             "reason": "subprocess and external assurance orchestration outside in-process Miri probe"}
            for name in paths
        ] + [{"path": "thumbv7em-none-eabi execution",
              "reason": "cross target is build checked, not executed by host Miri"}],
        "probes": outcomes,
        "status": "passed" if len(passed) == len(outcomes) else "incomplete",
    }
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    print(json.dumps({"status": report["status"], "counts": report["counts"],
                      "output": str(args.output.resolve())}, sort_keys=True))
    return 0 if report["status"] == "passed" else 1


if __name__ == "__main__":
    sys.exit(main())
