"""Raw pair controls for the V9 Criterion campaign gate."""

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import v9_criterion
import v9_gate


class V9GateTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name).resolve()
        self.pair_dirs = [self.root / "pair-1", self.root / "pair-2"]
        self.graph = {f"tl-{name}": {"revision": str(index) * 40,
                                      "cargo_toml_sha256": str(index + 4) * 64,
                                      "cargo_lock_sha256": str(index + 7) * 64}
                      for index, name in enumerate(v9_criterion.GROUPS, 1)}
        self.inputs = {}
        self.host = {"machine": "test-arm64", "rustc": "test-rustc"}
        self.report_path = self.root / "performance-native.json"
        for index, pair_dir in enumerate(self.pair_dirs):
            pair_dir.mkdir()
            crates = {}
            for name, (bench, group, cases) in v9_criterion.GROUPS.items():
                staged = pair_dir / name / "baseline_source"
                (staged / "benches" / "inputs").mkdir(parents=True)
                (staged / "benches" / f"{bench}.rs").write_text("fn main() {}\n")
                (staged / "benches" / "inputs" / "SHA256SUMS").write_text("")
                (staged / "benches" / "input-digests.json").write_text("{}\n")
                (staged / "Cargo.toml").write_text("[package]\nname = \"fixture\"\n")
                (staged / "Cargo.lock").write_text("# fixture\n")
                logs = {}
                distributions = {}
                for side, time in (("baseline", 1000.0), ("candidate", 1010.0)):
                    log = pair_dir / name / f"{side}.log"
                    log.parent.mkdir(exist_ok=True)
                    log.write_text("Criterion complete\n")
                    logs[side] = {"argv": ["cargo", "bench", "--locked", "--offline",
                                           *(["--features", "infinite-trace"] if name == "mltl"
                                             else []), "--bench", bench, "--", "--noplot"],
                                  "exit_code": 0,
                                  "log_sha256": hashlib.sha256(log.read_bytes()).hexdigest()}
                    for case in cases:
                        sample_dir = pair_dir / "samples" / name / case / side
                        sample_dir.mkdir(parents=True)
                        (sample_dir / "sample.json").write_text(json.dumps({
                            "iters": [1] * 20, "times": [time] * 20}))
                        (sample_dir / "estimates.json").write_text("{}")
                        distributions.setdefault(case, {})[side] = v9_criterion.samples(sample_dir)
                source = self.graph[f"tl-{name}"]
                crates[name] = {
                    "baseline_source": {
                        "revision": "a" * 40,
                        "manifest_sha256": v9_criterion.sha256(
                            (staged / "Cargo.toml").read_bytes()),
                        "lock_sha256": v9_criterion.sha256(
                            (staged / "Cargo.lock").read_bytes()),
                    },
                    "candidate_source": {"revision": source["revision"],
                                         "manifest_sha256": source["cargo_toml_sha256"],
                                         "lock_sha256": source["cargo_lock_sha256"]},
                    "copied_harness_sha256": v9_criterion.harness(staged, name),
                    "baseline_stage_sha256": v9_criterion.stage_digest(staged),
                    "baseline_run": logs["baseline"], "candidate_run": logs["candidate"],
                    "cases": distributions,
                }
            pair = {"schema": "tl-mltl.v9-criterion-pair/v1", "status": "measured",
                    "host_before": self.host, "host_after": self.host, "crates": crates}
            pair_path = pair_dir / "pair.json"
            pair_path.write_text(json.dumps(pair))
            self.inputs[f"v9_pair_{index}"] = {
                "path": str(pair_path.resolve()),
                "sha256": hashlib.sha256(pair_path.read_bytes()).hexdigest()}
        self.report = {"schema": "tl-mltl.v9-criterion-report/v1", "status": "passed",
                       "threshold": 0.20, "required_pairs": 2, "pair_count": 2,
                       "pair_dirs": [str(path) for path in self.pair_dirs],
                       "source_revisions": {name: ["a" * 40, self.graph[f"tl-{name}"]["revision"]]
                                            for name in v9_criterion.GROUPS},
                       "host": self.host, "cases": [], "reason": None}
        with patch.object(v9_criterion, "BOOTSTRAPS", 100):
            for name, (_, _, cases) in v9_criterion.GROUPS.items():
                for case in cases:
                    runs = []
                    for pair_dir in self.pair_dirs:
                        native = json.loads((pair_dir / "pair.json").read_text())
                        distribution = native["crates"][name]["cases"][case]
                        runs.append(v9_criterion.classify_pair(
                            distribution["baseline"], distribution["candidate"],
                            f"{name}/{case}/{pair_dir.name}"))
                    self.report["cases"].append({"crate": name, "case": case,
                                                 "runs": runs,
                                                 "status": "below_20pct_threshold"})

    def verify(self, report=None):
        with patch.object(v9_criterion, "BOOTSTRAPS", 100):
            return v9_gate.verify(json.dumps(report or self.report).encode(), self.report_path,
                                  self.pair_dirs, self.graph, self.inputs)

    def test_two_complete_pinned_pairs_pass(self):
        status, population, artifacts = self.verify()
        self.assertEqual(status, "passed")
        self.assertEqual(len(population["cases"]), 28)
        self.assertEqual(len(artifacts), 3)

    def test_restamped_change_and_mutated_raw_sample_refuse(self):
        report = json.loads(json.dumps(self.report))
        report["cases"][0]["runs"][0]["median_change_ratio"] = 0.0
        with self.assertRaisesRegex(ValueError, "classification was restamped"):
            self.verify(report)
        sample = self.pair_dirs[0] / "samples" / "parse" / "bounded_small" / "candidate" / "sample.json"
        sample.write_text("{}")
        with self.assertRaises((ValueError, KeyError)):
            self.verify()


if __name__ == "__main__":
    unittest.main()
