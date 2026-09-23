"""V4 gate fault controls for raw engine evidence and source identity."""

from __future__ import annotations

import gzip
import hashlib
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

import v1_campaign
import v4_fuzz


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def git(root: Path, *args: str) -> str:
    return subprocess.check_output(["git", *args], cwd=root, text=True,
                                   stderr=subprocess.DEVNULL).strip()


class V4FuzzTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.graph = {}
        for name, target in v4_fuzz.TARGETS.items():
            repo = self.root / name
            repo.mkdir()
            git(repo, "init", "-q")
            (repo / "Cargo.lock").write_bytes(b"root lock\n")
            fuzz = repo / "fuzz"
            (fuzz / "fuzz_targets").mkdir(parents=True)
            (fuzz / "corpus" / target).mkdir(parents=True)
            (fuzz / "Cargo.lock").write_bytes(b"fuzz lock\n")
            (fuzz / "Cargo.toml").write_bytes(b"fuzz manifest\n")
            (fuzz / "run_v4_campaign.py").write_bytes(b"runner source\n")
            (fuzz / "fuzz_targets" / f"{target}.rs").write_bytes(b"real target\n")
            seeds = {}
            for index in range(3):
                seed_name = f"seed{index}"
                value = f"input {index}\n".encode()
                (fuzz / "corpus" / target / seed_name).write_bytes(value)
                seeds[seed_name] = digest(value)
            (fuzz / "corpus" / target / "SHA256SUMS").write_text(
                "".join(f"{value}  {seed}\n" for seed, value in seeds.items()))
            git(repo, "add", ".")
            self.commit(repo)
            measured = git(repo, "rev-parse", "HEAD")
            evidence = repo / v4_fuzz.REPORT.parent
            evidence.mkdir(parents=True)
            stderr = (f"Running /tmp/{target} -runs=1000\nINFO: Seed: 181\n"
                      "#1000 DONE cov: 5\n").encode()
            stdout = b""
            compressed = {stream: gzip.compress(raw, mtime=0) for stream, raw in
                          (("stdout", stdout), ("stderr", stderr))}
            for stream, data in compressed.items():
                (evidence / f"{stream}.log.gz").write_bytes(data)
            report = {
                "schema": "tl-v4.libfuzzer-campaign/v1", "crate": name,
                "source_revision": measured, "target": target,
                "engine": "libFuzzer", "sanitizer": "address",
                "tool_versions": {"rustc": "nightly rustc", "cargo": "cargo 1.0",
                                  "cargo_fuzz": "cargo-fuzz 0.13.1"},
                "root_lock_sha256": digest(b"root lock\n"),
                "fuzz_lock_sha256": digest(b"fuzz lock\n"),
                "seed_files_sha256": seeds,
                "starting_corpus_sha256": digest(json.dumps(
                    seeds, sort_keys=True, separators=(",", ":")).encode()),
                "command": ["cargo", "fuzz", "run", "--sanitizer", "address", target,
                            "/tmp/corpus", "--", "-runs=1000", "-seed=181",
                            "-max_total_time=30", "-max_len=4096",
                            "-artifact_prefix=/tmp/artifacts/"],
                "budget": {"runs": 1000, "seed": 181, "seconds": 30,
                           "max_len": 4096},
                "observed": {"executions": 1000, "exit_code": 0,
                             "stop_reason": "run_budget"},
                "raw_output_sha256": {f"{stream}.log.gz": digest(data)
                                      for stream, data in compressed.items()},
                "raw_stream_sha256": {"stdout": digest(stdout), "stderr": digest(stderr)},
                "crash_artifacts_sha256": {},
                "replay": {"required": False, "confirmed": False,
                           "minimized_artifact_sha256": None},
                "status": "bounded_no_crash",
            }
            (repo / v4_fuzz.REPORT).write_text(json.dumps(report))
            git(repo, "add", ".")
            self.commit(repo)
            self.graph[name] = {"path": str(repo), "revision": git(repo, "rev-parse", "HEAD")}

    @staticmethod
    def commit(repo: Path) -> None:
        git(repo, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
            "commit", "-qm", "fixture")

    def test_four_complete_raw_source_pinned_runs_are_counted(self) -> None:
        status, population, raw = v4_fuzz.verify_four(self.graph)
        self.assertEqual(status, "passed")
        self.assertEqual(population["observed_executions"], 4000)
        self.assertEqual(set(raw), set(v4_fuzz.TARGETS))
        lane = {"id": "fuzz_replay", "milestone": "V4", "mode": "native",
                "seed": {"kind": "fixed", "value": 181}}
        result, _ = v1_campaign.run_lane(lane, self.graph, {}, self.root / "raw")
        self.assertEqual(result["status"], "passed")

    def test_short_or_tampered_run_cannot_earn_credit(self) -> None:
        repo = Path(self.graph["tl-mltl"]["path"])
        report_path = repo / v4_fuzz.REPORT
        report = json.loads(report_path.read_text())
        report["observed"]["executions"] = 999
        report_path.write_text(json.dumps(report))
        self.assertEqual(v4_fuzz.verify_four(self.graph)[0], "incomplete")
        report["observed"]["executions"] = 1000
        report_path.write_text(json.dumps(report))
        with (report_path.parent / "stderr.log.gz").open("ab") as stream:
            stream.write(b"tamper")
        self.assertEqual(v4_fuzz.verify_four(self.graph)[0], "incomplete")

    def test_wrong_pin_or_crash_artifact_cannot_earn_credit(self) -> None:
        repo = Path(self.graph["tl-mltl"]["path"])
        report_path = repo / v4_fuzz.REPORT
        report = json.loads(report_path.read_text())
        report["source_revision"] = "0" * 40
        report_path.write_text(json.dumps(report))
        self.assertEqual(v4_fuzz.verify_four(self.graph)[0], "incomplete")
        report["source_revision"] = git(repo, "rev-parse", "HEAD^")
        report_path.write_text(json.dumps(report))
        (report_path.parent / "artifacts").mkdir()
        (report_path.parent / "artifacts" / "crash").write_bytes(b"bad")
        self.assertEqual(v4_fuzz.verify_four(self.graph)[0], "incomplete")

    def test_arbitrary_command_is_not_a_v4_gate(self) -> None:
        lane = {"id": "fuzz_replay", "milestone": "V4", "mode": "command",
                "repo": "tl-mltl", "parser": "cargo_test",
                "argv": ["python3", "-c", "print('test result: ok. 1 passed; "
                         "0 failed; 0 ignored; 0 measured; 0 filtered out')"],
                "seed": {"kind": "fixed", "value": 181}}
        result, _ = v1_campaign.run_lane(lane, self.graph, {}, self.root / "raw")
        self.assertEqual(result["status"], "incomplete")


if __name__ == "__main__":
    unittest.main()
