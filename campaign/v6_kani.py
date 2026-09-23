"""Live, bounded Kani proofs and a replayed seeded-false control for V6."""

from __future__ import annotations

import hashlib
import io
import os
import re
import subprocess
import tarfile
import tempfile
import tomllib
from pathlib import Path

CLAIMS = {
    "tl-syntax": (
        "formula::graph::kani_proofs::interval_cardinality_matches_wide_arithmetic",
        "src/formula/graph.rs", "u32 start, u32 end; no assumptions"),
    "tl-mltl": (
        "future::horizon::kani_proofs::horizon_bound_addition_matches_checked_add",
        "src/future/horizon.rs", "u32 bound, u64 child; no assumptions"),
}
FALSE_HARNESS = '''    #[kani::proof]
    fn seeded_false_cardinality_claim() {
        let start: u32 = kani::any();
        let end: u32 = kani::any();
        if let Ok(interval) = Interval::new(start, end) {
            assert!(interval.cardinality() == Some(1));
        }
    }

'''
ANCHOR = "mod kani_proofs {\n    use super::Interval;\n\n"
FALSE_NAME = "formula::graph::kani_proofs::seeded_false_cardinality_claim"
BOUNDS = {"unwind": 2, "unwinding_checks": "default_enabled",
          "memory_safety_checks": "default_enabled",
          "overflow_checks": "default_enabled",
          "assertion_reachability_checks": "default_enabled",
          "solver": "cadical"}


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(harness: str, *, mutant: bool = False) -> list[str]:
    argv = ["cargo", "kani"]
    if mutant:
        argv += ["-Z", "concrete-playback"]
    argv += ["--lib", "--harness", harness, "--exact", "--unwind", "2",
             "--solver", "cadical"]
    if mutant:
        argv += ["--concrete-playback", "print"]
    return argv


def parse_clean(raw: bytes, exit_code: int, harness: str) -> dict:
    text = raw.decode(errors="replace")
    checks = re.findall(r"(?m)^Check (\d+): ([^\n]+)\n\s*- Status: (SUCCESS|FAILURE|UNKNOWN)", text)
    summary = re.findall(r"\*\* (\d+) of (\d+) failed", text)
    assertion_checks = sum(".assertion." in item[1] and harness in item[1]
                           for item in checks)
    solver = re.findall(r"Solving with (CaDiCaL [0-9.]+)", text)
    cbmc = re.findall(r"CBMC version ([0-9.]+) \(cbmc-[0-9.]+\)", text)
    complete = (
        exit_code == 0 and len(summary) == 1 and summary[0][0] == "0"
        and len(checks) == int(summary[0][1]) and len(checks) > 0
        and [int(item[0]) for item in checks] == list(range(1, len(checks) + 1))
        and all(item[2] == "SUCCESS" for item in checks)
        and assertion_checks > 0 and len(solver) >= 1 and len(set(solver)) == 1
        and len(cbmc) >= 1 and len(set(cbmc)) == 1
        and f"Checking harness {harness}..." in text
        and "VERIFICATION:- SUCCESSFUL" in text
        and "Complete - 1 successfully verified harnesses, 0 failures, 1 total." in text
    )
    return {"status": "passed" if complete else "incomplete",
            "checks": len(checks), "assertion_checks": assertion_checks,
            "failed_checks": int(summary[0][0]) if len(summary) == 1 else None,
            "solver_identity": solver[0] if solver else None,
            "cbmc_version": cbmc[0] if cbmc else None,
            "reason": None if complete else "partial_or_failed_kani_result"}


