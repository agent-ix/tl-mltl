#!/usr/bin/env python3
"""Run and report the V1 verification gates without inferring human approval."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

from v4_fuzz import verify_four

SOURCE_NAMES = ("tl-syntax", "tl-parse", "tl-rewrite", "tl-mltl", "tl-oracle")
# Every milestone names an executable lane. Additional campaign lanes can be
# recorded independently, but cannot replace these required gates.
REQUIRED = {
    "V1": ("independent_oracle", "oracle_fault_injection",
           "oracle_dependency_boundary", "production_finite_faults",
           "production_infinite_faults", "finite_lasso_oracle"),
    "V2": ("finite_small_partition", "full_domain_census"),
    "V3": ("semantic_properties",),
    "V4": ("fuzz_replay",),
    "V5": ("mutation_population",),
    "V6": ("bounded_proof",),
    "V7": ("embedded_miri_limits",),
    "V8": ("coverage",),
    "V9": ("performance",),
    "V10": ("live_r2u2",),
    "V11": ("infinite_oracle", "infinite_trace_behavior",
            "oracle_semantic_laws", "lasso_population_census"),
}
STATUSES = {"passed", "failed", "incomplete", "blocked", "not_run"}
FORBIDDEN_CLAIMS = {
    "accepted", "approval", "certified", "certification", "human_acceptance",
    "native_parity", "release", "released", "tl_215_accepted",
}
# These are the only presently qualified local milestone commands. A test
# summary from another command is useful raw evidence but cannot close a gate.
COMMAND_CONTRACTS = {
    "independent_oracle": ("tl-mltl", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--test", "oracle_finite_past",
    ]),
    "oracle_fault_injection": ("tl-oracle", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--test", "reference",
    ]),
    "oracle_dependency_boundary": ("tl-oracle", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--test", "dependency_boundary",
    ]),
    "production_finite_faults": ("tl-rewrite", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--test", "oracle_finite_rewrite",
    ]),
    "production_infinite_faults": ("tl-rewrite", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--features", "infinite-trace",
        "--test", "infinite_rules",
    ]),
    "finite_lasso_oracle": ("tl-mltl", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--features", "infinite-trace",
        "--test", "infinite_oracle",
    ]),
    "finite_small_partition": ("tl-mltl", "cargo_population", [
        "cargo", "test", "--locked", "--offline", "--test", "v1_finite_partition",
        "--", "--nocapture",
    ]),
    "infinite_oracle": ("tl-mltl", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--features", "infinite-trace",
        "--test", "infinite_oracle",
    ]),
    "infinite_trace_behavior": ("tl-mltl", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--features", "infinite-trace",
        "--test", "infinite_trace",
    ]),
    "oracle_semantic_laws": ("tl-oracle", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--test", "semantic_laws",
    ]),
}
NATIVE_CONTRACTS = {
    "fuzz_replay": "four_crates_five_source_pinned_libfuzzer_targets_and_raw_streams",
}
# A gate without a native output parser and exact invocation is intentionally
# open. Extend COMMAND_CONTRACTS together with classify() and fault tests when
# its producer emits a machine-checkable population; do not credit prose or a
# copied success exit code.
UNSUPPORTED_GATE_REASONS = {
    "full_domain_census": "depth_three_interval_0_4_trace_1_6_population_not_run",
    "semantic_properties": "no_per_criterion_population_output",
    "mutation_population": "no_reviewed_mutant_population_parser",
    "bounded_proof": "no_bound_and_unwind_parser",
    "embedded_miri_limits": "no_combined_target_miri_limit_parser",
    "coverage": "no_four_crate_branch_coverage_parser",
    "performance": "no_four_crate_paired_benchmark_parser",
    "live_r2u2": "no_fresh_target_receipt_parser",
    "lasso_population_census": "no_complete_lasso_fairness_partial_population_output",
}
assert set(COMMAND_CONTRACTS) | set(NATIVE_CONTRACTS) | set(UNSUPPORTED_GATE_REASONS) == {
    item for ids in REQUIRED.values() for item in ids
}
MEASUREMENT_LANES = {
    "domain_cardinalities": "finite_small_partition",
    "fuzz_populations": "fuzz_replay",
    "mutation_populations": "mutation_population",
    "proof_bounds": "bounded_proof",
    "coverage": "coverage",
    "performance": "performance",
    "live_target": "live_r2u2",
}
CARGO_RESULT = re.compile(
    r"test result: (ok|FAILED)\. (\d+) passed; (\d+) failed; "
    r"(\d+) ignored; (\d+) measured; (\d+) filtered out"
)


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def canonical(value: Any) -> bytes:
    return json.dumps(value, sort_keys=True, separators=(",", ":")).encode()


def reject_claims(value: Any) -> None:
    if isinstance(value, dict):
        forbidden = FORBIDDEN_CLAIMS.intersection(value)
        if forbidden:
            raise ValueError(f"human/release/parity/certification claims are forbidden: {forbidden}")
        for child in value.values():
            reject_claims(child)
    elif isinstance(value, list):
        for child in value:
            reject_claims(child)


def git_revision(path: Path) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"], cwd=path, capture_output=True, text=True, check=True
    )
    return result.stdout.strip()


def git_dirty(path: Path) -> bool:
    result = subprocess.run(
        ["git", "status", "--porcelain"], cwd=path, capture_output=True, text=True, check=True
    )
    return bool(result.stdout.strip())


def source_graph(manifest: dict[str, Any]) -> dict[str, dict[str, str]]:
    sources = manifest["sources"]
    if set(sources) != set(SOURCE_NAMES):
        raise ValueError("report requires exactly four production sources and tl-oracle")
    paths = [Path(sources[name]["path"]).resolve() for name in SOURCE_NAMES]
    if len(set(paths)) != len(paths):
        raise ValueError("source repositories must have distinct paths")
    graph = {}
    for name in SOURCE_NAMES:
        source = sources[name]
        path = Path(source["path"]).resolve()
        expected = source["revision"]
        if not re.fullmatch(r"[0-9a-f]{40}", expected):
            raise ValueError(f"{name}: revision must be an exact Git commit")
        actual = git_revision(path)
        if actual != expected:
            raise ValueError(f"{name}: stale source revision {expected}, actual {actual}")
        if git_dirty(path):
            raise ValueError(f"{name}: uncommitted source changes")
        cargo_toml = (path / "Cargo.toml").read_bytes()
        cargo_lock = (path / "Cargo.lock").read_bytes()
        cargo = tomllib.loads(cargo_toml.decode())
        if cargo.get("package", {}).get("name") != name:
            raise ValueError(f"{name}: Cargo package name does not match source label")
        dependencies = {}
        for section in ("dependencies", "dev-dependencies", "build-dependencies"):
            for dependency, pin in sorted(cargo.get(section, {}).items()):
                if isinstance(pin, dict):
                    dependencies[f"{section}.{dependency}"] = {
                        key: pin[key] for key in
                        ("version", "git", "rev", "features", "default-features")
                        if key in pin
                    }
                else:
                    dependencies[f"{section}.{dependency}"] = {"version": pin}
        graph[name] = {
            "revision": actual, "path": str(path),
            "package_name": name,
            "cargo_toml_sha256": sha256(cargo_toml),
            "cargo_lock_sha256": sha256(cargo_lock),
            "feature_declarations": cargo.get("features", {}),
            "direct_dependency_pins": dependencies,
        }
    return graph


def input_graph(manifest: dict[str, Any]) -> dict[str, dict[str, str]]:
    inputs = {}
    for name, entry in sorted(manifest.get("inputs", {}).items()):
        path = Path(entry["path"]).resolve()
        actual = sha256(path.read_bytes())
        if actual != entry["sha256"]:
            raise ValueError(f"{name}: stale input digest")
        inputs[name] = {"path": str(path), "sha256": actual}
    return inputs


def classify(raw: bytes, parser: str, exit_code: int) -> tuple[str, dict[str, Any]]:
    """Derive status only from captured bytes and process exit, never a receipt verdict."""
    if parser == "cargo_test":
        matches = CARGO_RESULT.findall(raw.decode(errors="replace"))
        if not matches:
            return ("failed" if exit_code else "incomplete", {"reason": "no_test_summary"})
        passed = sum(int(match[1]) for match in matches)
        failed = sum(int(match[2]) for match in matches)
        ignored = sum(int(match[3]) for match in matches)
        measured = sum(int(match[4]) for match in matches)
        filtered = sum(int(match[5]) for match in matches)
        population = {
            "passed": passed, "failed": failed, "ignored": ignored,
            "measured": measured, "filtered": filtered,
        }
        if failed or exit_code:
            return "failed", population
        if passed == 0 or ignored or filtered:
            return "incomplete", population
        return "passed", population
    if parser in ("population_json", "cargo_population"):
        try:
            if parser == "cargo_population":
                summaries = CARGO_RESULT.findall(raw.decode(errors="replace"))
                markers = re.findall(
                    rb"TL_CAMPAIGN_POPULATION (\{[^\r\n]*\})", raw
                )
                if len(summaries) != 1 or int(summaries[0][1]) < 2 or len(markers) != 1:
                    raise ValueError("missing or duplicated native census/test summary")
                if summaries[0][0] != "ok" or int(summaries[0][2]) != 0:
                    return "failed", {"reason": "native_test_failure"}
                observed = json.loads(markers[0])
                if (
                    observed.get("schema") != "tl-mltl.finite-partition/v1"
                    or observed.get("scope") != "depth1_atom1_closed0_2_words1_3"
                    or observed.get("atom_basis") != ["p0"]
                    or observed.get("max_depth") != 1
                    or observed.get("interval_max") != 2
                    or observed.get("trace_max_len") != 3
                    or observed.get("full_target_complete") is not False
                    or observed.get("formulas") != 375
                    or observed.get("word_positions") != 34
                    or observed.get("declared") != 375 * 34
                ):
                    raise ValueError("wrong finite partition scope")
            else:
                observed = json.loads(raw)
            counts = {key: observed[key] for key in ("declared", "visited", "refused", "failed")}
            if any(type(value) is not int or value < 0 for value in counts.values()):
                raise ValueError("non-natural population")
        except (ValueError, KeyError, TypeError):
            return "failed", {"reason": "malformed_population"}
        if parser == "cargo_population":
            counts["scope"] = observed["scope"]
            counts["formulas"] = observed["formulas"]
            counts["word_positions"] = observed["word_positions"]
            counts["full_target_complete"] = False
            counts["native_test_count"] = int(summaries[0][1])
        if counts["visited"] + counts["refused"] > counts["declared"]:
            return "failed", counts | {"reason": "duplicate_or_excess_visits"}
        if counts["failed"] or exit_code:
            return "failed", counts
        if counts["visited"] + counts["refused"] < counts["declared"]:
            return "incomplete", counts | {"reason": "unvisited_population"}
        if counts["visited"] == 0:
            return "incomplete", counts | {"reason": "exhausted_or_vacuous"}
        return "passed", counts
    raise ValueError(f"unknown output parser: {parser}")


def read_record(record_path: Path, graph: dict, inputs: dict, parser: str) -> tuple[bytes, bytes, int, dict]:
    """Import captured raw output only with an exact source/input receipt."""
    receipt = json.loads(record_path.read_text())
    reject_claims(receipt)
    revisions = {name: entry["revision"] for name, entry in graph.items()}
    digests = {name: entry["sha256"] for name, entry in inputs.items()}
    if receipt.get("source_revisions") != revisions or receipt.get("input_sha256") != digests:
        raise ValueError(f"stale external receipt: {record_path}")
    if receipt.get("parser") != parser:
        raise ValueError(f"parser mismatch: {record_path}")
    paths = {}
    raw = {}
    for stream in ("stdout", "stderr"):
        path = Path(receipt[stream]["path"]).resolve()
        data = path.read_bytes()
        if sha256(data) != receipt[stream]["sha256"]:
            raise ValueError(f"stale or changed {stream}: {path}")
        paths[stream] = {"path": str(path), "sha256": sha256(data)}
        raw[stream] = data
    code = receipt["exit_code"]
    if type(code) is not int:
        raise ValueError("record exit_code must be an integer")
    return raw["stdout"], raw["stderr"], code, paths


def run_lane(lane: dict, graph: dict, inputs: dict, raw_dir: Path) -> tuple[dict, dict]:
    lane_id = lane["id"]
    mode = lane["mode"]
    seed = lane.get("seed", {"kind": "not_reported"})
    if not isinstance(seed, dict) or seed.get("kind") not in ("fixed", "none", "not_reported"):
        raise ValueError(f"invalid seed identity for {lane_id}")
    if seed["kind"] == "fixed" and ("value" not in seed or type(seed["value"]) not in (int, str)):
        raise ValueError(f"missing fixed seed for {lane_id}")
    if seed["kind"] == "none" and not seed.get("reason"):
        raise ValueError(f"missing deterministic seed reason for {lane_id}")
    base = {"id": lane_id, "milestone": lane["milestone"], "mode": mode, "seed": seed}
    if mode == "not_run":
        return base | {"status": "not_run", "reason": lane.get("reason", "not_invoked")}, {}
    if mode == "blocked":
        return base | {"status": "blocked", "reason": lane["reason"]}, {}
    if mode == "native":
        if (lane_id not in NATIVE_CONTRACTS or set(lane) != {
                "id", "milestone", "mode", "seed"}
                or seed != {"kind": "fixed", "value": 181}):
            return base | {"status": "incomplete",
                           "reason": "unregistered_native_gate"}, {}
        status, population, raw = verify_four(graph)
        return base | {"status": status, "population": population,
                       "parser": "v4_fuzz_raw_reconciliation"}, raw
    parser = lane["parser"]
    if mode == "command":
        repo = lane["repo"]
        if repo not in graph:
            raise ValueError(f"unknown lane repo {repo}")
        argv = lane["argv"]
        if not isinstance(argv, list) or not argv or not all(isinstance(x, str) for x in argv):
            raise ValueError(f"invalid argv for {lane_id}")
        if lane_id in {item for ids in REQUIRED.values() for item in ids}:
            expected = COMMAND_CONTRACTS.get(lane_id)
            if expected is None or (repo, parser, argv) != expected:
                return base | {
                    "status": "incomplete", "reason": "unregistered_required_gate",
                    "argv": argv, "repo": repo,
                    "gate_contract": UNSUPPORTED_GATE_REASONS.get(lane_id, "exact_command_mismatch"),
                }, {}
        timeout = lane.get("timeout_seconds", 600)
        if type(timeout) is not int or not 1 <= timeout <= 86400:
            raise ValueError(f"invalid timeout for {lane_id}")
        try:
            result = subprocess.run(argv, cwd=graph[repo]["path"], capture_output=True, timeout=timeout)
            stdout, stderr, code = result.stdout, result.stderr, result.returncode
            timed_out = False
        except OSError as error:
            return base | {
                "status": "blocked", "reason": f"tool_unavailable:{error.filename}",
                "argv": argv, "repo": repo,
            }, {}
        except subprocess.TimeoutExpired as error:
            stdout, stderr, code = error.stdout or b"", error.stderr or b"", 124
            timed_out = True
        raw_dir.mkdir(parents=True, exist_ok=True)
        paths = {}
        for stream, data in (("stdout", stdout), ("stderr", stderr)):
            path = raw_dir / f"{lane_id}.{stream}"
            path.write_bytes(data)
            paths[stream] = {"path": str(path.resolve()), "sha256": sha256(data)}
        status, population = classify(stdout + b"\n" + stderr, parser, code)
        if timed_out:
            status = "incomplete"
            population["reason"] = "timeout"
        if base["seed"]["kind"] == "not_reported" and status == "passed":
            status = "incomplete"
            population["reason"] = "seed_not_reported"
        semantic = base | {
            "status": status, "parser": parser, "argv": argv, "repo": repo,
            "exit_code": code, "population": population,
        }
        return semantic, paths
    if mode == "record":
        try:
            stdout, stderr, code, paths = read_record(Path(lane["receipt"]), graph, inputs, parser)
        except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError) as error:
            return base | {"status": "incomplete", "reason": f"unusable_record:{type(error).__name__}"}, {}
        status, population = classify(stdout + b"\n" + stderr, parser, code)
        observed_status = status
        if lane_id in {item for ids in REQUIRED.values() for item in ids}:
            status = "incomplete"
            population["reason"] = "imported_record_not_executable_gate"
        return base | {
            "status": status, "parser": parser, "exit_code": code,
            "population": population, "observed_status": observed_status,
        }, paths
    raise ValueError(f"unknown lane mode: {mode}")


def gate_status(states: list[str]) -> str:
    if "failed" in states:
        return "failed"
    if "blocked" in states:
        return "blocked"
    if "incomplete" in states or "not_run" in states:
        return "incomplete"
    return "passed"


def tool_versions(command_cwd: Path) -> dict[str, str | None]:
    versions = {}
    for tool, argv in {
        "python": [sys.executable, "--version"],
        "git": ["git", "--version"],
        "rustc": ["rustc", "-Vv"],
        "cargo": ["cargo", "-V"],
    }.items():
        try:
            result = subprocess.run(
                argv, cwd=command_cwd, capture_output=True, text=True, check=False
            )
            versions[tool] = (result.stdout or result.stderr).strip() if result.returncode == 0 else None
        except OSError:
            versions[tool] = None
        versions[f"{tool}_path"] = shutil.which(argv[0])
    return versions


def build_report(manifest: dict, raw_dir: Path) -> dict:
    if manifest.get("schema") != "tl-mltl.v1-campaign-manifest/v1":
        raise ValueError("unsupported campaign manifest")
    reject_claims(manifest)
    graph = source_graph(manifest)
    inputs = input_graph(manifest)
    defined = {}
    for lane in manifest.get("lanes", []):
        lane_id = lane["id"]
        if lane_id in defined or not re.fullmatch(r"[a-z][a-z0-9_]*", lane_id):
            raise ValueError(f"duplicate or invalid lane ID: {lane_id}")
        if lane["milestone"] not in REQUIRED:
            raise ValueError(f"unknown milestone: {lane['milestone']}")
        defined[lane_id] = lane
    semantic_lanes, raw = {}, {}
    for milestone, required_ids in REQUIRED.items():
        for lane_id in required_ids:
            lane = defined.pop(lane_id, {"id": lane_id, "milestone": milestone, "mode": "not_run"})
            if lane["milestone"] != milestone:
                raise ValueError(f"misplaced required lane {lane_id}")
            semantic_lanes[lane_id], raw[lane_id] = run_lane(lane, graph, inputs, raw_dir)
    for lane_id, lane in sorted(defined.items()):
        semantic_lanes[lane_id], raw[lane_id] = run_lane(lane, graph, inputs, raw_dir)
    milestones = {}
    for milestone, required_ids in REQUIRED.items():
        states = [semantic_lanes[lane_id]["status"] for lane_id in required_ids]
        states += [lane["status"] for lane in semantic_lanes.values()
                   if lane["milestone"] == milestone and lane["id"] not in required_ids]
        milestones[milestone] = {
            "gate": list(required_ids), "status": gate_status(states),
            "contract": {
                lane_id: (
                    {"kind": "exact_command", "repo": COMMAND_CONTRACTS[lane_id][0],
                     "parser": COMMAND_CONTRACTS[lane_id][1],
                     "argv": COMMAND_CONTRACTS[lane_id][2]}
                    if lane_id in COMMAND_CONTRACTS else
                    {"kind": "native", "identity": NATIVE_CONTRACTS[lane_id]}
                    if lane_id in NATIVE_CONTRACTS else
                    {"kind": "unsupported", "reason": UNSUPPORTED_GATE_REASONS[lane_id]}
                )
                for lane_id in required_ids
            },
            "lane_statuses": {lane["id"]: lane["status"] for lane in semantic_lanes.values()
                              if lane["milestone"] == milestone},
        }
    semantic = {
        "source_revisions": {name: entry["revision"] for name, entry in graph.items()},
        "source_pins": {
            name: {key: entry[key] for key in
                   ("package_name", "cargo_toml_sha256", "cargo_lock_sha256", "feature_declarations",
                    "direct_dependency_pins")}
            for name, entry in graph.items()
        },
        "input_sha256": {name: entry["sha256"] for name, entry in inputs.items()},
        "tool_versions": tool_versions(Path(graph["tl-mltl"]["path"])),
        "measurements": {
            name: {
                "lane": lane_id,
                "status": semantic_lanes[lane_id]["status"],
                "population": semantic_lanes[lane_id].get("population"),
            }
            for name, lane_id in MEASUREMENT_LANES.items()
        },
        "lanes": semantic_lanes,
        "milestones": milestones,
        "aggregate_status": gate_status([item["status"] for item in milestones.values()]),
        "claim_boundary": "automated_evidence_only",
    }
    return {
        "schema": "tl-mltl.v1-campaign-report/v1",
        "semantic_payload": semantic,
        "semantic_sha256": sha256(canonical(semantic)),
        "raw_artifacts": raw,
        "source_paths": {name: entry["path"] for name, entry in graph.items()},
        "input_paths": {name: entry["path"] for name, entry in inputs.items()},
    }


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--raw-dir", type=Path, required=True)
    args = parser.parse_args()
    manifest = json.loads(args.manifest.read_text())
    report = build_report(manifest, args.raw_dir)
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n")


if __name__ == "__main__":
    main()
