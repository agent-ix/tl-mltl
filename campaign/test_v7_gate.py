"""TC-187/188: V7 gate rejects stale, missing, and altered native evidence."""

from __future__ import annotations

import json
import shutil
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import make_manifest
import v1_campaign
import v7_gate


ARCHIVE = Path(__file__).resolve().parent / "evidence"


class V7GateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.native = self.root / "embedded_miri_limits_native"
        self.native.mkdir()
        self.report_path = self.root / "embedded_miri_limits_native.json"
        self.report = json.loads((ARCHIVE / "v7-native.json").read_text())
        for probe in self.report["probes"]:
            name = probe["name"]
            shutil.copyfile(ARCHIVE / "tl-v7-raw" / f"{name}.log",
                            self.native / f"{name}.log")
            probe["raw_path"] = f"embedded_miri_limits_native/{name}.log"
        self.graph = {f"tl-{name}": {"revision": revision}
                      for name, revision in self.report["source_revisions"].items()}
        self.tools = self.report["tool_versions"]

    def check(self, report: dict | None = None, summary: dict | None = None) -> tuple[dict, dict]:
        report = self.report if report is None else report
        summary = summary or {"status": "passed", "counts": report["counts"],
                              "output": str(self.report_path.resolve())}
        return v7_gate.verify(json.dumps(report).encode(), json.dumps(summary).encode(),
                              self.root, self.graph, self.tools)

    def test_real_native_logs_reconcile_and_faults_fail(self) -> None:
        population, artifacts = self.check()
        self.assertEqual((population["embedded_builds"], population["miri_tests"],
                          population["paired_edges"], population["refusal_only_edges"]),
                         (3, 8, 34, 3))
        self.assertEqual(len(artifacts), 12)
        self.assertEqual(artifacts["oracle_limits"]["sha256"],
                         self.report["probes"][-1]["raw_sha256"])
        mutations = [
            lambda value: value["source_revisions"].update(oracle="0" * 40),
            lambda value: value["tool_versions"].update(miri="invented"),
            lambda value: value["probes"][0].update(status="failed"),
            lambda value: value["probes"][0].update(argv=["cargo", "build"]),
            lambda value: value["probes"][1].update(feature="default"),
            lambda value: value["probes"][2].update(paired_edges=1),
            lambda value: value["probes"][3].update(raw_sha256="0" * 64),
            lambda value: value["probes"].pop(),
            lambda value: value["counts"].update(paired_edges=33),
            lambda value: value["not_run_under_miri"].pop(),
        ]
        for mutate in mutations:
            changed = json.loads(json.dumps(self.report))
            mutate(changed)
            with self.assertRaises((ValueError, OSError)):
                self.check(changed)
        with self.assertRaises(ValueError):
            self.check(summary={"status": "passed", "counts": self.report["counts"],
                                "output": "/other/run.json"})
        raw = self.native / "oracle_limits.log"
        original = raw.read_bytes()
        raw.write_bytes(original.replace(b"test tc_188_oracle_limit_edges_are_typed ... ok",
                                         b"test tc_188_oracle_limit_edges_are_typed ... FAILED"))
        with self.assertRaises(ValueError):
            self.check()
        raw.write_bytes(original)
        changed = original.replace(b"test tc_188_oracle_limit_edges_are_typed ... ok",
                                   b"test tc_188_oracle_limit_edges_are_typed ... FAILED")
        raw.write_bytes(changed)
        altered = json.loads(json.dumps(self.report))
        altered["probes"][-1]["raw_sha256"] = v7_gate.digest(changed)
        with self.assertRaises(ValueError):
            self.check(altered)
        raw.write_bytes(original)
        duplicate_key = json.dumps(self.report).replace('"target":', '"target":"fake","target":', 1)
        with self.assertRaises(ValueError):
            v7_gate.verify(duplicate_key.encode(), b"{}", self.root, self.graph, self.tools)

    def test_opt_in_manifest_names_the_exact_v7_command(self) -> None:
        repos = self.root / "repos"
        cargo_home = self.root / "cargo-home"
        with patch.object(make_manifest, "git_revision", return_value="a" * 40), \
             patch.object(make_manifest, "corpus_paths", return_value=[]):
            ordinary = make_manifest.make_manifest(repos)
            opted = make_manifest.make_manifest(repos, v7_cargo_home=cargo_home)
        self.assertNotIn("embedded_miri_limits", {row["id"] for row in ordinary["lanes"]})
        lane = next(row for row in opted["lanes"] if row["id"] == "embedded_miri_limits")
        self.assertEqual(lane["argv"], [make_manifest.COMMAND_CONTRACTS[
            "embedded_miri_limits"][2][0], "campaign/v7_native.py"])
        self.assertEqual(lane["parser"], "v7_native")
        self.assertEqual(lane["cargo_home"], str(cargo_home.resolve()))
        self.assertEqual(lane["timeout_seconds"], 3600)

    def test_required_gate_refuses_forged_command_and_archived_record(self) -> None:
        repo, parser, argv = v1_campaign.COMMAND_CONTRACTS["embedded_miri_limits"]
        base = {"id": "embedded_miri_limits", "milestone": "V7", "mode": "command",
                "repo": repo, "parser": parser, "argv": argv,
                "seed": {"kind": "none", "reason": "bounded_native_probes"}}
        graph = {"tl-mltl": {"path": str(self.root)}}
        rejected, _ = v1_campaign.run_lane(base | {"argv": ["echo", "passed"]},
                                           graph, {}, self.root / "raw")
        self.assertEqual(rejected["reason"], "unregistered_required_gate")
        rejected, _ = v1_campaign.run_lane(base, graph, {}, self.root / "raw")
        self.assertEqual(rejected["reason"], "v7_cargo_home_required")
        rejected, _ = v1_campaign.run_lane(base | {"mode": "record", "receipt": "archive.json"},
                                           graph, {}, self.root / "raw")
        self.assertEqual(rejected["reason"], "v7_requires_live_execution")


if __name__ == "__main__":
    unittest.main()
