"""Reconcile retained Criterion pairs with the current three-crate source graph."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

import v9_criterion


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unique_pairs(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate V9 JSON key: {key}")
        result[key] = value
    return result


def verify(report_bytes: bytes, report_path: Path, pair_dirs: list[Path], graph: dict,
           inputs: dict) -> tuple[str, dict, dict]:
    report = json.loads(report_bytes, object_pairs_hook=unique_pairs)
    if report.get("schema") != "tl-mltl.v9-criterion-report/v1":
        raise ValueError("V9 report schema mismatch")
    paths = [path.resolve() for path in pair_dirs]
    if len(paths) != len(set(paths)) or not paths:
        raise ValueError("V9 pair population empty or duplicated")
    if report.get("pair_dirs") != [str(path) for path in paths]:
        raise ValueError("V9 pair identity changed")
    artifacts = {"report": {"path": str(report_path.resolve()),
                            "sha256": digest(report_bytes)}}
    pairs = []
    for index, path in enumerate(paths):
        pair_path = path / "pair.json"
        raw = pair_path.read_bytes()
        pinned = {"path": str(pair_path.resolve()), "sha256": digest(raw)}
        if inputs.get(f"v9_pair_{index}") != pinned:
            raise ValueError(f"V9 pair {index} changed from manifest pin")
        pair = json.loads(raw, object_pairs_hook=unique_pairs)
        if pair.get("status") != "incomplete":
            v9_criterion.validate_pair(path, pair)
        pairs.append(pair)
        artifacts[f"pair_{index}"] = pinned
    if any(pair.get("status") == "incomplete" for pair in pairs):
        if (report.get("status") != "incomplete" or
                report.get("reason") != "a_baseline_or_candidate_benchmark_failed"):
            raise ValueError("V9 failed pair was restamped")
        return "incomplete", {"declared_pairs": len(paths), "visited_pairs": len(pairs),
                              "reason": report["reason"]}, artifacts
    if report.get("pair_count") != len(paths) or report.get("threshold") != 0.20 or (
            report.get("required_pairs") != 2):
        raise ValueError("V9 threshold or pair census changed")
    candidate = {name: graph[f"tl-{name}"]["revision"] for name in v9_criterion.GROUPS}
    source_ids = report.get("source_revisions")
    if (not isinstance(source_ids, dict) or set(source_ids) != set(candidate) or
            any(not isinstance(source_ids[name], list) or len(source_ids[name]) != 2 or
                source_ids[name][1] != revision for name, revision in candidate.items())):
        raise ValueError("V9 candidate source is not the current graph")
    for pair in pairs:
        for name in v9_criterion.GROUPS:
            source = pair["crates"][name]["candidate_source"]
            if (source["revision"] != candidate[name] or
                    source["manifest_sha256"] != graph[f"tl-{name}"]["cargo_toml_sha256"] or
                    source["lock_sha256"] != graph[f"tl-{name}"]["cargo_lock_sha256"]):
                raise ValueError(f"V9 candidate source pin mismatch: {name}")
    first = pairs[0]
    comparable = all(pair.get("status") == "measured" and
                     pair.get("host_before") == pair.get("host_after") == first.get("host_before")
                     and all(pair["crates"][name]["copied_harness_sha256"] ==
                             first["crates"][name]["copied_harness_sha256"] and
                             (pair["crates"][name]["baseline_source"]["revision"],
                              pair["crates"][name]["candidate_source"]["revision"]) ==
                             tuple(source_ids[name]) for name in v9_criterion.GROUPS)
                     for pair in pairs)
    if not comparable:
        if report.get("status") != "inconclusive_host":
            raise ValueError("V9 unmatched host or harness was marked comparable")
        if report.get("cases") != []:
            raise ValueError("V9 unmatched host claimed comparable cases")
        return "incomplete", {"declared_pairs": len(paths),
                              "visited_pairs": len(paths),
                              "reason": "host_or_toolchain_mismatch"}, artifacts
    if report.get("host") != first["host_before"]:
        raise ValueError("V9 report host differs from native pairs")
    cases = report.get("cases")
    expected = {(name, case) for name, (_, _, names) in v9_criterion.GROUPS.items()
                for case in names}
    if (not isinstance(cases, list) or
            {(row.get("crate"), row.get("case")) for row in cases} != expected or
            len(cases) != len(expected)):
        raise ValueError("V9 benchmark case population changed")
    for row in cases:
        if len(row.get("runs", [])) != len(paths):
            raise ValueError("V9 case is missing a paired distribution")
        for index, run in enumerate(row["runs"]):
            pair = pairs[index]
            native = pair["crates"][row["crate"]]["cases"][row["case"]]
            if (run.get("baseline") != native["baseline"] or
                    run.get("candidate") != native["candidate"]):
                raise ValueError("V9 report distributions differ from raw pair")
            expected_run = v9_criterion.classify_pair(
                native["baseline"], native["candidate"],
                f"{row['crate']}/{row['case']}/{paths[index].name}",
            )
            if run != expected_run:
                raise ValueError("V9 paired change classification was restamped")
            for side in ("baseline", "candidate"):
                retained = paths[index] / "samples" / row["crate"] / row["case"] / side
                if v9_criterion.samples(retained) != native[side]:
                    raise ValueError("V9 raw Criterion distribution changed")
        confirmed = sum(run["status"] == "repeat_required_above_20pct"
                        for run in row["runs"])
        expected_case = ("confirmed_above_20pct" if confirmed >= 2 else
                         "repeat_required" if confirmed == 1 and len(paths) < 3 else
                         "one_run_spike" if confirmed == 1 else
                         "inconclusive_overlap" if any(run["status"] ==
                             "inconclusive_threshold_overlap" for run in row["runs"]) else
                         "below_20pct_threshold")
        if row.get("status") != expected_case:
            raise ValueError("V9 case classification was restamped")
    status = report.get("status")
    if status == "passed":
        if len(paths) < 2 or any(row.get("status") != "below_20pct_threshold" for row in cases):
            raise ValueError("V9 unconfirmed regression or missing repeat was marked passed")
        gate_status = "passed"
    elif status == "finding_required":
        if not any(row.get("status") == "confirmed_above_20pct" for row in cases):
            raise ValueError("V9 finding lacks confirmed regression")
        gate_status = "failed"
    elif status in {"incomplete", "repeat_required", "inconclusive_noise",
                    "inconclusive_host"}:
        gate_status = "incomplete"
    else:
        raise ValueError("unknown V9 report status")
    return gate_status, {"declared_pairs": len(paths), "visited_pairs": len(paths),
                         "cases": cases, "source_revisions": source_ids,
                         "report_status": status}, artifacts
