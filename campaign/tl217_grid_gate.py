#!/usr/bin/env python3
"""Run and reconcile the exact TL-217 live C2PO/R2U2 past grid.

The Rust producer owns formula construction and three-way verdict comparison.
This gate independently checks the raw target rows, the declared population,
artifact digests, and every reported classification. It grants credit only to
a clean, exact source run with no admitted mismatch or missing target cell.
"""

import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import subprocess
import sys


TARGET_REVISION = "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a"
COMPILER_SHA256 = "f978a32f667a8247c387a66bce35371c97b7d8f7b730035a8ee40cdfc428ce12"
MONITOR_SHA256 = "5743987dddb47cc01829a633e15623095c9c2aff2f8bb24e30d7f0e0f488f85f"
INTERVALS = {
    "zero-singleton": [0, 0],
    "zero-unit": [0, 1],
    "zero-upper": [0, 2],
    "nonzero-singleton-one": [1, 1],
    "nonzero-range": [1, 2],
    "nonzero-singleton": [2, 2],
}
OPERATORS = ("once", "historically", "since", "triggered")
TRACES = ("all-true", "all-false", "boundary-toggle")
ARTIFACT_KINDS = ("c2po", "csv", "bin", "compiler.stdout", "compiler.stderr",
                  "monitor.stdout", "monitor.stderr")
MARKER = b"TL217_PAST_GRID "


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git_output(repo: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=repo).decode().strip()


def target_rows(data: bytes, formula_count: int) -> dict[tuple[int, int], bool]:
    result = {}
    for line in data.decode("utf-8").splitlines():
        match = re.fullmatch(r"(0|[1-9][0-9]*):(0|[1-9][0-9]*),([TF])", line)
        if not match:
            raise ValueError("malformed target verdict row")
        formula, position = int(match[1]), int(match[2])
        if formula >= formula_count or (formula, position) in result:
            raise ValueError("duplicate or unknown target verdict")
        result[(formula, position)] = match[3] == "T"
    if not result:
        raise ValueError("empty target verdict output")
    return result


def expected_cases() -> dict[str, tuple[str, list[int] | None, int, int, str]]:
    cases = {}
    for group, interval in INTERVALS.items():
        for formula_id, (operator, depth) in enumerate(
            (operator, depth) for operator in OPERATORS for depth in range(1, 4)
        ):
            case = f"{operator}-{interval[0]}-{interval[1]}-d{depth}"
            cases[case] = (operator, interval, depth, formula_id, group)
    for depth in range(1, 4):
        cases[f"previous-none-d{depth}"] = ("previous", None, depth, depth - 1, "previous")
    return cases


def expected_expression(operator: str, interval: list[int] | None, depth: int) -> str:
    expression = "p"
    for _ in range(depth):
        if operator == "previous":
            expression = f"O[1,1]({expression})"
        else:
            assert interval is not None
            a, b = interval
            if operator == "once":
                expression = f"O[{a},{b}]({expression})"
            elif operator == "historically":
                expression = f"H[{a},{b}]({expression})"
            elif operator == "since":
                expression = f"({expression} S[{a},{b}] q)"
            elif operator == "triggered":
                expression = f"(!((!{expression}) S[{a},{b}] (!q)))"
            else:
                raise ValueError("unknown past operator")
    return expression


def expected_admission(operator: str, interval: list[int] | None, depth: int) -> bool:
    if operator == "previous":
        return depth <= 2
    return (operator == "once" and interval in ([0, 0], [0, 1]) or
            operator == "historically" and interval == [0, 0] or
            operator == "since" and interval in ([0, 0], [0, 1]) or
            operator == "triggered" and interval == [0, 0])


