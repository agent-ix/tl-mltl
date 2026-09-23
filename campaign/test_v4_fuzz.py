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
            targets = [target] + (["closed_eval"] if name == "tl-mltl" else [])
            repo = self.root / name
            repo.mkdir()
            git(repo, "init", "-q")
            (repo / "Cargo.lock").write_bytes(b"root lock\n")
            fuzz = repo / "fuzz"
            (fuzz / "fuzz_targets").mkdir(parents=True)
            (fuzz / "corpus").mkdir()
            (fuzz / "Cargo.lock").write_bytes(b"fuzz lock\n")
            (fuzz / "Cargo.toml").write_text(
                '[package]\nname="fixture-fuzz"\nversion="0.0.0"\n'
                + ''.join(f'[[bin]]\nname="{item}"\npath="fuzz_targets/{item}.rs"\n'
                          for item in targets))
            (fuzz / "run_v4_campaign.py").write_bytes(b"runner source\n")
            if name == "tl-mltl":
                (fuzz / "run_v4_eval_campaign.py").write_bytes(b"eval runner source\n")
            seed_sets = {}
            for item in targets:
                (fuzz / "fuzz_targets" / f"{item}.rs").write_bytes(b"real target\n")
                (fuzz / "corpus" / item).mkdir()
                seeds = {}
                for index in range(3):
                    seed_name = f"seed{index}"
                    value = f"input {item} {index}\n".encode()
                    (fuzz / "corpus" / item / seed_name).write_bytes(value)
                    seeds[seed_name] = digest(value)
                (fuzz / "corpus" / item / "SHA256SUMS").write_text(
                    "".join(f"{value}  {seed}\n" for seed, value in seeds.items()))
                seed_sets[item] = seeds
            git(repo, "add", ".")
            self.commit(repo)
            measured = git(repo, "rev-parse", "HEAD")
            for item in targets:
                report_path = (v4_fuzz.EVAL_REPORT if item == "closed_eval"
                               else v4_fuzz.REPORT)
                evidence = repo / report_path.parent
                evidence.mkdir(parents=True)
                stderr = (f"Running /tmp/{item} -runs=1000\nINFO: Seed: 181\n"
                          "#1000 DONE cov: 5\n").encode()
                stdout = b""
                compressed = {stream: gzip.compress(raw, mtime=0) for stream, raw in
                              (("stdout", stdout), ("stderr", stderr))}
                for stream, data in compressed.items():
                    (evidence / f"{stream}.log.gz").write_bytes(data)
                seeds = seed_sets[item]
                report = {
                "schema": "tl-v4.libfuzzer-campaign/v1", "crate": name,
                "source_revision": measured, "target": item,
                "engine": "libFuzzer", "sanitizer": "address",
                "tool_versions": {"rustc": "nightly rustc", "cargo": "cargo 1.0",
                                  "cargo_fuzz": "cargo-fuzz 0.13.1"},
                "root_lock_sha256": digest(b"root lock\n"),
                "fuzz_lock_sha256": digest(b"fuzz lock\n"),
                "seed_files_sha256": seeds,
                "starting_corpus_sha256": digest(json.dumps(
                    seeds, sort_keys=True, separators=(",", ":")).encode()),
                "command": ["cargo", "fuzz", "run", "--sanitizer", "address", item,
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
                (repo / report_path).write_text(json.dumps(report))
            git(repo, "add", ".")
            self.commit(repo)
            self.graph[name] = {"path": str(repo), "revision": git(repo, "rev-parse", "HEAD")}

    @staticmethod
    def commit(repo: Path) -> None:
        git(repo, "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
            "commit", "-qm", "fixture")

    def test_five_complete_raw_source_pinned_runs_are_counted(self) -> None:
        status, population, raw = v4_fuzz.verify_four(self.graph)
        self.assertEqual(status, "passed")
        self.assertEqual(population["observed_executions"], 5000)
        self.assertEqual(population["target_count"], 5)
        self.assertEqual(set(raw), set(v4_fuzz.TARGETS) | {"tl-mltl/closed_eval"})
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

    def test_missing_or_changed_evaluation_evidence_refuses_v4(self) -> None:
        repo = Path(self.graph["tl-mltl"]["path"])
        report_path = repo / v4_fuzz.EVAL_REPORT
        original = report_path.read_bytes()
        report_path.unlink()
        status, population, _ = v4_fuzz.verify_four(self.graph)
        self.assertEqual(status, "incomplete")
        self.assertEqual(population["observed_executions"], 4000)
        report_path.write_bytes(original)
        (repo / "fuzz/fuzz_targets/closed_eval.rs").write_bytes(b"changed target\n")
        status, population, _ = v4_fuzz.verify_four(self.graph)
        self.assertEqual(status, "incomplete")
        self.assertIn("changed", population["crates"]["tl-mltl/closed_eval"]["reason"])

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