def parse_false(raw: bytes, exit_code: int) -> dict:
    text = raw.decode(errors="replace")
    summary = re.findall(r"\*\* (\d+) of (\d+) failed", text)
    playback = text.split("Concrete playback unit test for", 1)
    values = re.findall(r"vec!\[([0-9, ]+)\]", playback[1]) if len(playback) == 2 else []
    decoded = []
    for value in values:
        try:
            row = [int(item.strip()) for item in value.split(",") if item.strip()]
        except ValueError:
            row = []
        decoded.append(row)
    valid_bytes = (len(decoded) == 2 and all(len(row) == 4 and
                   all(0 <= item <= 255 for item in row) for row in decoded))
    endpoints = [int.from_bytes(bytes(row), "little") for row in decoded] if valid_bytes else []
    reproduced = (valid_bytes and endpoints[0] <= endpoints[1] and
                  endpoints[1] - endpoints[0] + 1 != 1 and
                  endpoints[1] - endpoints[0] + 1 <= 0xffffffff)
    complete = (
        exit_code != 0 and len(summary) == 1 and summary[0][0] == "1"
        and "VERIFICATION:- FAILED" in text
        and "assertion failed: interval.cardinality() == Some(1)" in text
        and FALSE_NAME in text and reproduced
    )
    return {"status": "passed" if complete else "incomplete",
            "counterexample_bytes": decoded if valid_bytes else None,
            "endpoints": endpoints if valid_bytes else None,
            "cardinality": endpoints[1] - endpoints[0] + 1 if reproduced else None,
            "reason": None if complete else "false_claim_not_replayed_by_verifier"}


def capture(argv: list[str], cwd: Path, raw_dir: Path, name: str,
            env: dict[str, str], timeout: int = 900) -> tuple[bytes, int, dict]:
    try:
        process = subprocess.run(argv, cwd=cwd, env=env, capture_output=True,
                                 timeout=timeout, check=False)
        stdout, stderr, code = process.stdout, process.stderr, process.returncode
    except subprocess.TimeoutExpired as error:
        stdout, stderr, code = error.stdout or b"", error.stderr or b"", 124
    raw_dir.mkdir(parents=True, exist_ok=True)
    retained = {}
    for stream, data in (("stdout", stdout), ("stderr", stderr)):
        path = raw_dir / f"bounded_proof.{name}.{stream}"
        path.write_bytes(data)
        retained[stream] = {"path": str(path.resolve()), "sha256": digest(data)}
    return stdout + b"\n" + stderr, code, retained


def exact_harness_source(root: Path, relative: str, harness: str) -> dict:
    source = (root / relative).read_bytes()
    text = source.decode()
    function = harness.rsplit("::", 1)[1]
    match = re.search(r"(?s)#\[kani::proof\]\s*fn " + re.escape(function)
                      + r"\(\)\s*\{(.*?)\n    \}", text)
    if match is None or "kani::assume" in match[1] or match[1].count("kani::any()") != 2:
        raise ValueError(f"{harness}: missing or vacuous symbolic harness")
    return {"source_path": relative, "source_sha256": digest(source),
            "harness_sha256": digest(match[0].encode()), "symbolic_any_count": 2,
            "assumptions": 0}


def archive_source(root: Path, revision: str, destination: Path) -> None:
    archive = subprocess.run(["git", "archive", "--format=tar", revision],
                             cwd=root, capture_output=True, check=True).stdout
    with tarfile.open(fileobj=io.BytesIO(archive), mode="r:") as tar:
        tar.extractall(destination, filter="data")


def replay_test(rows: list[list[int]]) -> str:
    start, end = rows
    return ("use tl_syntax::Interval;\n\n"
            "#[test]\nfn seeded_false_cardinality_counterexample_replays() {\n"
            f"    let start = u32::from_le_bytes({start!r});\n"
            f"    let end = u32::from_le_bytes({end!r});\n"
            "    let interval = Interval::new(start, end).unwrap();\n"
            "    assert_ne!(interval.cardinality(), Some(1));\n"
            "}\n")


def version(argv: list[str], cwd: Path, env: dict[str, str]) -> str:
    result = subprocess.run(argv, cwd=cwd, env=env, capture_output=True,
                            text=True, check=False)
    if result.returncode:
        raise ValueError(f"tool version unavailable: {argv}")
    return (result.stdout or result.stderr).strip()


