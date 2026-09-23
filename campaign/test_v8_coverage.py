"""Coverage export controls for production scoping and branch gaps."""

import tempfile
import unittest
from pathlib import Path

from v8_coverage import classify_export


class CoverageExportTests(unittest.TestCase):
    def test_production_branch_gap_is_located_and_tests_are_excluded(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            production = root / "src" / "future.rs"
            test_file = root / "tests" / "future.rs"
            production.parent.mkdir()
            test_file.parent.mkdir()
            production.write_text("fn check() {}\n")
            test_file.write_text("fn test() {}\n")
            def file(path):
                return {"filename": str(path),
                        "summary": {"lines": {"count": 1, "covered": 1},
                                    "branches": {"count": 2, "covered": 1}},
                        "branches": [[1, 2, 1, 12, 7, 0, 0, 0, 4]]}
            export = {"type": "llvm.coverage.json.export", "data": [
                {"files": [file(production), file(test_file)]}]}
            measured = classify_export(export, root)
            self.assertEqual(list(measured["files"]), ["src/future.rs"])
            self.assertEqual(measured["totals"]["branches"], {"count": 2, "covered": 1})
            self.assertEqual(measured["files"]["src/future.rs"]
                             ["uncovered_branch_locations"], [
                                 {"line": 1, "column": 2, "true_count": 7,
                                  "false_count": 0}])

    def test_empty_or_impossible_production_measurement_refuses(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(ValueError, "no production branches"):
                classify_export({"type": "llvm.coverage.json.export",
                                 "data": [{"files": []}]}, root)
            production = root / "src" / "future.rs"
            production.parent.mkdir()
            production.write_text("fn check() {}\n")
            with self.assertRaisesRegex(ValueError, "impossible coverage counts"):
                classify_export({"type": "llvm.coverage.json.export", "data": [{"files": [
                    {"filename": str(production),
                     "summary": {"lines": {"count": 1, "covered": 2},
                                 "branches": {"count": 1, "covered": 1}},
                     "branches": [[1, 0, 1, 1, 1, 1, 0, 0, 4]]}]}]}, root)

    def test_summary_gap_cannot_hide_missing_detail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            file = root / "src" / "future.rs"
            file.parent.mkdir()
            file.write_text("fn check() {}\n")
            export = {"type": "llvm.coverage.json.export", "data": [{"files": [{
                "filename": str(file),
                "summary": {"lines": {"count": 1, "covered": 1},
                            "branches": {"count": 2, "covered": 1}},
                "branches": [],
            }]}]}
            with self.assertRaisesRegex(ValueError, "missing detailed branch population"):
                classify_export(export, root)

    def test_summary_cannot_claim_more_branches_than_detail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            file = root / "src" / "future.rs"
            file.parent.mkdir()
            file.write_text("fn check() {}\n")
            export = {"type": "llvm.coverage.json.export", "data": [{"files": [{
                "filename": str(file),
                "summary": {"lines": {"count": 1, "covered": 1},
                            "branches": {"count": 4, "covered": 4}},
                "branches": [[1, 2, 1, 12, 7, 8, 0, 0, 4]],
            }]}]}
            with self.assertRaisesRegex(ValueError, "summary exceeds detailed population"):
                classify_export(export, root)

    def test_repeated_instantiations_merge_before_locating_gaps(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            file = root / "src" / "future.rs"
            file.parent.mkdir()
            file.write_text("fn check() {}\n")
            export = {"type": "llvm.coverage.json.export", "data": [{"files": [{
                "filename": str(file),
                "summary": {"lines": {"count": 1, "covered": 1},
                            "branches": {"count": 2, "covered": 2}},
                "branches": [[1, 2, 1, 12, 0, 0, 0, 0, 4],
                             [1, 2, 1, 12, 7, 8, 0, 0, 4]],
            }]}]}
            measured = classify_export(export, root)
            self.assertEqual(measured["files"]["src/future.rs"]
                             ["uncovered_branch_locations"], [])

    def test_summary_gap_survives_opposite_instantiation_outcomes(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            file = root / "src" / "future.rs"
            file.parent.mkdir()
            file.write_text("fn check() {}\n")
            export = {"type": "llvm.coverage.json.export", "data": [{"files": [{
                "filename": str(file),
                "summary": {"lines": {"count": 1, "covered": 1},
                            "branches": {"count": 4, "covered": 2}},
                "branches": [[1, 2, 1, 12, 0, 7, 0, 0, 4],
                             [1, 2, 1, 12, 3, 0, 0, 0, 4]],
            }]}]}
            measured = classify_export(export, root)
            self.assertEqual(measured["files"]["src/future.rs"]
                             ["uncovered_branch_locations"], [
                                 {"line": 1, "column": 2, "true_count": 0,
                                  "false_count": 0}])

    def test_summary_counts_monomorphized_branches_beyond_unique_sites(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            file = root / "src" / "future.rs"
            file.parent.mkdir()
            file.write_text("fn check() {}\n")
            export = {"type": "llvm.coverage.json.export", "data": [{"files": [{
                "filename": str(file),
                "summary": {"lines": {"count": 1, "covered": 0},
                            "branches": {"count": 4, "covered": 0}},
                "branches": [[1, 2, 1, 12, 0, 0, 0, 0, 4],
                             [1, 2, 1, 12, 0, 0, 0, 0, 4]],
            }]}]}
            measured = classify_export(export, root)
            self.assertEqual(measured["files"]["src/future.rs"]
                             ["uncovered_branch_locations"], [
                                 {"line": 1, "column": 2, "true_count": 0,
                                  "false_count": 0}])

    def test_zero_summary_may_have_uninstantiated_detail(self):
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            counted = root / "src" / "future.rs"
            uncounted = root / "src" / "limits.rs"
            counted.parent.mkdir()
            counted.write_text("fn check() {}\n")
            uncounted.write_text("const fn minimum() {}\n")
            export = {"type": "llvm.coverage.json.export", "data": [{"files": [
                {"filename": str(counted),
                 "summary": {"lines": {"count": 1, "covered": 1},
                             "branches": {"count": 2, "covered": 2}},
                 "branches": [[1, 2, 1, 12, 7, 8, 0, 0, 4]]},
                {"filename": str(uncounted),
                 "summary": {"lines": {"count": 1, "covered": 0},
                             "branches": {"count": 0, "covered": 0}},
                 "branches": [[1, 2, 1, 12, 0, 0, 0, 0, 4]]},
            ]}]}
            measured = classify_export(export, root)
            self.assertEqual(measured["files"]["src/limits.rs"]
                             ["uncovered_branch_locations"], [])


if __name__ == "__main__":
    unittest.main()