def expected_run_inputs(group: str, trace: str) -> tuple[bytes, bytes]:
    cases = expected_cases()
    selected = sorted((formula_id, operator, interval, depth)
                      for operator, interval, depth, formula_id, case_group in cases.values()
                      if case_group == group)
    spec = ("INPUT\n p,q: bool;\nPTSPEC\n" + "\n".join(
        f" {expected_expression(operator, interval, depth)};"
        for _, operator, interval, depth in selected
    ) + "\n").encode()
    boundary = (1 if group == "previous" else INTERVALS[group][1])
    observations = []
    for position in range(6):
        if trace == "all-true":
            p, q = True, True
        elif trace == "all-false":
            p, q = False, False
        elif trace == "boundary-toggle":
            p = position >= boundary
            q = not p
        else:
            raise ValueError("unknown trace class")
        observations.append(f"{int(p)},{int(q)}\n")
    return spec, ("# p,q\n" + "".join(observations)).encode()


def verify_target_source(source: Path) -> None:
    source = source.resolve()
    if git_output(source, "rev-parse", "HEAD") != TARGET_REVISION:
        raise ValueError("wrong R2U2 source revision")
    if git_output(source, "status", "--porcelain"):
        raise ValueError("dirty R2U2 source")
    if sha256((source / "compiler/c2po.py").read_bytes()) != COMPILER_SHA256:
        raise ValueError("wrong C2PO compiler bytes")
    if sha256((source / "monitors/c/build/r2u2").read_bytes()) != MONITOR_SHA256:
        raise ValueError("wrong R2U2 monitor bytes")


