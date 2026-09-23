#!/usr/bin/env python3
"""Run and report the V1 verification gates without inferring human approval."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path
from typing import Any

from v4_fuzz import verify_four
from v6_kani import run_v6
import v7_gate
import v8_gate
import v5_gate
import v9_gate
import tl217_grid_gate

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
    "V10": ("live_r2u2", "live_past_grid"),
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
    "semantic_properties": ("tl-mltl", "cargo_properties", [
        "cargo", "test", "--locked", "--offline", "--all-features",
        "--test", "property", "--test", "infinite_trace", "--test", "infinite_oracle",
        "--", "--nocapture", "--test-threads=1",
    ]),
    "embedded_miri_limits": ("tl-mltl", "v7_native", [
        sys.executable, "campaign/v7_native.py",
    ]),
    "mutation_population": ("tl-mltl", "v5_native", [
        sys.executable, "campaign/v5_run.py",
    ]),
    "coverage": ("tl-mltl", "v8_native", [
        sys.executable, "campaign/v8_coverage.py",
    ]),
    "performance": ("tl-mltl", "v9_native", [
        sys.executable, "campaign/v9_criterion.py", "report",
    ]),
    "infinite_oracle": ("tl-mltl", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--features", "infinite-trace",
        "--test", "infinite_oracle",
    ]),
    "live_r2u2": ("tl-mltl", "cargo_live_target", [
        "cargo", "run", "--locked", "--offline", "--features", "infinite-trace",
        "--example", "v1_live_r2u2",
    ]),
    "live_past_grid": ("tl-mltl", "tl217_live_grid", [
        "cargo", "run", "--locked", "--offline", "--example", "tl217_live_past_grid",
    ]),
    "infinite_trace_behavior": ("tl-mltl", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--features", "infinite-trace",
        "--test", "infinite_trace",
    ]),
    "oracle_semantic_laws": ("tl-oracle", "cargo_test", [
        "cargo", "test", "--locked", "--offline", "--test", "semantic_laws",
    ]),
    "lasso_population_census": ("tl-mltl", "cargo_v11_population", [
        "cargo", "test", "--locked", "--offline", "--features", "infinite-trace",
        "--test", "v11_lasso_partition", "--", "--nocapture",
    ]),
}
NATIVE_CONTRACTS = {
    "fuzz_replay": "four_crates_five_source_pinned_libfuzzer_targets_and_raw_streams",
    "bounded_proof": "live_two_harness_kani_and_seeded_false_replay",
}
# A gate without a native output parser and exact invocation is intentionally
# open. Extend COMMAND_CONTRACTS together with classify() and fault tests when
# its producer emits a machine-checkable population; do not credit prose or a
# copied success exit code.
UNSUPPORTED_GATE_REASONS = {
    "full_domain_census": "depth_three_interval_0_4_trace_1_6_population_not_run",
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
LIVE_TARGET_REVISION = "336a2453dd2bd89bd26e9e45fb772a4bf77e4a6a"
LIVE_COMPILER_SHA256 = "f978a32f667a8247c387a66bce35371c97b7d8f7b730035a8ee40cdfc428ce12"
LIVE_MONITOR_SHA256 = "5743987dddb47cc01829a633e15623095c9c2aff2f8bb24e30d7f0e0f488f85f"


def live_target_source(path: str) -> tuple[Path, str | None]:
    """Check the explicit foreign source preimage before invoking anything."""
    source = Path(path).resolve()
    if not source.is_dir():
        return source, "target_source_absent"
    try:
        if git_revision(source) != LIVE_TARGET_REVISION or git_dirty(source):
            return source, "target_source_revision_or_tree_mismatch"
        for relative, expected in (
            ("compiler/c2po.py", LIVE_COMPILER_SHA256),
            ("monitors/c/build/r2u2", LIVE_MONITOR_SHA256),
        ):
            if sha256((source / relative).read_bytes()) != expected:
                return source, f"target_binary_digest_mismatch:{relative}"
    except (OSError, subprocess.CalledProcessError):
        return source, "target_source_unreadable"
    return source, None
V1_CRITERION_PREFIXES = tuple(
    f"FR-{number:03d}-" for number in (*range(27, 35), *range(38, 56))
)
PROPERTY_LAWS = {
    "duality", "bounded_embedding", "lasso_unrolling", "fairness_weakening",
    "partial_information_monotonicity", "finite_prefix_refutation",
}


def v1_criterion_ids() -> set[str]:
    requirements = Path(__file__).resolve().parent.parent / "spec" / "requirements"
    ids = set()
    for path in requirements.glob("FR-*.md"):
        if not path.name.startswith(V1_CRITERION_PREFIXES):
            continue
        ids.update(re.findall(r"^\| (FR-\d+-AC-\d+) \|", path.read_text(), re.M))
    return ids


def unique_object(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate property marker key: {key}")
        result[key] = value
    return result


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
    if parser == "cargo_live_target":
        try:
            markers = re.findall(rb"TL_CAMPAIGN_LIVE_TARGET (\{[^\r\n]*\})", raw)
            if len(markers) != 1:
                raise ValueError("missing or duplicate native live marker")
            if exit_code:
                return "failed", {"reason": "live_target_example_failed"}
            marker = json.loads(markers[0])
            if (
                marker.get("schema") != "tl-mltl.live-r2u2/v1"
                or marker.get("source_revision") != LIVE_TARGET_REVISION
                or marker.get("compiler_sha256") != LIVE_COMPILER_SHA256
                or marker.get("monitor_sha256") != LIVE_MONITOR_SHA256
                or marker.get("license") != "Apache-2.0"
                or (marker.get("bounded_cells"), marker.get("past_cells"), marker.get("unsafe_cells"))
                != (8, 18, 1)
                or marker.get("bad_prefix") != {
                    "basis": "bad_prefix", "disposition": "refuted",
                    "oracle": "refuted", "violation_position": 0,
                    "target_case": "r2u2-globally-counterexample-v1",
                    "target_position": 0, "target_verdict": False,
                }
            ):
                raise ValueError("wrong exact target or census")
            artifacts = marker["artifacts"]
            expected_artifacts = {
                f"{case}.{kind}" for case in ("bounded", "past", "unsafe-since")
                for kind in ("bin", "compiler.stdout", "compiler.stderr",
                             "r2u2.stdout", "r2u2.stderr")
            }
            if set(artifacts) != expected_artifacts or any(
                not isinstance(digest, str) or not re.fullmatch(r"[0-9a-f]{64}", digest)
                for digest in artifacts.values()
            ):
                raise ValueError("missing or malformed raw target artifacts")
            expected_runs = {
                "bounded": ("corpus/r2u2-v4.2/formulas.c2po",
                            "corpus/r2u2-v4.2/trace.csv",
                            "corpus/r2u2-v4.2/signals.map"),
                "past": ("corpus/past-c2po-v1/target-4.2/past.c2po",
                         "corpus/past-c2po-v1/target-4.2/trace.csv", None),
                "unsafe-since": ("corpus/past-c2po-v1/target-4.2/unsafe-since.c2po",
                                 "corpus/past-c2po-v1/target-4.2/unsafe-since.csv", None),
            }
            runs = marker["runs"]
            if set(runs) != set(expected_runs):
                raise ValueError("missing target command result")
            for name, (spec, trace, signal_map) in expected_runs.items():
                run = runs[name]
                if run != {"compiler_exit": 0, "monitor_exit": 0,
                           "spec": spec, "trace": trace, "map": signal_map}:
                    raise ValueError("wrong target command or exit state")
            rows = marker["classifications"]
            if not isinstance(rows, list) or len(rows) != 27:
                raise ValueError("wrong per-step population")
            seen = set()
            bounded_cases = {
                "r2u2-future-witness-v1": 0,
                "r2u2-globally-counterexample-v1": 0,
                "r2u2-future-deadline-v1": 0,
                "r2u2-until-lower-bound-v1": 0,
                "r2u2-release-lower-bound-v1": 0,
                "r2u2-nested-until-v1": 0,
                "r2u2-future-at-one-v1": 1,
                "r2u2-globally-at-one-v1": 1,
            }
            past_cases = {
                "once-zero-one", "historically-zero-one", "previous",
                "since-zero-two", "triggered-zero-two", "once-one-one",
            }
            expected_cells = (
                {("bounded", case, at) for case, at in bounded_cases.items()}
                | {("past", case, at) for case in past_cases for at in range(3)}
                | {("past", "unsafe-since", 2)}
            )
            counts = {"bounded_agreement": 0, "past_agreement": 0,
                      "unsupported_mapping": 0}
            for row in rows:
                key = (row["family"], row["case"], row["position"])
                if key in seen or type(row["position"]) is not int or row["position"] < 0:
                    raise ValueError("duplicate or invalid target cell")
                seen.add(key)
                if type(row["oracle"]) is not bool or type(row["target"]) is not bool:
                    raise ValueError("invalid target truth value")
                if row["classification"] == "agreement":
                    if row["oracle"] != row["target"]:
                        raise ValueError("false agreement")
                    category = f"{row['family']}_agreement"
                    if category not in counts:
                        raise ValueError("unknown agreement family")
                    counts[category] += 1
                elif row["classification"] == "unsupported_mapping":
                    if row["family"] != "past" or row["case"] not in (
                        "since-zero-two", "triggered-zero-two", "unsafe-since"
                    ):
                        raise ValueError("unreviewed mapping refusal")
                    if row["case"] == "unsafe-since" and (row["oracle"], row["target"]) != (False, True):
                        raise ValueError("known origin mismatch disappeared")
                    counts["unsupported_mapping"] += 1
                else:
                    raise ValueError("unrecognized target classification")
            if seen != expected_cells:
                raise ValueError("target cell census mismatch")
            bad_cell = next(row for row in rows if (
                row["family"], row["case"], row["position"]
            ) == ("bounded", "r2u2-globally-counterexample-v1", 0))
            if bad_cell["target"] is not False or bad_cell["oracle"] is not False:
                raise ValueError("bad-prefix target witness mismatch")
            if counts != {"bounded_agreement": 8, "past_agreement": 12,
                          "unsupported_mapping": 7}:
                raise ValueError("classification counts do not reconcile")
        except (ValueError, KeyError, TypeError, json.JSONDecodeError):
            return "failed", {"reason": "malformed_live_target_population"}
        return "passed", {
            **counts, "declared": 27, "visited": len(seen),
            "source_revision": marker["source_revision"],
            "compiler_sha256": marker["compiler_sha256"],
            "monitor_sha256": marker["monitor_sha256"],
            "license": marker["license"],
            "bad_prefix": marker["bad_prefix"],
            "classifications": rows, "artifacts": artifacts,
            "runs": runs,
        }
    if parser == "cargo_v11_population":
        decoded = raw.decode(errors="replace")
        summaries = CARGO_RESULT.findall(decoded)
        markers = re.findall(rb"TL_CAMPAIGN_V11_POPULATION (\{[^\r\n]*\})", raw)
        if len(summaries) != 1 or len(markers) != 1:
            return "failed", {"reason": "missing_or_duplicate_native_v11_census"}
        summary = summaries[0]
        if (summary[0] != "ok" or int(summary[1]) < 1 or
                any(int(value) for value in summary[2:]) or exit_code):
            return "failed", {"reason": "native_v11_test_failure"}
        try:
            observed = json.loads(markers[0])
            expected_axes = {
                "schema": "tl-mltl.v11-lasso-partition/v1",
                "scope": "formulas30_words372_fair3_anchors4",
                "formula_count": 30,
                "complete_words": 228,
                "single_unknown_words": 136,
                "mixed_words": 8,
                "word_count": 372,
                "fairness_modes": 3,
                "anchors": [0, 1, 3, 6],
                "max_materialized_lasso_len": 3,
                "full_target_complete": False,
            }
            if any(type(observed.get(key)) is not type(value) or observed.get(key) != value
                   for key, value in expected_axes.items()):
                raise ValueError("wrong V11 partition scope")
            if any(type(anchor) is not int for anchor in observed["anchors"]):
                raise ValueError("non-integer V11 anchor")
            counts = {key: observed[key] for key in ("declared", "visited", "refused", "failed")}
            if any(type(value) is not int or value < 0 for value in counts.values()):
                raise ValueError("non-natural V11 population")
        except (ValueError, KeyError, TypeError):
            return "failed", {"reason": "malformed_v11_population"}
        counts |= {"scope": observed["scope"], "full_target_complete": False,
                   "native_test_count": int(summary[1])}
        if counts["declared"] != 133_920:
            return "failed", counts | {"reason": "wrong_v11_declared_population"}
        accounted = counts["visited"] + counts["refused"]
        if counts["failed"] or accounted > counts["declared"]:
            return "failed", counts | {"reason": "failed_or_excess_v11_cases"}
        if accounted < counts["declared"]:
            return "incomplete", counts | {"reason": "unvisited_v11_population"}
        if counts["visited"] != 108_720 or counts["refused"] != 25_200:
            return "failed", counts | {"reason": "wrong_v11_admission_partition"}
        return "passed", counts
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
    if parser == "cargo_properties":
        decoded = raw.decode(errors="replace")
        summaries = CARGO_RESULT.findall(decoded)
        markers = re.findall(rb"TL_CAMPAIGN_PROPERTIES (\{[^\r\n]*\})", raw)
        if exit_code or any(item[0] != "ok" or int(item[2]) for item in summaries):
            return "failed", {"reason": "native_test_failure"}
        if not summaries or sum(int(item[1]) for item in summaries) < 3 or len(markers) != 1:
            return "failed", {"reason": "missing_or_duplicate_property_population"}
        try:
            observed = json.loads(markers[0], object_pairs_hook=unique_object)
            cases = observed["law_cases"]
            classes = observed["classifications"]
            if not isinstance(cases, dict) or not isinstance(classes, dict):
                raise ValueError("property cases and classifications must be objects")
            expected = v1_criterion_ids()
            if (
                observed["schema"] != "tl-mltl.semantic-properties/v1"
                or observed["scope"] != "tl_mltl_v1_semantic_laws_and_owner_wires"
                or observed["seed_hex"] != "45" * 32
                or type(observed["generated"]) is not int
                or observed["generated"] != 64
                or type(observed["accepted"]) is not int
                or observed["accepted"] != 64
                or type(observed["rejected"]) is not int
                or observed["rejected"] != 0
                or set(cases) != PROPERTY_LAWS
                or any(type(count) is not int or count != 64 for count in cases.values())
                or type(observed["wire_checks"]) is not int
                or observed["wire_checks"] != 24
                or observed["rewrite_equivalence_owner"] != "tl-rewrite"
                or len(expected) != 61
                or set(classes) != expected
            ):
                raise ValueError("property population or criterion scope differs")
            # With --nocapture, the property test prints its marker between
            # Cargo's test prefix and the final `ok` line.
            passed_names = set(re.findall(
                r"^test (?:[\w]+::)*([\w]+) \.\.\. "
                r"(?:TL_CAMPAIGN_PROPERTIES \{[^\r\n]*\}\r?\n)?ok$",
                decoded, re.M,
            ))
            if not {"native_semantic_laws_and_strict_round_trips", "seeded_law_fault_is_detected"} <= passed_names:
                raise ValueError("native property or fault control did not run")
            kinds = {"property": 0, "example": 0, "excluded": 0}
            for classification in classes.values():
                kind = classification["kind"]
                evidence = classification["evidence"]
                if kind not in kinds or not isinstance(evidence, str) or not evidence:
                    raise ValueError("unclassified criterion")
                if kind == "property" and evidence not in PROPERTY_LAWS | {"strict_round_trips"}:
                    raise ValueError("unexecuted property")
                if kind == "example" and evidence not in passed_names:
                    raise ValueError("unexecuted example")
                if kind == "excluded" and ":" not in evidence:
                    raise ValueError("unjustified exclusion")
                kinds[kind] += 1
            if kinds["property"] == 0 or kinds["example"] == 0:
                raise ValueError("vacuous classification")
        except (ValueError, KeyError, TypeError, json.JSONDecodeError):
            return "failed", {"reason": "malformed_property_population"}
        return "passed", {
            "declared": len(expected), "classified": len(classes),
            "classification_kinds": kinds, "generated": observed["generated"],
            "accepted": observed["accepted"], "rejected": observed["rejected"],
            "law_cases": cases, "wire_checks": observed["wire_checks"],
            "native_test_count": sum(int(item[1]) for item in summaries),
        }
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
        accepted_seeds = {
            "fuzz_replay": {"kind": "fixed", "value": 181},
            "bounded_proof": {"kind": "none", "reason": "symbolic_no_random_seed"},
        }
        if (lane_id not in NATIVE_CONTRACTS or set(lane) != {
                "id", "milestone", "mode", "seed"}
                or seed != accepted_seeds[lane_id]):
            return base | {"status": "incomplete",
                           "reason": "unregistered_native_gate"}, {}
        status, population, raw = (verify_four(graph) if lane_id == "fuzz_replay"
                                   else run_v6(graph, raw_dir))
        return base | {"status": status, "population": population,
                       "parser": ("v4_fuzz_raw_reconciliation" if lane_id == "fuzz_replay"
                                  else "v6_kani_live_proof_replay")}, raw
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
        environment = None
        target_raw_dir = None
        v7_raw_dir = None
        v7_report_path = None
        v8_raw_dir = None
        v8_report_path = None
        v5_output_dir = None
        v5_selection_path = None
        v9_report_path = None
        v9_pair_dirs = None
        executed_argv = argv
        if lane_id in ("live_r2u2", "live_past_grid"):
            target_source = lane.get("target_source")
            if not isinstance(target_source, str) or not Path(target_source).is_absolute():
                return base | {"status": "incomplete", "reason": "explicit_target_source_required"}, {}
            source, reason = live_target_source(target_source)
            if reason:
                return base | {"status": "blocked", "reason": reason}, {}
            target_raw_dir = raw_dir / f"{lane_id}_target"
            if target_raw_dir.exists() and any(target_raw_dir.iterdir()):
                return base | {"status": "incomplete", "reason": "target_raw_dir_not_empty"}, {}
            target_raw_dir.mkdir(parents=True, exist_ok=True)
            environment = os.environ.copy()
            environment["TL_MLTL_C2PO_SOURCE"] = str(source)
            environment["TL_MLTL_LIVE_RAW_DIR"] = str(target_raw_dir.resolve())
        if lane_id == "embedded_miri_limits":
            cargo_home = lane.get("cargo_home")
            if (not isinstance(cargo_home, str) or not Path(cargo_home).is_absolute() or
                    not Path(cargo_home).is_dir()):
                return base | {"status": "blocked", "reason": "v7_cargo_home_required"}, {}
            v7_raw_dir = raw_dir / "embedded_miri_limits_native"
            v7_report_path = raw_dir / "embedded_miri_limits_native.json"
            if v7_report_path.exists() or v7_raw_dir.exists():
                return base | {"status": "incomplete", "reason": "v7_raw_dir_not_empty"}, {}
            executed_argv = argv + [
                *[piece for name in ("syntax", "parse", "mltl", "rewrite", "oracle")
                  for piece in (f"--{name}", graph[f"tl-{name}"]["path"])],
                "--cargo-home", str(Path(cargo_home).resolve()),
                "--raw-dir", str(v7_raw_dir.resolve()),
                "--output", str(v7_report_path.resolve()),
            ]
        if lane_id == "coverage":
            cargo_home = lane.get("cargo_home")
            if (not isinstance(cargo_home, str) or not Path(cargo_home).is_absolute() or
                    not Path(cargo_home).is_dir()):
                return base | {"status": "blocked", "reason": "v8_cargo_home_required"}, {}
            v8_raw_dir = raw_dir / "coverage-native"
            v8_report_path = raw_dir / "coverage-native.json"
            if v8_report_path.exists() or v8_raw_dir.exists():
                return base | {"status": "incomplete", "reason": "v8_raw_dir_not_empty"}, {}
            executed_argv = argv + [
                *[piece for name in ("syntax", "parse", "mltl", "rewrite")
                  for piece in (f"--{name}", graph[f"tl-{name}"]["path"])],
                "--cargo-home", str(Path(cargo_home).resolve()),
                "--raw-dir", str(v8_raw_dir.resolve()),
                "--output", str(v8_report_path.resolve()),
            ]
        if lane_id == "mutation_population":
            selected = lane.get("selection_path")
            if not isinstance(selected, str) or not Path(selected).is_absolute():
                return base | {"status": "blocked", "reason": "v5_selection_required"}, {}
            v5_selection_path = Path(selected).resolve()
            try:
                selected_digest = sha256(v5_selection_path.read_bytes())
            except OSError:
                return base | {"status": "blocked", "reason": "v5_selection_unreadable"}, {}
            if inputs.get("v5_selection") != {"path": str(v5_selection_path),
                                               "sha256": selected_digest}:
                return base | {"status": "incomplete", "reason": "v5_selection_not_pinned"}, {}
            try:
                v5_gate.selection(v5_selection_path, graph)
            except (OSError, ValueError, KeyError, TypeError, json.JSONDecodeError):
                return base | {"status": "incomplete", "reason": "v5_selection_graph_mismatch"}, {}
            v5_output_dir = raw_dir / "mutation_population_native"
            if v5_output_dir.exists():
                return base | {"status": "incomplete", "reason": "v5_raw_dir_not_empty"}, {}
            executed_argv = argv + ["--selection", str(v5_selection_path),
                                    "--output-dir", str(v5_output_dir.resolve())]
        if lane_id == "performance":
            pairs = lane.get("pair_dirs")
            if (not isinstance(pairs, list) or not pairs or
                    any(not isinstance(path, str) or not Path(path).is_absolute()
                        for path in pairs)):
                return base | {"status": "blocked", "reason": "v9_pair_dirs_required"}, {}
            v9_pair_dirs = [Path(path).resolve() for path in pairs]
            if len(set(v9_pair_dirs)) != len(v9_pair_dirs):
                return base | {"status": "incomplete", "reason": "v9_duplicate_pair"}, {}
            for index, path in enumerate(v9_pair_dirs):
                pair_file = path / "pair.json"
                try:
                    pair_digest = sha256(pair_file.read_bytes())
                except OSError:
                    return base | {"status": "blocked", "reason": "v9_pair_unreadable"}, {}
                if inputs.get(f"v9_pair_{index}") != {
                    "path": str(pair_file.resolve()), "sha256": pair_digest,
                }:
                    return base | {"status": "incomplete", "reason": "v9_pair_not_pinned"}, {}
            v9_report_path = raw_dir / "performance-native.json"
            if v9_report_path.exists():
                return base | {"status": "incomplete", "reason": "v9_report_path_not_empty"}, {}
            executed_argv = argv + [piece for path in v9_pair_dirs
                                    for piece in ("--pair-dir", str(path))] + [
                                        "--output", str(v9_report_path.resolve())]
        try:
            result = subprocess.run(
                executed_argv, cwd=graph[repo]["path"], capture_output=True,
                timeout=timeout, env=environment,
            )
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
        if lane_id == "embedded_miri_limits":
            assert v7_report_path is not None
            if code:
                status, population = "failed", {"reason": "v7_native_process_failed"}
            else:
                try:
                    tools = v7_gate.current_tools(Path(graph[repo]["path"]))
                    population, native_artifacts = v7_gate.verify(
                        v7_report_path.read_bytes(), stdout, raw_dir, graph, tools
                    )
                    paths["native_artifacts"] = native_artifacts
                    status = "passed"
                except (OSError, ValueError, TypeError, KeyError, subprocess.TimeoutExpired):
                    status, population = "failed", {"reason": "malformed_v7_native_evidence"}
        elif lane_id == "coverage":
            assert v8_report_path is not None and v8_raw_dir is not None
            try:
                tools = v8_gate.tool_versions()
                status, population, native_artifacts = v8_gate.verify(
                    v8_report_path.read_bytes(), v8_raw_dir, graph, tools
                )
                if code not in (0, 1) or (status == "passed") != (code == 0):
                    raise ValueError("V8 process/coverage status mismatch")
                paths["native_artifacts"] = native_artifacts
            except (OSError, ValueError, TypeError, KeyError, subprocess.TimeoutExpired,
                    subprocess.CalledProcessError, json.JSONDecodeError):
                status, population = "failed", {"reason": "malformed_v8_native_evidence"}
        elif lane_id == "mutation_population":
            assert v5_output_dir is not None and v5_selection_path is not None
            try:
                status, population, native_artifacts = v5_gate.verify(
                    v5_selection_path, v5_output_dir, graph
                )
                if code not in (0, 1) or (status == "passed") != (code == 0):
                    raise ValueError("V5 process/score status mismatch")
                paths["native_artifacts"] = native_artifacts
            except (OSError, ValueError, TypeError, KeyError, subprocess.TimeoutExpired,
                    subprocess.CalledProcessError, json.JSONDecodeError):
                status, population = "failed", {"reason": "malformed_v5_native_evidence"}
        elif lane_id == "performance":
            assert v9_report_path is not None and v9_pair_dirs is not None
            try:
                status, population, native_artifacts = v9_gate.verify(
                    v9_report_path.read_bytes(), v9_report_path, v9_pair_dirs, graph, inputs
                )
                if code not in (0, 1) or (status == "passed") != (code == 0):
                    raise ValueError("V9 process/report status mismatch")
                paths["native_artifacts"] = native_artifacts
            except (OSError, ValueError, TypeError, KeyError, subprocess.TimeoutExpired,
                    subprocess.CalledProcessError, json.JSONDecodeError):
                status, population = "failed", {"reason": "malformed_v9_native_evidence"}
        elif lane_id == "live_past_grid":
            assert target_raw_dir is not None
            try:
                tl217_grid_gate.verify_target_source(Path(lane["target_source"]))
                population = tl217_grid_gate.verify(
                    stdout, target_raw_dir, graph["tl-mltl"]["revision"],
                    graph["tl-mltl"]["cargo_lock_sha256"],
                )
                if code != 0:
                    raise ValueError("live grid producer failed")
                status = "passed"
                parsed = json.loads(stdout.split(tl217_grid_gate.MARKER, 1)[1])
                paths["target_artifacts"] = {
                    name: {"path": str((target_raw_dir / name).resolve()), "sha256": digest}
                    for name, digest in parsed["artifacts"].items()
                }
            except (OSError, ValueError, TypeError, KeyError, json.JSONDecodeError,
                    subprocess.CalledProcessError):
                status, population = "failed", {"reason": "malformed_live_past_grid"}
        else:
            status, population = classify(stdout + b"\n" + stderr, parser, code)
        if lane_id == "live_r2u2" and status == "passed":
            assert target_raw_dir is not None
            target_paths = {}
            for name, expected in population["artifacts"].items():
                path = target_raw_dir / name
                try:
                    actual = sha256(path.read_bytes())
                except OSError:
                    actual = None
                if actual != expected:
                    status = "failed"
                    population["reason"] = f"stale_or_missing_target_artifact:{name}"
                    break
                target_paths[name] = {"path": str(path.resolve()), "sha256": actual}
            if status == "passed":
                paths["target_artifacts"] = target_paths
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
        if lane_id == "embedded_miri_limits":
            semantic["executed_argv"] = executed_argv
        if lane_id in ("live_r2u2", "live_past_grid"):
            semantic["target_source"] = lane["target_source"]
        return semantic, paths
    if mode == "record":
        if lane_id == "embedded_miri_limits":
            return base | {"status": "incomplete", "reason": "v7_requires_live_execution"}, {}
        if lane_id == "live_past_grid":
            return base | {"status": "incomplete", "reason": "live_grid_requires_execution"}, {}
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
