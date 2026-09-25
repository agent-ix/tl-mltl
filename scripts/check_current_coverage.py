#!/usr/bin/env python3
"""Gate current Quire coverage while retaining superseded campaign documents.

Quire 0.33 scans every document of a declared archetype, including artifacts
whose frontmatter says ``status: superseded``. Keep its full report intact and
exclude only the explicitly retired predecessors from this *current* gate.
The successor V1 matrix and every other current document remain answerable.
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path


# The V1 disposition in MRS-004 names these predecessors. Adding another path
# requires a reviewed change here; merely changing frontmatter cannot make a
# current requirement disappear from the gate.
SUPERSEDED = frozenset({
    "spec/corpus-campaign-test-matrix.md",
    "spec/requirements/FR-008-corpus-coverage-model.md",
    "spec/requirements/FR-009-versioned-fixture-families.md",
    "spec/requirements/FR-010-explicit-interoperability-dispositions.md",
    "spec/requirements/FR-019-consume-qobs-c00.md",
    "spec/requirements/FR-020-property-obligation-ledger.md",
    "spec/requirements/FR-021-budgeted-fuzz-campaigns.md",
    "spec/requirements/FR-022-measured-mutation-campaign.md",
    "spec/requirements/FR-023-bounded-kani-claims.md",
    "spec/requirements/FR-024-shared-effectiveness-intake.md",
    "spec/requirements/NFR-004-reproducible-corpus-retention.md",
    "spec/requirements/NFR-005-truthful-effectiveness-evidence.md",
    "spec/verification-effectiveness-test-matrix.md",
})
UNREAD_REASONS = frozenset({"status-column-matches-nothing", "hollow-denominator"})


def verified_predecessors(scope: Path) -> None:
    for relative in sorted(SUPERSEDED):
        path = scope / relative
        lines = path.read_text(encoding="utf-8").splitlines()
        if not lines or lines[0] != "---":
            raise ValueError(f"superseded predecessor lacks frontmatter: {relative}")
        try:
            end = lines.index("---", 1)
        except ValueError as error:
            raise ValueError(f"superseded predecessor has unclosed frontmatter: {relative}") from error
        statuses = [line.removeprefix("status:").strip() for line in lines[1:end]
                    if line.startswith("status:")]
        if statuses != ["superseded"]:
            raise ValueError(f"superseded predecessor status changed: {relative}: {statuses}")


def summarize(report: dict, scope: Path) -> tuple[dict, list[dict], list[dict]]:
    verified_predecessors(scope)
    required = ("unbacked_rows", "status_lies", "groups", "totals", "diagnostics")
    if not isinstance(report, dict) or any(key not in report for key in required):
        raise ValueError("Quire coverage report is missing required fields")
    if not all(isinstance(report[key], list) for key in required if key != "totals"):
        raise ValueError("Quire coverage report has a non-list finding field")
    if not isinstance(report["totals"], dict):
        raise ValueError("Quire coverage report has no totals object")

    groups = report["groups"]
    if not groups or any(not isinstance(group, dict) for group in groups):
        raise ValueError("Quire matched no declared target groups")
    for group in groups:
        if not isinstance(group.get("document"), str):
            raise ValueError("Quire target group has no document")
        for key in ("backed", "total"):
            value = group.get(key)
            if type(value) is not int or value < 0:
                raise ValueError(f"Quire target group has invalid {key}")
        if group["backed"] > group["total"]:
            raise ValueError("Quire target group backs more rows than it contains")
    if sum(group["total"] for group in groups) != report["totals"].get("total"):
        raise ValueError("Quire coverage totals disagree with target groups")
    if sum(group["backed"] for group in groups) != report["totals"].get("backed"):
        raise ValueError("Quire coverage backed count disagrees with target groups")
    current_groups = [group for group in groups if group["document"] not in SUPERSEDED]
    current_total = sum(group["total"] for group in current_groups)
    if current_total == 0:
        raise ValueError("Quire matched no current target rows")

    def partition(rows: list[dict]) -> tuple[list[dict], list[dict]]:
        current, historical = [], []
        for row in rows:
            if not isinstance(row, dict) or not isinstance(row.get("document"), str):
                raise ValueError("Quire finding has no document")
            (historical if row["document"] in SUPERSEDED else current).append(row)
        return current, historical

    unbacked, old_unbacked = partition(report["unbacked_rows"])
    lies, old_lies = partition(report["status_lies"])
    unread = sorted({item.get("reason") for item in report["diagnostics"]
                     if isinstance(item, dict) and item.get("reason") in UNREAD_REASONS})
    if unread:
        raise ValueError(f"Quire could not evaluate declared input: {', '.join(unread)}")
    summary = {
        "current_backed_targets": sum(group["backed"] for group in current_groups),
        "current_total_targets": current_total,
        "current_unbacked_references": len(unbacked),
        "current_status_lies": len(lies),
        "superseded_unbacked_references": len(old_unbacked),
        "superseded_status_lies": len(old_lies),
    }
    return summary, unbacked, lies


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--quire", default="quire", help="Quire CLI executable")
    parser.add_argument("--scope", type=Path, default=Path("."))
    parser.add_argument("--module", action="append", default=[], help="explicit Quire module root")
    args = parser.parse_args()
    command = [args.quire, "coverage", "--scope", str(args.scope), "--json"]
    for module in args.module:
        command.extend(["--module", module])
    try:
        run = subprocess.run(command, text=True, capture_output=True, check=False)
    except OSError as error:
        print(f"current coverage gate: could not run Quire: {error}", file=sys.stderr)
        return 2
    if run.returncode:
        sys.stderr.write(run.stderr)
        print(f"current coverage gate: Quire exited {run.returncode}", file=sys.stderr)
        return 2
    try:
        summary, unbacked, lies = summarize(json.loads(run.stdout), args.scope)
    except (ValueError, OSError, UnicodeError) as error:
        print(f"current coverage gate: {error}", file=sys.stderr)
        return 2
    print("Quire current coverage: "
          f"{summary['current_backed_targets']}/{summary['current_total_targets']} "
          "target rows backed; "
          f"{summary['current_unbacked_references']} current unbacked references; "
          f"{summary['superseded_unbacked_references']} superseded references retained")
    for row in unbacked:
        print(f"  unbacked {row['document']}:{row.get('line', '?')} {row.get('row_id')}")
    for row in lies:
        print(f"  contradicted status {row['document']}:{row.get('line', '?')} "
              f"{row.get('row_id')}")
    return 1 if unbacked or lies else 0


if __name__ == "__main__":
    sys.exit(main())