def run_v6(graph: dict, raw_dir: Path) -> tuple[str, dict, dict]:
    syntax = Path(graph["tl-syntax"]["path"])
    mltl = Path(graph["tl-mltl"]["path"])
    population = {"schema": "tl-mltl.v6-bounded-kani/v1", "bounds": BOUNDS,
                  "source_revisions": {name: graph[name]["revision"] for name in CLAIMS},
                  "claims": {}, "counterexample": {"status": "not_run"}}
    raw = {}
    try:
        channel = tomllib.loads((mltl / "rust-toolchain.toml").read_text())["toolchain"]["channel"]
        toolchain_cargo = version(["rustup", "which", "cargo", "--toolchain", channel],
                                  mltl, os.environ.copy())
        env = os.environ.copy()
        env["PATH"] = str(Path(toolchain_cargo).parent) + os.pathsep + env.get("PATH", "")
        population["tool_versions"] = {
            "kani": version(["cargo", "kani", "--version"], mltl, env),
            "cargo": version(["cargo", "-V"], mltl, env),
            "rustc": version(["rustc", "-Vv"], mltl, env),
            "cargo_path": toolchain_cargo,
        }
        if "Kani Rust Verifier 0.68.0" not in population["tool_versions"]["kani"]:
            raise ValueError("unreviewed Kani version")
        for name, (harness, relative, domain) in CLAIMS.items():
            root = syntax if name == "tl-syntax" else mltl
            source = exact_harness_source(root, relative, harness)
            output, code, raw[name] = capture(command(harness), root, raw_dir, name, env)
            population["claims"][name] = {
                "harness": harness, "symbolic_domain": domain,
                "argv": command(harness), "exit_code": code,
                **source, **parse_clean(output, code, harness),
            }
        with tempfile.TemporaryDirectory(prefix="tl-v6-syntax-") as directory:
            copy = Path(directory)
            archive_source(syntax, graph["tl-syntax"]["revision"], copy)
            relative = CLAIMS["tl-syntax"][1]
            original = (copy / relative).read_text()
            if original.count(ANCHOR) != 1 or FALSE_NAME.rsplit("::", 1)[1] in original:
                raise ValueError("seeded false harness anchor is not unique")
            mutated = original.replace(ANCHOR, ANCHOR + FALSE_HARNESS, 1)
            if mutated.replace(FALSE_HARNESS, "", 1) != original:
                raise ValueError("verifier-only mutation altered production body")
            (copy / relative).write_text(mutated)
            output, code, raw["seeded_false"] = capture(
                command(FALSE_NAME, mutant=True), copy, raw_dir, "seeded_false", env)
            false_result = parse_false(output, code)
            false_result.update({
                "argv": command(FALSE_NAME, mutant=True), "exit_code": code,
                "mutated_source_sha256": digest(mutated.encode()),
                "production_subject_unchanged": True,
            })
            if false_result["status"] == "passed":
                test_path = copy / "tests" / "v6_counterexample_replay.rs"
                test_path.write_text(replay_test(false_result["counterexample_bytes"]))
                output, code, raw["ordinary_replay"] = capture(
                    ["cargo", "test", "--locked", "--offline", "--test",
                     "v6_counterexample_replay"], copy, raw_dir, "ordinary_replay", env)
                summary = re.findall(rb"test result: ok\. 1 passed; 0 failed; 0 ignored;", output)
                false_result["ordinary_replay"] = {
                    "status": "passed" if code == 0 and len(summary) == 1 else "incomplete",
                    "exit_code": code, "test_sha256": digest(test_path.read_bytes()),
                    "argv": ["cargo", "test", "--locked", "--offline", "--test",
                             "v6_counterexample_replay"],
                }
            population["counterexample"] = false_result
    except (OSError, ValueError, subprocess.SubprocessError, tarfile.TarError) as error:
        population["reason"] = f"verifier_unavailable_or_incomplete:{type(error).__name__}:{error}"
    claims = population["claims"]
    passed = (len(claims) == 2 and all(item["status"] == "passed" for item in claims.values())
              and population["counterexample"].get("status") == "passed"
              and population["counterexample"].get("ordinary_replay", {}).get("status") == "passed"
              and not population.get("reason"))
    return ("passed" if passed else "incomplete"), population, raw
