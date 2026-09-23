"""Fault controls for the current-only Quire gate's retirement boundary."""

import tempfile
import unittest
from pathlib import Path

import check_current_coverage as gate


class CurrentCoverageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        for relative in gate.SUPERSEDED:
            path = self.root / relative
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("---\nstatus: superseded\n---\n")
        self.current = "spec/v1-verification-test-matrix.md"
        self.old = "spec/corpus-campaign-test-matrix.md"

    def report(self) -> dict:
        return {
            "unbacked_rows": [], "status_lies": [], "diagnostics": [],
            "groups": [
                {"document": self.current, "backed": 1, "total": 2},
                {"document": self.old, "backed": 0, "total": 1},
            ],
            "totals": {"backed": 1, "total": 3},
        }

    def test_only_explicit_predecessors_are_exempt(self) -> None:
        report = self.report()
        report["unbacked_rows"] = [
            {"document": self.current, "row_id": "TC-198"},
            {"document": self.old, "row_id": "TC-091"},
            {"document": "spec/new-superseded.md", "row_id": "TC-999"},
        ]
        summary, current, lies = gate.summarize(report, self.root)
        self.assertEqual([row["row_id"] for row in current], ["TC-198", "TC-999"])
        self.assertEqual(summary["superseded_unbacked_references"], 1)
        self.assertEqual(lies, [])

    def test_historical_status_drift_refuses(self) -> None:
        (self.root / self.old).write_text("---\nstatus: active\n---\n")
        with self.assertRaisesRegex(ValueError, "status changed"):
            gate.summarize(self.report(), self.root)

    def test_current_status_lie_and_unread_measurement_refuse(self) -> None:
        report = self.report()
        report["status_lies"] = [
            {"document": self.current, "row_id": "TC-198"},
            {"document": self.old, "row_id": "TC-091"},
        ]
        summary, _, lies = gate.summarize(report, self.root)
        self.assertEqual(len(lies), 1)
        self.assertEqual(summary["superseded_status_lies"], 1)
        report["diagnostics"] = [{"reason": "hollow-denominator"}]
        with self.assertRaisesRegex(ValueError, "could not evaluate"):
            gate.summarize(report, self.root)

    def test_empty_current_scope_and_false_totals_refuse(self) -> None:
        report = self.report()
        report["groups"] = [report["groups"][1]]
        report["totals"] = {"backed": 0, "total": 1}
        with self.assertRaisesRegex(ValueError, "no current target rows"):
            gate.summarize(report, self.root)
        report["groups"].append({"document": self.current, "backed": 1, "total": 2})
        with self.assertRaisesRegex(ValueError, "totals disagree"):
            gate.summarize(report, self.root)


if __name__ == "__main__":
    unittest.main()
