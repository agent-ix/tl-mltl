"""Reconcile the four bounded V4 libFuzzer runs against source and raw bytes."""

from __future__ import annotations

import gzip
import hashlib
import json
import re
import subprocess
from pathlib import Path

TARGETS = {
    "tl-syntax": "infinite_wire_decode",
    "tl-parse": "unbounded_parse_roundtrip",
    "tl-rewrite": "infinite_rewrite",
    "tl-mltl": "c2po_map",
}
REPORT = Path("fuzz/evidence/v4-2026-09-23/report.json")
HEX = re.compile(r"[0-9a-f]{64}\Z")


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def checked_digest(value: str, data: bytes, label: str) -> None:
    if not isinstance(value, str) or not HEX.fullmatch(value) or digest(data) != value:
        raise ValueError(f"{label}: digest mismatch")


def git_bytes(root: Path, revision: str, relative: str) -> bytes:
    result = subprocess.run(["git", "show", f"{revision}:{relative}"], cwd=root,
                            capture_output=True, check=False)
    if result.returncode:
        raise ValueError(f"{relative}: unavailable at measured revision")
    return result.stdout


def verify_one(name: str, entry: dict) -> tuple[dict, dict]:
    root = Path(entry["path"])
    target = TARGETS[name]
    evidence = root / REPORT.parent
    report_bytes = (root / REPORT).read_bytes()
    report = json.loads(report_bytes)
    if report.get("schema") != "tl-v4.libfuzzer-campaign/v1" or report.get("crate") != name:
        raise ValueError("wrong V4 report schema or crate")
    if report.get("target") != target or report.get("engine") != "libFuzzer":
        raise ValueError("wrong V4 target or engine")
    if report.get("sanitizer") != "address":
        raise ValueError("missing address sanitizer")
    measured = report.get("source_revision", "")
    if not re.fullmatch(r"[0-9a-f]{40}", measured):
        raise ValueError("invalid measured source revision")
    ancestor = subprocess.run(["git", "merge-base", "--is-ancestor", measured,
                               entry["revision"]], cwd=root, capture_output=True)
    if ancestor.returncode:
        raise ValueError("measured source is not an ancestor of candidate")
    versions = report.get("tool_versions", {})
    if ("nightly" not in versions.get("rustc", "")
            or "cargo-fuzz " not in versions.get("cargo_fuzz", "")
            or not versions.get("cargo")):
        raise ValueError("missing nightly fuzz tool identity")
    for relative, key in (("Cargo.lock", "root_lock_sha256"),
                          ("fuzz/Cargo.lock", "fuzz_lock_sha256")):
        current = (root / relative).read_bytes()
        checked_digest(report.get(key), current, relative)
        if git_bytes(root, measured, relative) != current:
            raise ValueError(f"{relative}: changed since measured run")
    for relative in ("fuzz/Cargo.toml", "fuzz/run_v4_campaign.py",
                     f"fuzz/fuzz_targets/{target}.rs"):
        if git_bytes(root, measured, relative) != (root / relative).read_bytes():
            raise ValueError(f"{relative}: measured target changed")
    seeds = report.get("seed_files_sha256")
    if not isinstance(seeds, dict) or len(seeds) < 3:
        raise ValueError("missing checked seed population")
    corpus_dir = root / "fuzz" / "corpus" / target
    actual_names = {path.name for path in corpus_dir.iterdir() if path.is_file()}
    if actual_names != set(seeds) | {"SHA256SUMS"}:
        raise ValueError("missing or undeclared checked seed")
    manifest = (corpus_dir / "SHA256SUMS").read_text().splitlines()
    if len(manifest) != len(seeds) or set(manifest) != {
            f"{expected}  {seed_name}" for seed_name, expected in seeds.items()}:
        raise ValueError("checked seed manifest mismatch")
    for name_seed, expected in seeds.items():
        if not re.fullmatch(r"[A-Za-z0-9_.-]+", name_seed):
            raise ValueError("unsafe seed name")
        relative = f"fuzz/corpus/{target}/{name_seed}"
        current = (root / relative).read_bytes()
        checked_digest(expected, current, relative)
        if git_bytes(root, measured, relative) != current:
            raise ValueError(f"{relative}: changed since measured run")
    checked_digest(report.get("starting_corpus_sha256"),
                   json.dumps(seeds, sort_keys=True, separators=(",", ":")).encode(),
                   "starting corpus")
    budget = report.get("budget")
    if budget != {"runs": 1000, "seed": 181, "seconds": 30, "max_len": 4096}:
        raise ValueError("unreviewed V4 budget")
    argv = report.get("command", [])
    if (len(argv) != 13
            or argv[:6] != ["cargo", "fuzz", "run", "--sanitizer", "address", target]
            or Path(argv[6]).name != "corpus" or argv[7:12] != [
                "--", "-runs=1000", "-seed=181", "-max_total_time=30", "-max_len=4096"]
            or not argv[12].startswith("-artifact_prefix=")):
        raise ValueError("unreviewed fuzz invocation")
    raw = {"report": {"path": str(root / REPORT), "sha256": digest(report_bytes)}}
    streams = {}
    for stream in ("stdout", "stderr"):
        filename = f"{stream}.log.gz"
        compressed = (evidence / filename).read_bytes()
        if len(compressed) > 10_000_000:
            raise ValueError("oversized compressed engine stream")
        checked_digest(report.get("raw_output_sha256", {}).get(filename), compressed,
                       filename)
        content = gzip.decompress(compressed)
        if len(content) > 100_000_000:
            raise ValueError("oversized engine stream")
        checked_digest(report.get("raw_stream_sha256", {}).get(stream), content, stream)
        streams[stream] = content
        raw[stream] = {"path": str(evidence / filename), "sha256": digest(compressed),
                       "stream_sha256": digest(content)}
    engine_log = streams["stdout"] + b"\n" + streams["stderr"]
    done = re.findall(rb"(?m)^#(\d+)\s+DONE\b", engine_log)
    if (len(done) != 1 or done[0] != b"1000"
            or b"INFO: Seed: 181" not in engine_log
            or f"/{target} ".encode() not in engine_log):
        raise ValueError("raw engine output does not prove 1,000 target executions")
    if (report.get("observed", {}).get("executions") != 1000
            or report["observed"].get("exit_code") != 0
            or report["observed"].get("stop_reason") != "run_budget"
            or report.get("status") != "bounded_no_crash"):
        raise ValueError("reported execution count or stop disagrees with clean run")
    if report.get("crash_artifacts_sha256") != {} or report.get("replay") != {
            "required": False, "confirmed": False, "minimized_artifact_sha256": None}:
        raise ValueError("crash artifact or unconfirmed replay cannot earn clean credit")
    artifact_dir = evidence / "artifacts"
    if artifact_dir.exists() and any(artifact_dir.iterdir()):
        raise ValueError("unreported crash artifact")
    return ({"status": "passed", "target": target, "measured_revision": measured,
             "candidate_revision": entry["revision"], "executions": 1000,
             "starting_corpus_sha256": report["starting_corpus_sha256"],
             "checked_seed_count": len(seeds), "sanitizer": "address",
             "bounded_no_crash": True}, raw)


def verify_four(graph: dict) -> tuple[str, dict, dict]:
    crates, raw = {}, {}
    for name in TARGETS:
        try:
            crates[name], raw[name] = verify_one(name, graph[name])
        except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError,
                gzip.BadGzipFile) as error:
            crates[name] = {"status": "incomplete", "reason": str(error)}
            raw[name] = {}
    status = "passed" if all(item["status"] == "passed" for item in crates.values()) else "incomplete"
    return status, {"schema": "tl-mltl.v4-fuzz-population/v1",
                    "scope": "four_checked_targets_1000_executions_each",
                    "crate_count": len(crates),
                    "observed_executions": sum(item.get("executions", 0) for item in crates.values()),
                    "crates": crates,
                    "claim_boundary": "bounded_no_crash_observation"}, raw
