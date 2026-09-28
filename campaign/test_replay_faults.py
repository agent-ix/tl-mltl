"""Local checks for the disposable Campaign fault replayer."""

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path

import replay_faults


class ReplayFaultTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.original = self.root / "original"
        self.copy = self.root / "copy"
        store = self.original / "spec/evidence/campaigns"
        (store / "runs").mkdir(parents=True)
        (store / "domain-verdicts").mkdir()
        (store / "results").mkdir()
        (store / "raw").mkdir()
        self.run_id = "sample"
        self.run = {"id": self.run_id, "verdict": "accepted", "sourceGraphDigest": "a" * 64,
                    "attempts": [{"member": "V1.independent_oracle", "index": 1,
                                  "status": "completed", "domainVerdictDigest": "b" * 64,
                                  "checkerResultDigest": "d" * 64,
                                  "rawArtifacts": [{"digest": "c" * 64, "role": "data"}]},
                                 {"member": "V2.other", "index": 1, "status": "completed",
                                  "rawArtifacts": []}]}
        replay_faults.save(store / "runs/sample.json", self.run)
        (store / "domain-verdicts" / ("b" * 64 + ".json")).write_text("{}")
        (store / "results" / ("d" * 64 + ".json")).write_text("{}")
        (store / "raw" / ("c" * 64 + ".bin")).write_bytes(b"original")
        replay_faults.copied_store(self.original / "spec/evidence",
                                   self.copy / "spec/evidence")

    def test_every_mutation_leaves_original_bytes_untouched(self) -> None:
        baseline = {path.relative_to(self.original): path.read_bytes()
                    for path in self.original.rglob("*") if path.is_file()}
        for case in ("omission", "repeated_attempt", "absent_checker", "altered_raw",
                     "stale_source"):
            with self.subTest(case=case):
                selected = self.run["attempts"][0]
                replay_faults.mutate(case, self.copy, self.run_id, selected)
                changed_run = replay_faults.load(
                    self.copy / "spec/evidence/campaigns/runs/sample.json")
                self.assertEqual(changed_run["verdict"], self.run["verdict"])
                if case == "omission":
                    self.assertEqual([row["member"] for row in changed_run["attempts"]],
                                     ["V2.other"])
                elif case == "repeated_attempt":
                    self.assertEqual(changed_run["attempts"][-1], selected)
                elif case == "stale_source":
                    self.assertEqual(changed_run["sourceGraphDigest"], "0" * 64)
                for relative, expected in baseline.items():
                    self.assertEqual((self.original / relative).read_bytes(), expected)
                if case == "absent_checker":
                    self.assertFalse((self.copy / "spec/evidence/campaigns/results" /
                                      ("d" * 64 + ".json")).exists())
                    self.assertTrue((self.copy / "spec/evidence/campaigns/domain-verdicts" /
                                     ("b" * 64 + ".json")).exists())
                if case == "altered_raw":
                    self.assertNotEqual((self.copy / "spec/evidence/campaigns/raw" /
                                         ("c" * 64 + ".bin")).read_bytes(), b"original")
                # Each fault starts from the same original retained run.
                for path in (self.copy / "spec/evidence").rglob("*"):
                    if path.is_file():
                        path.unlink()
                replay_faults.copied_store(self.original / "spec/evidence",
                                           self.copy / "spec/evidence")

    def test_replay_refuses_accept_and_sibling_changes(self) -> None:
        member = lambda name, verdict: {"name": name, "verdict": verdict, "reasons": [],
                                        "attempts": 1, "required": True, "group": None}
        baseline = {"decision": {"members": [member("target", "accept"),
                                              member("sibling", "accept")]}}
        rejected = {"decision": {"verdict": "reject",
                                 "members": [member("target", "reject"),
                                             member("sibling", "accept")]}}
        self.assertEqual(replay_faults.check_case("altered_raw", baseline, rejected, "exit=1",
                                                 "target", set())["result"], "reject")
        rejected["decision"]["members"][1]["verdict"] = "reject"
        with self.assertRaisesRegex(AssertionError, "unrelated sibling"):
            replay_faults.check_case("altered_raw", baseline, rejected, "exit=1",
                                     "target", set())
        with self.assertRaisesRegex(AssertionError, "no independent receipt"):
            replay_faults.check_case("absent_checker", baseline, None, "exit=2",
                                     "target", set())


if __name__ == "__main__":
    unittest.main()