def verify(report_bytes: bytes, raw_dir: Path, source_revision: str,
           cargo_lock_sha256: str) -> dict:
    lines = report_bytes.splitlines()
    if len(lines) != 1 or not lines[0].startswith(MARKER):
        raise ValueError("missing or duplicate TL-217 report marker")
    report = json.loads(lines[0][len(MARKER):])
    expected_header = {
        "schema": "tl-mltl.r2u2-past-grid/v1",
        "source_revision": source_revision,
        "source_state": "clean",
        "cargo_lock_sha256": cargo_lock_sha256,
        "target_revision": TARGET_REVISION,
        "compiler_sha256": COMPILER_SHA256,
        "monitor_sha256": MONITOR_SHA256,
        "license": "Apache-2.0",
        "steps": 6,
        "formula_trace_cases": 225,
        "per_step_cells": 1350,
        "unexplained_admitted_cells": 0,
    }
    for key, expected in expected_header.items():
        if report.get(key) != expected:
            raise ValueError(f"wrong {key}")
    if report.get("intervals") != [[name, *interval] for name, interval in INTERVALS.items()]:
        raise ValueError("wrong interval axis")
    cases = expected_cases()
    run_pairs = {f"{group}-{trace}": (group, trace)
                 for group in [*INTERVALS, "previous"] for trace in TRACES}
    run_ids = set(run_pairs)
    if set(report.get("runs", {})) != run_ids:
        raise ValueError("missing or extra live target run")
    artifact_names = {f"{run}.{kind}" for run in run_ids for kind in ARTIFACT_KINDS}
    if set(report.get("artifacts", {})) != artifact_names:
        raise ValueError("missing or extra raw artifact")
    raw = {}
    for name in artifact_names:
        data = (raw_dir / name).read_bytes()
        if sha256(data) != report["artifacts"][name]:
            raise ValueError(f"raw artifact digest mismatch: {name}")
        raw[name] = data
    target = {}
    for run in run_ids:
        group, trace = run_pairs[run]
        expected_spec, expected_trace = expected_run_inputs(group, trace)
        if raw[f"{run}.c2po"] != expected_spec or raw[f"{run}.csv"] != expected_trace:
            raise ValueError(f"wrong C2PO spec or trace bytes for {run}")
        count = 3 if group == "previous" else 12
        parsed = target_rows(raw[f"{run}.monitor.stdout"], count)
        run_report = report["runs"][run]
        if run_report != {
            "target": {"compiler_exit": 0, "monitor_exit": 0,
                       "formula_count": count, "trace_positions": 6},
            "extra_target_positions": sum(position >= 6 for _, position in parsed),
        }:
            raise ValueError(f"wrong command/target population for {run}")
        target[run] = parsed
    seen = set()
    counts = {}
    for row in report.get("rows", []):
        case = row["case"]
        if case not in cases:
            raise ValueError("unknown formula shape")
        operator, interval, depth, formula_id, group = cases[case]
        trace, position = row["trace"], row["position"]
        if (row["operator"], row["interval"], row["depth"]) != (operator, interval, depth):
            raise ValueError("mislabelled formula shape")
        if trace not in TRACES or type(position) is not int or position not in range(6):
            raise ValueError("invalid trace or position")
        key = (case, trace, position)
        if key in seen:
            raise ValueError("duplicate grid cell")
        seen.add(key)
        origin_hazard = ((operator in ("historically", "triggered") and
                          interval is not None and position < interval[1] * depth) or
                         (operator == "previous" and position < depth))
        if row.get("origin_hazard") is not origin_hazard:
            raise ValueError("mislabelled origin hazard")
        observed = target[f"{group}-{trace}"].get((formula_id, position))
        if row["target"] is not observed:
            raise ValueError("reported verdict differs from raw target")
        if type(row["tl"]) is not bool or row["oracle"] is not row["tl"]:
            raise ValueError("source/oracle disagreement")
        admitted = row["mapping"]["status"] == "admitted"
        if not admitted and row["mapping"]["status"] != "refused":
            raise ValueError("invalid mapping status")
        if admitted != expected_admission(operator, interval, depth):
            raise ValueError("mapping admission differs from reviewed partition")
        if admitted:
            expression = expected_expression(operator, interval, depth)
            if row["mapping"] != {"status": "admitted",
                                   "expression_sha256": sha256(expression.encode())}:
                raise ValueError("wrong admitted expression identity")
        elif not isinstance(row["mapping"].get("reason"), str) or not row["mapping"]["reason"]:
            raise ValueError("missing typed mapping refusal")
        classification = ("unsupported_mapping" if not admitted else
                          "nonconclusive_target_missing" if observed is None else
                          "agreement" if observed is row["tl"] else "semantic_mismatch")
        if row["classification"] != classification:
            raise ValueError("false grid classification")
        counts[classification] = counts.get(classification, 0) + 1
    expected_cells = {(case, trace, position) for case in cases
                      for trace in TRACES for position in range(6)}
    if seen != expected_cells or report.get("classifications") != counts:
        raise ValueError("grid population/count mismatch")
    if counts != {"agreement": 360, "unsupported_mapping": 990}:
        raise ValueError("reviewed admission/target census changed")
    if counts.get("semantic_mismatch", 0) or counts.get("nonconclusive_target_missing", 0):
        raise ValueError("unexplained admitted target cell")
    return {"status": "passed", "visited": len(seen), "classifications": counts,
            "source_revision": source_revision, "target_revision": TARGET_REVISION}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo", type=Path, required=True)
    parser.add_argument("--target-source", type=Path, required=True)
    parser.add_argument("--raw-dir", type=Path, required=True)
    parser.add_argument("--report", type=Path, required=True)
    args = parser.parse_args()
    repo = args.repo.resolve()
    if git_output(repo, "status", "--porcelain"):
        raise ValueError("source checkout must be clean")
    verify_target_source(args.target_source)
    revision = git_output(repo, "rev-parse", "HEAD")
    lock_hash = sha256((repo / "Cargo.lock").read_bytes())
    if args.report.exists() or (args.raw_dir.exists() and any(args.raw_dir.iterdir())):
        raise ValueError("report/raw paths must be new or empty")
    args.raw_dir.mkdir(parents=True, exist_ok=True)
    environment = os.environ.copy()
    environment["TL_MLTL_C2PO_SOURCE"] = str(args.target_source.resolve())
    environment["TL_MLTL_LIVE_RAW_DIR"] = str(args.raw_dir.resolve())
    process = subprocess.run(
        ["cargo", "run", "--locked", "--offline", "--example", "tl217_live_past_grid"],
        cwd=repo, env=environment, capture_output=True, check=False,
    )
    args.report.write_bytes(process.stdout)
    if process.returncode:
        sys.stderr.buffer.write(process.stderr)
        raise ValueError(f"live grid exited {process.returncode}")
    result = verify(process.stdout, args.raw_dir, revision, lock_hash)
    print(json.dumps(result, sort_keys=True))
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
        print(f"TL-217 grid gate refused: {error}", file=sys.stderr)
        raise SystemExit(1)
