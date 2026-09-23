"""Verify a fresh four-crate cargo-mutants run from native archives."""

from __future__ import annotations

import hashlib
import json
import tarfile
from pathlib import Path

import v5_mutation
import v5_run

CRATES = ("tl-syntax", "tl-parse", "tl-mltl", "tl-rewrite")
SCOPE = {
    "tl-syntax": {
        "source_file": "src/formula/infinite.rs",
        "selection_regex": "TemporalInterval::start|select_infinite_profile|validate_resource_limits|preflight_resource_limits|InfiniteFormulaDocument::content_identity",
        "critical_scope": "V5 selected critical semantics",
        "test_tail": ["--test", "infinite_formula", "--test", "infinite_trace",
                      "--test", "infinite_trace_corpus"],
        "minimum_selected": 18,
    },
    "tl-parse": {
        "source_file": "src/infinite.rs",
        "selection_regex": "parse_clean_ascii_v4|Parser.*::interval|Parser.*::lower_derived",
        "critical_scope": "V5 selected critical semantics",
        "test_tail": ["--test", "infinite_v4", "--test", "infinite_trace_corpus",
                      "--test", "owner_infinite_corpus"],
        "minimum_selected": 23,
    },
    "tl-rewrite": {
        "source_file": "src/infinite.rs",
        "selection_regex": "check_infinite_rewrite|classify_infinite_results",
        "critical_scope": "V5 selected critical semantics",
        "test_tail": ["--lib", "--test", "infinite_conformance", "--test",
                      "infinite_rules", "--test", "infinite_owner_corpus"],
        "minimum_selected": 39,
    },
    "tl-mltl": {
        "source_file": "src/infinite/mod.rs",
        "selection_regex": "evaluate_trace|evaluate_lasso",
        "critical_scope": "infinite trace/lasso evaluator",
        "test_tail": ["--lib", "--test", "infinite_trace", "--test", "infinite_oracle"],
        "minimum_selected": 43,
    },
}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def unique_pairs(pairs: list[tuple[str, object]]) -> dict:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate V5 JSON key: {key}")
        result[key] = value
    return result


def selection(path: Path, graph: dict) -> dict:
    selected = json.loads(path.read_bytes(), object_pairs_hook=unique_pairs)
    if selected.get("schema") != "tl-mltl.v5-mutation-selection/v1":
        raise ValueError("wrong V5 selection schema")
    rows = selected.get("runs")
    if (not isinstance(rows, list) or len(rows) != len(CRATES) or
            not all(isinstance(row, dict) for row in rows)):
        raise ValueError("V5 selection must name exactly four owning crates")
    if {row.get("crate") for row in rows} != set(CRATES):
        raise ValueError("V5 owning-crate identity changed")
    for row in rows:
        crate = row["crate"]
        if (Path(row["source_path"]).resolve() != Path(graph[crate]["path"]).resolve() or
                row["source_revision"] != graph[crate]["revision"]):
            raise ValueError(f"V5 source graph changed: {crate}")
        source_file = Path(row["source_file"])
        if (source_file.is_absolute() or ".." in source_file.parts or
                source_file.suffix != ".rs" or source_file.parts[0] != "src"):
            raise ValueError(f"V5 selection is not a production Rust file: {crate}")
        if (not isinstance(row.get("critical_scope"), str) or
                not row["critical_scope"].strip() or not isinstance(row.get("test_tail"), list) or
                not row["test_tail"] or not all(isinstance(x, str) and x for x in row["test_tail"])):
            raise ValueError(f"V5 critical scope or test selection is empty: {crate}")
        if not isinstance(row.get("selection_regex"), str) or not row["selection_regex"]:
            raise ValueError(f"V5 mutant selection regex is absent: {crate}")
        if any(row.get(key) != value for key, value in SCOPE[crate].items()
               if key != "minimum_selected"):
            raise ValueError(f"V5 reviewed critical selection changed: {crate}")
    return selected


def valid_native_exit(code: int, counts: dict) -> bool:
    """cargo-mutants 27 returns 2 for completed missed/timeout populations."""
    expected = 2 if counts["missed"] or counts["timed_out"] else 0
    return type(code) is int and code == expected


