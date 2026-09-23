"""Fault controls for retained V9 Criterion distributions and provenance."""

from __future__ import annotations

import argparse
import json
import tempfile
import unittest
from pathlib import Path

import v9_criterion as v9


class V9CriterionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)

    def pair(self, number: int, candidate_ns: float = 100.0,
             host_node: str = "host-a") -> Path:
        path = self.root / f"pair-{number}"
        host = {"machine": "arm64", "system": "Darwin", "release": "1",
                "node": host_node, "cpu": "Apple", "cpu_count": "8",
                "rustc": "rustc 1.98.1", "cargo": "cargo 1.98.1"}
        crates = {}
        for name, (bench, _, cases) in v9.GROUPS.items():
            row = {"baseline_source": {"revision": "a" * 40},
                   "candidate_source": {"revision": "b" * 40},
                   "copied_harness_sha256": {"bench": "c" * 64}, "cases": {}}
            for side, duration in (("baseline", 100.0), ("candidate", candidate_ns)):
                log = path / name / f"{side}.log"
                log.parent.mkdir(parents=True, exist_ok=True)
                log.write_text("Criterion run\n")
                row[f"{side}_run"] = {
                    "argv": ["cargo", "bench", "--locked", "--offline",
                             *(["--features", "infinite-trace"] if name == "mltl" else []),
                             "--bench", bench, "--", "--noplot"],
                    "exit_code": 0, "log_sha256": v9.sha256(log.read_bytes())}
                for case in cases:
                    directory = path / "samples" / name / case / side
                    directory.mkdir(parents=True, exist_ok=True)
                    (directory / "sample.json").write_text(json.dumps({
                        "iters": [1] * v9.SAMPLES, "times": [duration] * v9.SAMPLES}))
                    (directory / "estimates.json").write_text("{}")
                    row["cases"].setdefault(case, {})[side] = v9.samples(directory)
            crates[name] = row
        (path / "pair.json").write_text(json.dumps({
            "schema": "tl-mltl.v9-criterion-pair/v1", "host_before": host,
            "host_after": host, "crates": crates, "status": "measured"}))
        return path

    def report(self, *paths: Path) -> dict:
        output = self.root / "report.json"
        v9.report(argparse.Namespace(pair_dir=list(paths), output=output))
        return json.loads(output.read_text())

    def test_two_paired_distributions_pass_below_threshold(self) -> None:
        result = self.report(self.pair(1), self.pair(2, 110.0))
        self.assertEqual(result["status"], "passed")
        self.assertEqual(len(result["cases"]), 28)
        self.assertEqual(result["cases"][0]["runs"][0]["baseline"]["sample_count"], 20)

    def test_spike_requires_repeat_and_two_confirmations_require_finding(self) -> None:
        first, second = self.pair(1, 130.0), self.pair(2)
        self.assertEqual(self.report(first, second)["status"], "repeat_required")
        third = self.pair(3, 130.0)
        self.assertEqual(self.report(first, second, third)["status"], "finding_required")

    def test_host_mismatch_is_inconclusive(self) -> None:
        result = self.report(self.pair(1), self.pair(2, host_node="host-b"))
        self.assertEqual(result["status"], "inconclusive_host")

    def test_changed_raw_sample_or_log_is_rejected(self) -> None:
        first, second = self.pair(1), self.pair(2)
        sample = first / "samples" / "parse" / "bounded_small" / "baseline" / "sample.json"
        sample.write_text('{"iters": [], "times": []}')
        with self.assertRaisesRegex(ValueError, "vacuous"):
            self.report(first, second)
        self.pair(1)
        log = first / "parse" / "baseline.log"
        log.write_text("changed")
        with self.assertRaisesRegex(ValueError, "raw log changed"):
            self.report(first, second)

    def test_empty_and_failed_population_remain_incomplete(self) -> None:
        self.assertEqual(self.report()["status"], "incomplete")
        first = self.pair(1)
        metadata = json.loads((first / "pair.json").read_text())
        metadata["status"] = "incomplete"
        metadata["reason"] = "baseline_benchmark_failed"
        (first / "pair.json").write_text(json.dumps(metadata))
        self.assertEqual(self.report(first)["status"], "incomplete")


if __name__ == "__main__":
    unittest.main()
