"""Independent verifier for one fresh V7 native producer run.

The expected probe inventory is duplicated here deliberately: a producer
change cannot silently change the campaign's accepted population.
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path
from typing import Any

TARGET = "thumbv7em-none-eabi"
FLAGS = "-Zmiri-strict-provenance -Zmiri-symbolic-alignment-check"
TEST_RESULT = re.compile(
    r"^test result: ok\. 1 passed; 0 failed; 0 ignored; 0 measured; \d+ filtered out(?:; finished in [^\n]+)?$",
    re.M,
)
BUILD_RESULT = re.compile(r"^\s*Finished `(?:dev|test)` profile", re.M)


def build(feature: str) -> list[str]:
    return ["rustup", "run", "1.98.1", "cargo", "build", "--locked", "--offline",
            "--target", TARGET, "--no-default-features",
            *(["--features", feature] if feature != "core" else []), "--lib"]


def miri(crate: str, test_file: str, name: str, feature: str) -> list[str]:
    return ["rustup", "run", "nightly", "cargo", "miri", "test", "--locked",
            "--offline", *(["--features", feature] if feature != "default" else []),
            "--test", test_file, name, "--", "--exact"]


# (name, repo, kind, feature, argv, test name, paired edges, refusal-only edges)
EXPECTED = (
    ("embedded_core", "syntax", "build", "core", build("core"), None, 0, 0),
    ("embedded_alloc", "syntax", "build", "alloc", build("alloc"), None, 0, 0),
    ("embedded_serde", "syntax", "build", "serde", build("serde"), None, 0, 0),
    ("syntax_formula_limits", "syntax", "miri", "serde", miri(
        "syntax", "infinite_formula",
        "strict_unbounded_reader_honors_lowered_node_and_depth_limits", "serde"),
     "strict_unbounded_reader_honors_lowered_node_and_depth_limits", 3, 0),
    ("syntax_borrowed_ownership", "syntax", "miri", "serde", miri(
        "syntax", "typed_signal_context",
        "borrowed_and_owned_catalogs_round_trip_with_distinct_identities", "serde"),
     "borrowed_and_owned_catalogs_round_trip_with_distinct_identities", 0, 0),
    ("parse_limits_utf8", "parse", "miri", "default", miri(
        "parse", "versioned_dialects",
        "parse_and_format_limits_are_exact_and_hostile_utf8_never_unwinds", "default"),
     "parse_and_format_limits_are_exact_and_hostile_utf8_never_unwinds", 8, 0),
    ("mltl_lasso_limits", "mltl", "miri", "infinite-trace", miri(
        "mltl", "infinite_trace",
        "every_lasso_resource_dimension_refuses_one_over_without_panic", "infinite-trace"),
     "every_lasso_resource_dimension_refuses_one_over_without_panic", 10, 0),
    ("mltl_prefix_limits", "mltl", "miri", "infinite-trace", miri(
        "mltl", "infinite_trace",
        "prefix_resource_dimensions_refuse_one_over_without_panic", "infinite-trace"),
     "prefix_resource_dimensions_refuse_one_over_without_panic", 6, 0),
    ("rewrite_budgets", "rewrite", "miri", "default", miri(
        "rewrite", "rewrite", "iteration_application_and_work_budgets_fail_closed", "default"),
     "iteration_application_and_work_budgets_fail_closed", 0, 3),
    ("rewrite_record_limits", "rewrite", "miri", "default", miri(
        "rewrite", "tc_053_profile_subsystems",
        "tc_053_report_replay_and_all_work_limits_are_exact_and_fail_closed", "default"),
     "tc_053_report_replay_and_all_work_limits_are_exact_and_fail_closed", 3, 0),
    ("oracle_limits", "oracle", "miri", "default", miri(
        "oracle", "reference", "tc_188_oracle_limit_edges_are_typed", "default"),
     "tc_188_oracle_limit_edges_are_typed", 4, 0),
)
assert len(EXPECTED) == 11 and sum(row[6] for row in EXPECTED) == 34


def unique_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    value: dict[str, Any] = {}
    for key, item in pairs:
        if key in value:
            raise ValueError(f"duplicate V7 JSON key: {key}")
        value[key] = item
    return value


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def current_tools(cwd: Path) -> dict[str, str]:
    commands = {
        "rustc": ["rustup", "run", "1.98.1", "rustc", "--version"],
        "nightly_rustc": ["rustup", "run", "nightly", "rustc", "--version"],
        "miri": ["rustup", "run", "nightly", "cargo", "miri", "--version"],
        "build_rustc": ["rustup", "which", "rustc", "--toolchain", "1.98.1"],
    }
    observed = {}
    for name, argv in commands.items():
        result = subprocess.run(argv, cwd=cwd, capture_output=True, text=True,
                                timeout=10, check=False)
        if result.returncode:
            raise ValueError(f"V7 tool unavailable: {name}")
        observed[name] = result.stdout.strip()
    return observed


def verify(report_bytes: bytes, summary_bytes: bytes, root: Path,
           graph: dict[str, dict[str, str]], tools: dict[str, str]) -> tuple[dict, dict]:
    """Recompute status from the live report and every retained native log."""
    report = json.loads(report_bytes, object_pairs_hook=unique_pairs)
    summary = json.loads(summary_bytes, object_pairs_hook=unique_pairs)
    if not isinstance(report, dict) or set(report) != {
        "schema", "target", "source_revisions", "tool_versions", "miri_flags",
        "embedded_scope", "counts", "not_run_under_miri", "probes", "status",
    }:
        raise ValueError("V7 native report has missing or unknown fields")
    revisions = {name.removeprefix("tl-"): entry["revision"] for name, entry in graph.items()}
    if (report.get("schema") != "tl-mltl.v7-native/v1" or
            report.get("source_revisions") != revisions or
            report.get("target") != TARGET or report.get("miri_flags") != FLAGS or
            report.get("tool_versions") != tools or
            report.get("embedded_scope") != {
                "syntax_features": ["core", "alloc", "serde"],
                "std_only_crates": ["parse", "mltl", "rewrite", "oracle"],
            }):
        raise ValueError("V7 source, target, feature, or tool identity mismatch")
    not_run = [{"path": f"tl-{name}/tests/shared_assurance.rs",
                "reason": "subprocess and external assurance orchestration outside in-process Miri probe"}
               for name in ("syntax", "parse", "mltl", "rewrite", "oracle")]
    not_run.append({"path": "thumbv7em-none-eabi execution",
                    "reason": "cross target is build checked, not executed by host Miri"})
    if report.get("not_run_under_miri") != not_run:
        raise ValueError("V7 unsupported-path census mismatch")
    rows = report.get("probes")
    if not isinstance(rows, list) or len(rows) != len(EXPECTED):
        raise ValueError("V7 probe census missing or duplicated")
    artifacts = {"report": {"path": str((root / "embedded_miri_limits_native.json").resolve()),
                            "sha256": digest(report_bytes)}}
    for row, expected in zip(rows, EXPECTED, strict=True):
        name, repo, kind, feature, argv, test_name, paired, refusal = expected
        identity = {"name": name, "repo": repo, "kind": kind, "feature": feature,
                    "argv": argv, "exit_code": 0, "status": "passed",
                    "paired_edges": paired, "refusal_only_edges": refusal,
                    "raw_path": f"embedded_miri_limits_native/{name}.log"}
        if (not isinstance(row, dict) or set(row) != set(identity) | {"raw_sha256"} or any(
            type(row.get(key)) is not type(value) or row.get(key) != value
            for key, value in identity.items()
        )):
            raise ValueError(f"V7 probe identity or outcome mismatch: {name}")
        raw_path = root / identity["raw_path"]
        raw = raw_path.read_bytes()
        if row.get("raw_sha256") != digest(raw):
            raise ValueError(f"V7 raw digest mismatch: {name}")
        output = raw.decode(errors="replace")
        if kind == "build":
            if len(BUILD_RESULT.findall(output)) != 1:
                raise ValueError(f"V7 embedded build completion missing: {name}")
        elif (len(re.findall(r"^running 1 test$", output, re.M)) != 1 or
              len(re.findall(rf"^test {re.escape(test_name)} \.\.\. ok$", output, re.M)) != 1 or
              len(TEST_RESULT.findall(output)) != 1 or
              len(re.findall(r"^test result:", output, re.M)) != 1):
            raise ValueError(f"V7 Miri test did not run exactly once: {name}")
        artifacts[name] = {"path": str(raw_path.resolve()), "sha256": digest(raw)}
    counts = {"probes": 11, "passed": 11, "miri_passed": 8,
              "paired_edges": 34, "refusal_only_edges": 3}
    if (report.get("counts") != counts or report.get("status") != "passed" or
            summary != {"status": "passed", "counts": counts,
                        "output": str((root / "embedded_miri_limits_native.json").resolve())}):
        raise ValueError("V7 summary or boundary population mismatch")
    return {"declared": 11, "visited": 11, "embedded_builds": 3, "miri_tests": 8,
            "paired_edges": 34, "refusal_only_edges": 3,
            "not_run_under_miri": len(not_run), "target": TARGET,
            "source_revisions": revisions, "tool_versions": tools}, artifacts