def verify(selection_path: Path, output_dir: Path, graph: dict) -> tuple[str, dict, dict]:
    """Recompute the score and survivor inventory from all four native archives."""
    selected = selection(selection_path, graph)
    manifest_path = output_dir / "manifest.json"
    report_path = output_dir / "report.json"
    manifest_bytes = manifest_path.read_bytes()
    report_bytes = report_path.read_bytes()
    manifest = json.loads(manifest_bytes, object_pairs_hook=unique_pairs)
    claimed = json.loads(report_bytes, object_pairs_hook=unique_pairs)
    if manifest.get("schema") != "tl-mltl.v5-mutation-manifest/v1" or not isinstance(
            manifest.get("runs"), list) or len(manifest["runs"]) != len(CRATES):
        raise ValueError("V5 native manifest has incomplete run population")
    artifacts = {
        "manifest": {"path": str(manifest_path.resolve()), "sha256": digest(manifest_bytes)},
        "report": {"path": str(report_path.resolve()), "sha256": digest(report_bytes)},
        "selection": {"path": str(selection_path.resolve()),
                      "sha256": digest(selection_path.read_bytes())},
    }
    invocation_codes = {}
    for entry, source in zip(manifest["runs"], selected["runs"], strict=True):
        crate = source["crate"]
        fields = ("crate", "source_path", "source_revision", "source_file",
                  "selection_regex", "critical_scope", "test_tail", "survivor_reviews")
        if any(entry.get(field, [] if field == "survivor_reviews" else None) !=
               source.get(field, [] if field == "survivor_reviews" else None)
               for field in fields):
            raise ValueError(f"V5 native run differs from fixed selection: {crate}")
        crate_dir = (output_dir / crate).resolve()
        expected_paths = {"discovery": crate_dir / "discovery.json",
                          "archive": crate_dir / "native.tar.gz"}
        for kind, path in expected_paths.items():
            if (Path(entry[f"{kind}_path"]).resolve() != path.resolve() or
                    entry[f"{kind}_sha256"] != digest(path.read_bytes())):
                raise ValueError(f"V5 raw path/digest mismatch: {crate}/{kind}")
            artifacts[f"{crate}.{kind}"] = {"path": str(path.resolve()),
                                             "sha256": entry[f"{kind}_sha256"]}
        with tarfile.open(expected_paths["archive"], "r:gz") as archive:
            names = archive.getnames()
            if not {"invocation.json", "invocation.stdout", "invocation.stderr"} <= set(names):
                raise ValueError(f"V5 native command streams missing: {crate}")
            member = archive.extractfile("invocation.json")
            if member is None:
                raise ValueError(f"V5 native command metadata missing: {crate}")
            invocation = json.loads(member.read(), object_pairs_hook=unique_pairs)
        expected_command = v5_run.command(source["source_file"], source["selection_regex"],
                                          crate_dir / "native", source["test_tail"])
        expected_discovery = ["cargo", "mutants", "--no-config", "--all-features",
                              "--list", "--json", "--file", source["source_file"]]
        if (invocation.get("source_revision") != source["source_revision"] or
                invocation.get("discovery_command") != expected_discovery or
                invocation.get("mutation_command") != expected_command or
                type(invocation.get("exit_code")) is not int):
            raise ValueError(f"V5 native command or revision mismatch: {crate}")
        invocation_codes[crate] = invocation["exit_code"]
    measured = v5_mutation.report(manifest, manifest_path.parent)
    if claimed != measured:
        raise ValueError("V5 native result was restamped")
    for crate in CRATES:
        if measured["runs"][crate]["selected"] < SCOPE[crate]["minimum_selected"]:
            raise ValueError(f"V5 selected population shrank: {crate}")
        if not valid_native_exit(invocation_codes[crate], measured["runs"][crate]):
            raise ValueError(f"V5 native cargo-mutants exited abnormally: {crate}")
    population = {"declared_crates": 4, "visited_crates": len(measured["runs"]),
                  "runs": measured["runs"], "selection_sha256": artifacts["selection"]["sha256"]}
    return measured["status"], population, artifacts
