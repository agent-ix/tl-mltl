"""Reconcile one live four-crate llvm-cov run against its raw exports."""

from __future__ import annotations

import hashlib
import json
import platform
import subprocess
from pathlib import Path

from v8_coverage import (CRATES, FEATURES, SCHEMA, classify_export, critical_census,
                         critical_summaries_accounted, critical_summary_missing, gap_key,
                         parse_prep_binary, parse_prep_command, summary_key,
                         tool_path, validate_reviews)


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unique_pairs(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate V8 report key: {key}")
        result[key] = value
    return result


def tool_versions() -> dict[str, str]:
    commands = {"rustc": (str(tool_path("rustc")), "--version"),
                "llvm_cov": (str(tool_path("llvm-cov")), "--version"),
                "llvm_profdata": (str(tool_path("llvm-profdata")), "--version"),
                "cargo_llvm_cov": ("cargo", "llvm-cov", "--version")}
    return {name: subprocess.run(argv, capture_output=True, text=True,
                                 timeout=10, check=True).stdout.strip()
            for name, argv in commands.items()}


def expected_runs(raw_dir: Path) -> list[tuple[str, str, str, list[str]]]:
    rows = []
    for name in CRATES:
        for feature, flags in FEATURES[name]:
            key = f"{name}-{feature}"
            argv = ["cargo", "llvm-cov", "--branch", "--json", "--output-path",
                    str((raw_dir / f"{key}.json").resolve()), "--lib", "--tests",
                    *flags, "--locked", "--offline"]
            rows.append((key, name, feature, argv))
    return rows


def verify(report_bytes: bytes, raw_dir: Path, graph: dict,
           tools: dict[str, str], expected_review_bytes: bytes | None = None,
           report_path: Path | None = None) -> tuple[str, dict, dict]:
    report = json.loads(report_bytes, object_pairs_hook=unique_pairs)
    expected_fields = {
        "schema", "source_revisions", "cargo_lock_sha256", "tools", "profile",
        "test_selection", "host", "runs", "status",
    }
    if (not isinstance(report, dict) or
            set(report) not in (expected_fields, expected_fields | {"reviewed_infeasibility"})):
        raise ValueError("V8 report shape mismatch")
    if expected_review_bytes is not None:
        expected_reviews = json.loads(expected_review_bytes, object_pairs_hook=unique_pairs)
        if report.get("reviewed_infeasibility") != expected_reviews:
            raise ValueError("V8 report reviews differ from declared input")
    revisions = {name: graph[f"tl-{name}"]["revision"] for name in CRATES}
    locks = {name: graph[f"tl-{name}"]["cargo_lock_sha256"] for name in CRATES}
    if (report["schema"] != SCHEMA or
            report["source_revisions"] != revisions or
            report["cargo_lock_sha256"] != locks or report["tools"] != tools or
            report["profile"] != "test" or report["test_selection"] != ["lib", "tests"] or
            report["host"] != {"system": platform.system(),
                               "machine": platform.machine()}):
        raise ValueError("V8 source, tool, profile, or host mismatch")
    if not isinstance(report["runs"], list) or len(report["runs"]) != 8:
        raise ValueError("V8 feature population incomplete")
    artifacts = {"report": {"path": str((report_path or
                                         raw_dir.parent / "coverage-native.json").resolve()),
                            "sha256": digest(report_bytes)}}
    critical_uncovered = []
    critical_summary_deficits = []
    covered = 0
    total = 0
    line_covered = 0
    line_total = 0
    failures = []
    successful = []
    for row, (key, name, feature, argv) in zip(report["runs"], expected_runs(raw_dir), strict=True):
        if (not isinstance(row, dict) or row.get("id") != key or
                row.get("repo") != name or row.get("feature") != feature or
                row.get("source_revision") != revisions[name] or row.get("argv") != argv or
                type(row.get("exit_code")) is not int):
            raise ValueError(f"V8 run identity mismatch: {key}")
        if name == "parse":
            prep = row.get("prep")
            if (not isinstance(prep, dict) or prep.get("argv") != parse_prep_command() or
                    type(prep.get("exit_code")) is not int or
                    not isinstance(prep.get("raw"), dict) or
                    set(prep["raw"]) != {"stdout", "stderr"}):
                raise ValueError(f"V8 parse preparation identity mismatch: {key}")
            for kind in ("stdout", "stderr"):
                path = raw_dir / f"{key}.prep.{kind}"
                record = {"path": str(path.resolve()), "sha256": digest(path.read_bytes())}
                if prep["raw"][kind] != record:
                    raise ValueError(f"V8 parse preparation log changed: {key}/{kind}")
                artifacts[f"{key}.prep.{kind}"] = record
            binary = parse_prep_binary(raw_dir / f"{key}.target")
            if prep["exit_code"] == 0 and "binary" in prep:
                record = {"path": str(binary.resolve()), "sha256": digest(binary.read_bytes())}
                if prep["binary"] != record:
                    raise ValueError(f"V8 parse preparation binary changed: {key}")
                artifacts[f"{key}.prep.binary"] = record
            else:
                if ("binary" in prep or row["exit_code"] != 125 or
                        row.get("status") != "failed" or
                        row.get("reason") != "parse_example_prep_failed" or
                        "export" in row.get("raw", {})):
                    raise ValueError(f"V8 failed parse preparation claimed coverage: {key}")
        elif "prep" in row:
            raise ValueError(f"V8 unexpected preparation: {key}")
        expected_raw = {kind: raw_dir / f"{key}.{suffix}" for kind, suffix in
                        (("stdout", "stdout"), ("stderr", "stderr"), ("export", "json"))}
        records = row.get("raw")
        if not isinstance(records, dict) or not {"stdout", "stderr"} <= set(records):
            raise ValueError(f"V8 raw streams missing: {key}")
        for kind, path in expected_raw.items():
            if kind == "export" and kind not in records:
                continue
            record = records[kind]
            raw = path.read_bytes()
            if (record != {"path": str(path.resolve()), "sha256": digest(raw)}):
                raise ValueError(f"V8 raw path or digest mismatch: {key}/{kind}")
            artifacts[f"{key}.{kind}"] = record
        if row["exit_code"] != 0:
            if row.get("status") not in ("failed", "incomplete") or "export" in records:
                raise ValueError(f"V8 failed process claimed coverage: {key}")
            failures.append({"id": key, "reason": row.get("reason", "process_failed")})
            continue
        if "export" not in records:
            raise ValueError(f"V8 successful process missing export: {key}")
        export = json.loads(expected_raw["export"].read_bytes(), object_pairs_hook=unique_pairs)
        measured = classify_export(export, Path(graph[f"tl-{name}"]["path"]))
        if row.get("coverage") != measured:
            raise ValueError(f"V8 production coverage tampered: {key}")
        files, missing, gaps = critical_census(measured, name, feature)
        census = {"files": files, "missing_files": missing,
                  "count": sum(item["count"] for item in files.values()),
                  "covered": sum(item["covered"] for item in files.values())}
        if row.get("critical_branch_census") != census or row.get("critical_uncovered") != gaps:
            raise ValueError(f"V8 critical branch census tampered: {key}")
        summaries = critical_summary_missing(measured, files, key,
                                              records["export"]["sha256"])
        successful.append((row, key, files, missing, gaps, summaries))
        total += measured["totals"]["branches"]["count"]
        covered += measured["totals"]["branches"]["covered"]
        line_total += measured["totals"]["lines"]["count"]
        line_covered += measured["totals"]["lines"]["covered"]
        critical_uncovered.extend({"run": key, **gap} for gap in gaps)
        critical_summary_deficits.extend(summaries)
    reviews = report.get("reviewed_infeasibility", [])
    reviewed_locations, reviewed_summaries = validate_reviews(
        reviews, critical_uncovered, critical_summary_deficits,
        {name: Path(graph[f"tl-{name}"]["path"]) for name in CRATES})
    for row, key, files, missing, gaps, summaries in successful:
        unreviewed = (any(gap_key({"run": key, **gap}) not in reviewed_locations
                          for gap in gaps) or
                      any(summary_key(summary) not in reviewed_summaries
                          for summary in summaries))
        expected_status = "passed" if (not missing and not unreviewed and
                                       critical_summaries_accounted(files, summaries)) else "incomplete"
        expected_reason = (None if expected_status == "passed" else
                           "critical_branches_not_instrumented" if missing else
                           "critical_branch_target_open")
        if row.get("status") != expected_status or row.get("reason") != expected_reason:
            raise ValueError(f"V8 critical target status mismatch: {key}")
    expected_status = "passed" if not failures and all(
        row["status"] == "passed" for row in report["runs"]) else "incomplete"
    if report["status"] != expected_status:
        raise ValueError("V8 aggregate status mismatch")
    population = {"declared": 8, "visited": len(report["runs"]),
                  "production_branches": {"count": total, "covered": covered},
                  "production_lines": {"count": line_total, "covered": line_covered},
                  "critical_uncovered": critical_uncovered,
                  "critical_summary_missing": critical_summary_deficits,
                  "reviewed_infeasibility": reviews,
                  "run_failures": failures, "source_revisions": revisions,
                  "cargo_lock_sha256": locks, "tools": tools}
    return expected_status, population, artifacts


def verify_retained(report_bytes: bytes, raw_dir: Path, graph: dict,
                    tools: dict[str, str], review_bytes: bytes, report_path: Path,
                    review_path: Path | None) -> tuple[str, dict, dict]:
    """Apply exact human reviews to one previously measured, immutable V8 run."""
    if report_path.read_bytes() != report_bytes:
        raise ValueError("V8 retained report bytes changed")
    if review_path is not None and review_path.read_bytes() != review_bytes:
        raise ValueError("V8 retained review bytes changed")
    base_status, population, artifacts = verify(
        report_bytes, raw_dir, graph, tools, report_path=report_path)
    report = json.loads(report_bytes, object_pairs_hook=unique_pairs)
    if report.get("reviewed_infeasibility", []) != []:
        raise ValueError("V8 retained report must be unreviewed")
    reviews = json.loads(review_bytes, object_pairs_hook=unique_pairs)
    if review_path is None and reviews != []:
        raise ValueError("V8 retained reviews require a pinned input path")
    reviewed_locations, reviewed_summaries = validate_reviews(
        reviews, population["critical_uncovered"], population["critical_summary_missing"],
        {name: Path(graph[f"tl-{name}"]["path"]) for name in CRATES})
    unreviewed = (any(gap_key(gap) not in reviewed_locations
                      for gap in population["critical_uncovered"]) or
                  any(summary_key(summary) not in reviewed_summaries
                      for summary in population["critical_summary_missing"]))
    missing_files = any(row.get("critical_branch_census", {}).get("missing_files")
                        for row in report["runs"])
    status = ("passed" if not population["run_failures"] and not missing_files and
              not unreviewed else "incomplete")
    population = population | {"reviewed_infeasibility": reviews,
                               "retained_report_status": base_status}
    if review_path is not None:
        artifacts = artifacts | {"review_input": {"path": str(review_path.resolve()),
                                                  "sha256": digest(review_bytes)}}
    return status, population, artifacts
