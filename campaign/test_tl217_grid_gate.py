"""TL-217: live grid receipt faults must fail independent reconciliation."""

import json
from pathlib import Path
import tempfile
import unittest

import tl217_grid_gate as gate


class GridGateTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.raw = Path(self.temp.name)
        self.report = {
            "schema": "tl-mltl.r2u2-past-grid/v1",
            "source_revision": "candidate",
            "source_state": "clean",
            "cargo_lock_sha256": "lock",
            "target_revision": gate.TARGET_REVISION,
            "compiler_sha256": gate.COMPILER_SHA256,
            "monitor_sha256": gate.MONITOR_SHA256,
            "license": "Apache-2.0",
            "intervals": [[name, *interval] for name, interval in gate.INTERVALS.items()],
            "steps": 6,
            "formula_trace_cases": 225,
            "per_step_cells": 1350,
            "unexplained_admitted_cells": 0,
            "classifications": {"agreement": 1350},
            "runs": {},
            "rows": [],
            "artifacts": {},
        }
        cases = gate.expected_cases()
        for group in [*gate.INTERVALS, "previous"]:
            count = 3 if group == "previous" else 12
            for trace in gate.TRACES:
                run = f"{group}-{trace}"
                self.report["runs"][run] = {
                    "target": {"compiler_exit": 0, "monitor_exit": 0,
                               "formula_count": count, "trace_positions": 6},
                    "extra_target_positions": 0,
                }
                for kind in gate.ARTIFACT_KINDS:
                    name = f"{run}.{kind}"
                    data = ("".join(f"{formula}:{position},F\n"
                                    for formula in range(count) for position in range(6)).encode()
                            if kind == "monitor.stdout" else b"fixture")
                    (self.raw / name).write_bytes(data)
                    self.report["artifacts"][name] = gate.sha256(data)
        for case, (operator, interval, depth, _, _) in cases.items():
            for trace in gate.TRACES:
                for position in range(6):
                    self.report["rows"].append({
                        "case": case, "operator": operator, "interval": interval,
                        "depth": depth, "trace": trace, "position": position,
                        "tl": False, "oracle": False, "target": False,
                        "origin_hazard": (
                            operator in ("historically", "triggered")
                            and interval is not None and position < interval[1] * depth
                        ) or (operator == "previous" and position < depth),
                        "mapping": {"status": "admitted"},
                        "classification": "agreement",
                    })

    def verify(self) -> dict:
        data = gate.MARKER + json.dumps(self.report).encode() + b"\n"
        return gate.verify(data, self.raw, "candidate", "lock")

    def test_full_population_and_raw_rows_reconcile(self) -> None:
        self.assertEqual(self.verify()["visited"], 1350)

    def test_omitted_cell_cannot_receive_credit(self) -> None:
        self.report["rows"].pop()
        with self.assertRaisesRegex(ValueError, "population"):
            self.verify()

    def test_wrong_verdict_cannot_receive_credit(self) -> None:
        self.report["rows"][0]["target"] = True
        with self.assertRaisesRegex(ValueError, "raw target"):
            self.verify()

    def test_raw_edit_cannot_receive_credit(self) -> None:
        (self.raw / "previous-all-true.monitor.stdout").write_bytes(b"0:0,T\n")
        with self.assertRaisesRegex(ValueError, "digest mismatch"):
            self.verify()

    def test_false_success_count_cannot_receive_credit(self) -> None:
        self.report["classifications"] = {"agreement": 1349}
        with self.assertRaisesRegex(ValueError, "population/count"):
            self.verify()

    def test_origin_hazard_cannot_be_suppressed(self) -> None:
        row = next(row for row in self.report["rows"] if row["case"] == "historically-0-1-d1"
                   and row["position"] == 0)
        row["origin_hazard"] = False
        with self.assertRaisesRegex(ValueError, "origin hazard"):
            self.verify()


if __name__ == "__main__":
    unittest.main()
